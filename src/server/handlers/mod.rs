// HTTP request handlers for handy-cli
//
// Supports multiple ASR engines: Whisper, SenseVoice, Parakeet, Moonshine,
// GigaAM, Canary, Cohere.

use crate::models::registry::ModelRegistry;
use crate::transcriber::{
    CanaryTranscriber, CohereTranscriber, GigaAMTranscriber, MoonshineTranscriber,
    ParakeetTranscriber, SenseVoiceTranscriber, Transcriber, TranscriptionResult,
    WhisperTranscriber,
};
use axum::response::sse::{Event as SseEvent, Sse};
use axum::response::Response;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{Stream, StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

// Number of samples to accumulate before triggering transcription (1 second at 16kHz)
#[allow(dead_code)] // Reserved for future use
const STREAM_CHUNK_SAMPLES: usize = 16_000;

// Global transcriber instance (lazy loaded)
static TRANSCRIBER: Lazy<Mutex<Option<TranscriberWrapper>>> = Lazy::new(|| Mutex::new(None));

enum TranscriberWrapper {
    Whisper(WhisperTranscriber),
    SenseVoice(SenseVoiceTranscriber),
    Parakeet(ParakeetTranscriber),
    Moonshine(MoonshineTranscriber),
    GigaAM(GigaAMTranscriber),
    Canary(CanaryTranscriber),
    Cohere(CohereTranscriber),
}

#[allow(dead_code)] // Some variants reserved for future VAD integration
#[derive(Debug, Clone, Serialize)]
pub enum SseEventData {
    #[allow(dead_code)] // Reserved for VAD integration
    SpeechStart {
        timestamp: i64,
    },
    #[allow(dead_code)] // Reserved for VAD integration
    SpeechEnd {
        timestamp: i64,
        duration: f32,
    },
    Transcript {
        text: String,
        partial: bool,
    },
    Error {
        message: String,
    },
}

impl SseEventData {
    fn to_sse_event(&self, event_type: &str) -> SseEvent {
        let data = match self {
            SseEventData::SpeechStart { timestamp } => {
                serde_json::json!({"timestamp": timestamp}).to_string()
            }
            SseEventData::SpeechEnd {
                timestamp,
                duration,
            } => serde_json::json!({"timestamp": timestamp, "duration": duration}).to_string(),
            SseEventData::Transcript { text, partial } => {
                serde_json::json!({"text": text, "partial": partial}).to_string()
            }
            SseEventData::Error { message } => serde_json::json!({"message": message}).to_string(),
        };
        SseEvent::default().event(event_type).data(data)
    }
}

pub struct RouterState {
    pub engine: String,
    pub model: Option<String>,
    #[allow(dead_code)] // Reserved for VAD integration
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

        // Check if already loaded with the same engine
        if let Some(ref current) = *guard {
            let current_engine = match current {
                TranscriberWrapper::Whisper(_) => "whisper",
                TranscriberWrapper::SenseVoice(_) => "sensevoice",
                TranscriberWrapper::Parakeet(_) => "parakeet",
                TranscriberWrapper::Moonshine(_) => {
                    // Check if streaming variant
                    if self.engine.starts_with("moonshine-streaming")
                        || self.engine.starts_with("moonshinestreaming")
                    {
                        "moonshine-streaming"
                    } else {
                        "moonshine"
                    }
                }
                TranscriberWrapper::GigaAM(_) => "gigaam",
                TranscriberWrapper::Canary(_) => "canary",
                TranscriberWrapper::Cohere(_) => "cohere",
            };
            if current_engine == self.engine {
                return Ok(());
            }
        }

        let model_id = self
            .model
            .clone()
            .unwrap_or_else(|| match self.engine.as_str() {
                "sensevoice" => "sense-voice-int8".to_string(),
                "whisper" => "small".to_string(),
                "parakeet" => "parakeet-tdt-0.6b-v2".to_string(),
                "moonshine" | "moonshine-base" => "moonshine-base".to_string(),
                "moonshine-streaming" | "moonshinestreaming" | "moonshine-tiny-streaming-en" => {
                    "moonshine-tiny-streaming-en".to_string()
                }
                "moonshine-small-streaming-en" => "moonshine-small-streaming-en".to_string(),
                "moonshine-medium-streaming-en" => "moonshine-medium-streaming-en".to_string(),
                "gigaam" | "gigaam-v3" => "gigaam-v3-e2e-ctc".to_string(),
                "canary" | "canary-180m" => "canary-180m-flash".to_string(),
                "canary-1b" => "canary-1b-v2".to_string(),
                "cohere" => "cohere-int8".to_string(),
                _ => "small".to_string(),
            });

        // For whisper, model_id refers to the model name (small, base, etc.)
        // The actual file is ggml-{model_id}.bin
        let model_path = if self.engine == "whisper" {
            self.model_dir.join(format!("ggml-{}.bin", model_id))
        } else {
            self.model_dir.join(&model_id)
        };

        if !model_path.exists() {
            return Err(format!(
                "Model not found: {:?}. Run `handy-cli download --model {}` first",
                model_path, model_id
            ));
        }

        let transcriber = match self.engine.as_str() {
            "whisper" => TranscriberWrapper::Whisper(
                WhisperTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load Whisper: {}", e))?,
            ),
            "sensevoice" => TranscriberWrapper::SenseVoice(
                SenseVoiceTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load SenseVoice: {}", e))?,
            ),
            "parakeet" | "parakeet-v2" | "parakeet-v3" => TranscriberWrapper::Parakeet(
                ParakeetTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load Parakeet: {}", e))?,
            ),
            "moonshine" | "moonshine-base" => TranscriberWrapper::Moonshine(
                MoonshineTranscriber::new_auto(&model_path)
                    .map_err(|e| format!("Failed to load Moonshine: {}", e))?,
            ),
            "moonshine-streaming"
            | "moonshinestreaming"
            | "moonshine-tiny-streaming-en"
            | "moonshine-small-streaming-en"
            | "moonshine-medium-streaming-en" => TranscriberWrapper::Moonshine(
                // Moonshine uses a unified model - streaming variants use the same engine
                MoonshineTranscriber::new_auto(&model_path)
                    .map_err(|e| format!("Failed to load Moonshine: {}", e))?,
            ),
            "gigaam" | "gigaam-v3" => TranscriberWrapper::GigaAM(
                GigaAMTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load GigaAM: {}", e))?,
            ),
            "canary" | "canary-180m" | "canary-1b" => TranscriberWrapper::Canary(
                CanaryTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load Canary: {}", e))?,
            ),
            "cohere" => TranscriberWrapper::Cohere(
                CohereTranscriber::new(&model_path)
                    .map_err(|e| format!("Failed to load Cohere: {}", e))?,
            ),
            _ => return Err(format!("Unknown engine: {}", self.engine)),
        };

        *guard = Some(transcriber);
        Ok(())
    }

    pub fn transcribe(
        &self,
        audio: &[f32],
        language: Option<&str>,
        translate: bool,
    ) -> Result<TranscriptionResult, String> {
        let mut guard = TRANSCRIBER.lock().map_err(|e| e.to_string())?;

        let lang = language.or_else(|| {
            if self.language != "auto" {
                Some(self.language.as_str())
            } else {
                None
            }
        });

        match guard.as_mut() {
            Some(TranscriberWrapper::Whisper(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            Some(TranscriberWrapper::SenseVoice(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            Some(TranscriberWrapper::Parakeet(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            Some(TranscriberWrapper::Moonshine(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            Some(TranscriberWrapper::GigaAM(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            Some(TranscriberWrapper::Canary(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            Some(TranscriberWrapper::Cohere(t)) => t
                .transcribe(audio, lang, translate)
                .map_err(|e| e.to_string()),
            None => Err("Transcriber not loaded. Call /api/health first".to_string()),
        }
    }
}

// Health check endpoint
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub engine: String,
    pub model: Option<String>,
    pub loaded: bool,
}

pub async fn health(
    State(state): State<Arc<RouterState>>,
) -> Result<Json<HealthResponse>, AppError> {
    let loaded = state.load_transcriber().is_ok();

    Ok(Json(HealthResponse {
        status: if loaded {
            "ok".to_string()
        } else {
            "model_not_loaded".to_string()
        },
        engine: state.engine.clone(),
        model: state.model.clone(),
        loaded,
    }))
}

// List available models
pub async fn list_models() -> Json<serde_json::Value> {
    let models = ModelRegistry::available_models();
    Json(json!({
        "models": models
    }))
}

// List downloaded models
pub async fn list_downloaded_models(
    State(state): State<Arc<RouterState>>,
) -> Json<serde_json::Value> {
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

// Transcribe request
#[derive(Deserialize)]
pub struct TranscribeRequest {
    pub audio: String,
    pub language: Option<String>,
    pub sample_rate: Option<u32>,
    pub translate: Option<bool>,
}

#[derive(Serialize)]
pub struct TranscribeResponse {
    pub text: String,
    pub language: Option<String>,
    pub duration: f32,
    pub segments: Option<Vec<SegmentResponse>>,
}

#[derive(Serialize)]
pub struct SegmentResponse {
    pub text: String,
    pub start: f32,
    pub end: f32,
}

pub async fn transcribe(
    State(state): State<Arc<RouterState>>,
    Json(req): Json<TranscribeRequest>,
) -> Result<Json<TranscribeResponse>, AppError> {
    let audio_bytes = BASE64
        .decode(&req.audio)
        .map_err(|e| AppError::bad_request(format!("Invalid base64 audio: {}", e)))?;

    let samples: Vec<f32> = audio_bytes
        .chunks(2)
        .filter_map(|chunk: &[u8]| {
            if chunk.len() == 2 {
                let s = i16::from_le_bytes([chunk[0], chunk[1]]);
                Some(s as f32 / i16::MAX as f32)
            } else {
                None
            }
        })
        .collect();

    if samples.is_empty() {
        return Err(AppError::bad_request("No audio samples found".to_string()));
    }

    let sample_rate = req.sample_rate.unwrap_or(16000);
    let final_samples = if sample_rate != 16000 {
        resample_audio(&samples, sample_rate, 16000)?
    } else {
        samples
    };

    state
        .load_transcriber()
        .map_err(|e| AppError::internal(format!("Failed to load transcriber: {}", e)))?;

    let result = state
        .transcribe(
            &final_samples,
            req.language.as_deref(),
            req.translate.unwrap_or(false),
        )
        .map_err(|e| AppError::internal(format!("Transcription failed: {}", e)))?;

    let segments = result.segments.map(|segs| {
        segs.into_iter()
            .map(|s| SegmentResponse {
                text: s.text,
                start: s.start,
                end: s.end,
            })
            .collect()
    });

    Ok(Json(TranscribeResponse {
        text: result.text,
        language: result.language,
        duration: result.duration,
        segments,
    }))
}

// SSE streaming transcription - accepts streaming base64 audio data
//
// Request body: base64-encoded PCM audio (little-endian 16-bit signed, mono)
// The audio is accumulated and transcribed in chunks as data is received.
//
// Query parameters:
//   - sample_rate: Sample rate of the audio (default: 16000)
//   - language: Language hint (optional)
//   - translate: Whether to translate to English (default: false)
pub async fn transcribe_stream(
    State(state): State<Arc<RouterState>>,
    _headers: HeaderMap,
    Query(params): Query<StreamParams>,
    body: axum::body::Body,
) -> Result<Sse<impl Stream<Item = Result<SseEvent, std::io::Error>>>, AppError> {
    // Load transcriber first
    state
        .load_transcriber()
        .map_err(|e| AppError::internal(format!("Failed to load transcriber: {}", e)))?;

    // Create broadcast channel for SSE events
    let (tx, rx) = broadcast::channel::<SseEventData>(100);

    // Send initial event
    let _ = tx.send(SseEventData::Transcript {
        text: "Receiving audio...".to_string(),
        partial: true,
    });

    // Get parameters from query or headers
    let sample_rate = params.sample_rate.unwrap_or(16000);
    let language = params.language.clone();
    let translate = params.translate.unwrap_or(false);

    // Spawn async task to process streaming audio
    tokio::spawn(async move {
        if let Err(e) =
            process_streaming_audio(tx, body, sample_rate, language, translate, state).await
        {
            tracing::error!("Streaming transcription error: {}", e);
        }
    });

    // Create SSE stream from broadcast channel
    let stream = BroadcastStream::new(rx)
        .map_ok(|event| event.to_sse_event("transcript"))
        .map_err(|e| std::io::Error::other(e.to_string()));

    use axum::response::sse::KeepAlive;
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// Parameters for streaming transcription
#[derive(Debug, Deserialize, Default)]
pub struct StreamParams {
    pub sample_rate: Option<u32>,
    pub language: Option<String>,
    pub translate: Option<bool>,
}

// Number of samples to accumulate before triggering partial transcription (~1 second at 16kHz)
const PARTIAL_TRANSCRIBE_SAMPLES: usize = 16000;

// Process streaming audio data - sends partial results as audio accumulates
async fn process_streaming_audio(
    tx: broadcast::Sender<SseEventData>,
    body: axum::body::Body,
    sample_rate: u32,
    language: Option<String>,
    translate: bool,
    state: Arc<RouterState>,
) -> Result<(), String> {
    // Buffer for accumulating base64 data
    let mut base64_buffer = Vec::new();
    // Buffer for decoded audio samples (accumulates ALL received data)
    let mut audio_buffer: Vec<f32> = Vec::new();
    // Track if we need to trigger partial transcription
    let mut last_triggered_samples = 0usize;

    // Ensure transcriber is loaded
    if let Err(e) = state.load_transcriber() {
        let _ = tx.send(SseEventData::Error {
            message: format!("Failed to load transcriber: {}", e),
        });
        return Ok(());
    }

    // Convert body to data stream
    let mut body_stream = body.into_data_stream();

    while let Some(chunk_result) = body_stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Body read error: {}", e))?;

        // Extend base64 buffer
        base64_buffer.extend_from_slice(&chunk);

        // Decode current chunk (filter out invalid base64 chars like newlines)
        let clean_b64: String = base64_buffer
            .iter()
            .filter(|&&c| c != b'\n' && c != b'\r' && c != b' ')
            .map(|&c| c as char)
            .collect();

        // Try to decode
        if let Ok(decoded) = BASE64.decode(&clean_b64) {
            // Decode audio from base64 bytes
            for chunk_bytes in decoded.chunks(2) {
                if chunk_bytes.len() == 2 {
                    let s = i16::from_le_bytes([chunk_bytes[0], chunk_bytes[1]]);
                    audio_buffer.push(s as f32 / i16::MAX as f32);
                }
            }

            // Check if we should trigger partial transcription
            // Only trigger if we've received enough NEW samples since last trigger
            let new_samples = audio_buffer.len() - last_triggered_samples;
            if new_samples >= PARTIAL_TRANSCRIBE_SAMPLES {
                let current_samples = audio_buffer.len();
                let audio_duration = current_samples as f32 / 16000.0;

                tracing::info!(
                    "Partial transcription: {} samples ({:.1}s), triggering...",
                    current_samples,
                    audio_duration
                );

                // Resample if needed
                let samples = if sample_rate != 16000 {
                    match resample_audio(&audio_buffer, sample_rate, 16000) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Resample error: {}", e);
                            continue;
                        }
                    }
                } else {
                    audio_buffer.clone()
                };

                // Send partial result
                let _ = tx.send(SseEventData::Transcript {
                    text: format!("[Partial {:.1}s] Transcribing...", audio_duration),
                    partial: true,
                });

                // Trigger transcription in background
                let tx_clone = tx.clone();
                let state_clone = state.clone();
                let lang_clone = language.clone();

                tokio::spawn(async move {
                    let result = state_clone.transcribe(&samples, lang_clone.as_deref(), false);
                    match result {
                        Ok(result) => {
                            let _ = tx_clone.send(SseEventData::Transcript {
                                text: result.text,
                                partial: true, // partial, more may come
                            });
                        }
                        Err(e) => {
                            tracing::error!("Partial transcription error: {}", e);
                        }
                    }
                });

                // Update last triggered position
                last_triggered_samples = current_samples;

                // Clear base64 buffer (we've already decoded it)
                base64_buffer.clear();
            }
        }
    }

    // Final transcription with ALL accumulated audio
    if !audio_buffer.is_empty() {
        let audio_duration = audio_buffer.len() as f32 / 16000.0;
        tracing::info!(
            "Final transcription: {} samples ({:.1}s)",
            audio_buffer.len(),
            audio_duration
        );

        let _ = tx.send(SseEventData::Transcript {
            text: format!("[Final] Transcribing {:.1}s audio...", audio_duration),
            partial: true,
        });

        let samples = if sample_rate != 16000 {
            match resample_audio(&audio_buffer, sample_rate, 16000) {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(SseEventData::Error {
                        message: format!("Resample error: {}", e),
                    });
                    return Ok(());
                }
            }
        } else {
            audio_buffer
        };

        let result = state.transcribe(&samples, language.as_deref(), translate);

        match result {
            Ok(result) => {
                let event = SseEventData::Transcript {
                    text: result.text,
                    partial: false, // final result
                };
                if let Err(e) = tx.send(event) {
                    tracing::error!("Failed to send final result: {}", e);
                }
            }
            Err(e) => {
                let _ = tx.send(SseEventData::Error {
                    message: format!("Transcription error: {}", e),
                });
            }
        }
    } else {
        let _ = tx.send(SseEventData::Error {
            message: "No audio data received".to_string(),
        });
    }

    Ok(())
}

// Helper function to resample audio
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
        let src_idx_ceil = (src_idx.ceil() as usize).min(samples.len() - 1);
        let frac = src_idx - src_idx.floor();

        let sample = samples[src_idx_floor] * (1.0 - frac) + samples[src_idx_ceil] * frac;
        resampled.push(sample);
    }

    Ok(resampled)
}

// Error type for handlers
pub struct AppError {
    message: String,
    status: StatusCode,
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppError")
            .field("message", &self.message)
            .field("status", &self.status)
            .finish()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}

impl AppError {
    pub fn new(status: StatusCode, message: String) -> Self {
        Self { message, status }
    }

    pub fn bad_request(message: String) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn internal(message: String) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(err: base64::DecodeError) -> Self {
        Self::bad_request(format!("Base64 decode error: {}", err))
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        Self::internal(err)
    }
}
