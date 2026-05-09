// HTTP request handlers for handy-cli

use crate::models::registry::ModelRegistry;
use crate::transcriber::{EngineType, TranscriptionResult, WhisperTranscriber, SenseVoiceTranscriber};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

// Global transcriber instance (lazy loaded)
static TRANSCRIBER: Lazy<Mutex<Option<TranscriberWrapper>>> = Lazy::new(|| Mutex::new(None));

enum TranscriberWrapper {
    Whisper(WhisperTranscriber),
    SenseVoice(SenseVoiceTranscriber),
}

pub struct RouterState {
    pub engine: String,
    pub model: Option<String>,
    pub vad_threshold: f32,
    pub language: String,
    pub model_dir: PathBuf,
}

impl RouterState {
    pub fn new(
        engine: String,
        model: Option<String>,
        vad_threshold: f32,
        language: String,
    ) -> Self {
        // Default model directory
        let model_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("handy-cli")
            .join("models");

        Self {
            engine,
            model,
            vad_threshold,
            language,
            model_dir,
        }
    }

    pub fn load_transcriber(&self) -> Result<(), String> {
        let mut guard = TRANSCRIBER.lock().map_err(|e| e.to_string())?;
        
        // Check if already loaded with same engine
        if let Some(ref current) = *guard {
            match (current, self.engine.as_str()) {
                (TranscriberWrapper::Whisper(_), "whisper") => return Ok(()),
                (TranscriberWrapper::SenseVoice(_), "sensevoice") => return Ok(()),
                _ => {} // Need to reload
            }
        }

        let model_id = self.model.clone().unwrap_or_else(|| {
            if self.engine == "sensevoice" {
                "sense-voice-int8".to_string()
            } else {
                "small".to_string()
            }
        });

        let model_path = self.model_dir.join(&model_id);
        
        if !model_path.exists() {
            return Err(format!("Model not found: {:?}. Run `handy-cli download --model {}` first", model_path, model_id));
        }

        let transcriber = match self.engine.as_str() {
            "whisper" => {
                // For whisper, we need the .bin file
                let bin_path = model_path.join(format!("ggml-{}.bin", model_id));
                let path = if bin_path.exists() { bin_path } else { model_path };
                TranscriberWrapper::Whisper(WhisperTranscriber::new(&path)
                    .map_err(|e| format!("Failed to load Whisper: {}", e))?)
            }
            "sensevoice" => {
                TranscriberWrapper::SenseVoice(SenseVoiceTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load SenseVoice: {}", e))?)
            }
            _ => return Err(format!("Unknown engine: {}", self.engine)),
        };

        *guard = Some(transcriber);
        Ok(())
    }

    pub fn transcribe(&self, audio: &[f32], language: Option<&str>) -> Result<TranscriptionResult, String> {
        let mut guard = TRANSCRIBER.lock().map_err(|e| e.to_string())?;
        
        let lang = language.or_else(|| {
            if self.language != "auto" { Some(self.language.as_str()) } else { None }
        });

        match guard.as_mut() {
            Some(TranscriberWrapper::Whisper(t)) => {
                t.transcribe(audio, lang).map_err(|e| e.to_string())
            }
            Some(TranscriberWrapper::SenseVoice(t)) => {
                t.transcribe(audio, lang).map_err(|e| e.to_string())
            }
            None => Err("Transcriber not loaded. Call /api/health first".to_string()),
        }
    }
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub engine: String,
    pub model: Option<String>,
    pub loaded: bool,
}

pub async fn health(State(state): State<Arc<RouterState>>) -> Result<Json<HealthResponse>, AppError> {
    // Try to load the transcriber
    let loaded = state.load_transcriber().is_ok();

    Ok(Json(HealthResponse {
        status: if loaded { "ok".to_string() } else { "model_not_loaded".to_string() },
        engine: state.engine.clone(),
        model: state.model.clone(),
        loaded,
    }))
}

pub async fn list_models() -> Json<serde_json::Value> {
    let models = ModelRegistry::available_models();
    Json(json!({
        "models": models
    }))
}

pub async fn list_downloaded_models(State(state): State<Arc<RouterState>>) -> Json<serde_json::Value> {
    let mut downloaded = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&state.model_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                downloaded.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    Json(json!({
        "downloaded": downloaded
    }))
}

#[derive(Deserialize)]
pub struct TranscribeRequest {
    pub audio: String,  // base64 encoded audio
    pub language: Option<String>,
    pub sample_rate: Option<u32>,
}

#[derive(Serialize)]
pub struct TranscribeResponse {
    pub text: String,
    pub language: Option<String>,
    pub duration: f32,
}

pub async fn transcribe(
    State(state): State<Arc<RouterState>>,
    Json(req): Json<TranscribeRequest>,
) -> Result<Json<TranscribeResponse>, AppError> {
    // Decode base64 audio
    let audio_bytes = BASE64.decode(&req.audio)
        .map_err(|e| AppError::BadRequest(format!("Invalid base64 audio: {}", e)))?;

    // Convert to f32 samples (expects 16-bit PCM mono)
    let samples: Vec<f32> = audio_bytes
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                let s = i16::from_le_bytes([chunk[0], chunk[1]]);
                Some(s as f32 / i16::MAX as f32)
            } else {
                None
            }
        })
        .collect();

    if samples.is_empty() {
        return Err(AppError::BadRequest("No audio samples found".to_string()));
    }

    // Resample if needed (transcribe-rs expects 16kHz)
    let sample_rate = req.sample_rate.unwrap_or(16000);
    let final_samples = if sample_rate != 16000 {
        resample_audio(&samples, sample_rate, 16000)?
    } else {
        samples
    };

    // Ensure transcriber is loaded
    state.load_transcriber()
        .map_err(|e| AppError::Internal(format!("Failed to load transcriber: {}", e)))?;

    // Transcribe
    let result = state.transcribe(&final_samples, req.language.as_deref())
        .map_err(|e| AppError::Internal(format!("Transcription failed: {}", e)))?;

    Ok(Json(TranscribeResponse {
        text: result.text,
        language: result.language,
        duration: result.duration,
    }))
}

pub async fn transcribe_stream(
    State(state): State<Arc<RouterState>>,
    Json(req): Json<TranscribeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // For streaming, we just do the same as regular transcribe for now
    // A full implementation would use SSE
    transcribe(State(state), Json(req)).await
    .map(|r| Json(json!({"text": r.text, "language": r.language, "duration": r.duration})))
}

/// Simple resampling using linear interpolation
fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>, AppError> {
    if from_rate == to_rate {
        return Ok(samples.to_vec());
    }

    let ratio = to_rate as f32 / from_rate as f32;
    let new_len = (samples.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f32 / ratio;
        let src_idx_floor = src_idx.floor() as usize;
        let src_idx_ceil = src_idx.ceil() as usize.min(samples.len() - 1);
        let frac = src_idx - src_idx.floor();

        let sample = samples[src_idx_floor] * (1.0 - frac) + samples[src_idx_ceil] * frac;
        resampled.push(sample);
    }

    Ok(resampled)
}

pub struct AppError {
    message: String,
    status: StatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(err: base64::DecodeError) -> Self {
        Self {
            message: format!("Base64 decode error: {}", err),
            status: StatusCode::BAD_REQUEST,
        }
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        Self {
            message: err,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, Option<TranscriberWrapper>>>> for AppError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, Option<TranscriberWrapper>>>) -> Self {
        Self {
            message: "Mutex poisoned".to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
