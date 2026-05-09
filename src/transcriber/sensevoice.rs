// SenseVoice transcription engine using transcribe-rs

use super::{EngineType, TranscriptionResult, Transcriber};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::sense_voice::{SenseVoiceModel, SenseVoiceParams};
use transcribe_rs::onnx::Quantization;

pub struct SenseVoiceTranscriber {
    engine: SenseVoiceModel,
}

impl SenseVoiceTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self> {
        // model_dir should contain the model files
        let engine = SenseVoiceModel::load(model_dir, &Quantization::Int8)?;
        Ok(Self { engine })
    }
}

impl Transcriber for SenseVoiceTranscriber {
    fn transcribe(&mut self, audio: &[f32], language: Option<&str>) -> Result<TranscriptionResult> {
        let lang = match language.unwrap_or("auto") {
            "zh" | "zh-Hans" | "zh-Hant" => Some("zh".to_string()),
            "en" => Some("en".to_string()),
            "ja" => Some("ja".to_string()),
            "ko" => Some("ko".to_string()),
            "yue" => Some("yue".to_string()),
            _ => None,
        };

        let params = SenseVoiceParams {
            language: lang,
            use_itn: Some(true),
        };

        let result = self.engine.transcribe_with(audio, &params)?;

        Ok(TranscriptionResult {
            text: result.text,
            language: result.language,
            duration: result.duration.unwrap_or(0.0),
            language_probability: None,
        })
    }

    fn engine_type(&self) -> EngineType {
        EngineType::SenseVoice
    }
}
