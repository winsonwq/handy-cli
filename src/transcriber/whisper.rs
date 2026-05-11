// Whisper transcription engine using transcribe-rs

use super::{TranscriptionResult, Transcriber};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::whisper_cpp::{WhisperEngine, WhisperInferenceParams};

pub struct WhisperTranscriber {
    engine: WhisperEngine,
}

impl WhisperTranscriber {
    pub fn new(model_path: &Path) -> Result<Self> {
        let engine = WhisperEngine::load(model_path)?;
        Ok(Self { engine })
    }
}

impl Transcriber for WhisperTranscriber {
    fn transcribe(&mut self, audio: &[f32], language: Option<&str>, translate: bool) -> Result<TranscriptionResult> {
        let lang = language.map(|l| {
            if l == "zh-Hans" || l == "zh-Hant" {
                "zh".to_string()
            } else {
                l.to_string()
            }
        });

        let params = WhisperInferenceParams {
            language: lang,
            translate,
            ..Default::default()
        };

        let result = self.engine.transcribe_with(audio, &params)?;

        Ok(TranscriptionResult {
            text: result.text,
            language: None,
            duration: 0.0,
            segments: None,
            language_probability: None,
        })
    }

}
