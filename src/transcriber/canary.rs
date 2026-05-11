// Canary transcription engine using transcribe-rs
// Supports translation mode for English, German, Spanish, French

use super::{TranscriptionResult, TranscriptionSegment, Transcriber};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::canary::{CanaryModel, CanaryParams};
use transcribe_rs::onnx::Quantization;

pub struct CanaryTranscriber {
    engine: CanaryModel,
}

impl CanaryTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let engine = CanaryModel::load(model_dir, &Quantization::Int8)?;
        Ok(Self { engine })
    }
}

impl Transcriber for CanaryTranscriber {
    fn transcribe(&mut self, audio: &[f32], language: Option<&str>, translate: bool) -> Result<TranscriptionResult> {
        let lang = language.map(|l| {
            if l == "zh-Hans" || l == "zh-Hant" {
                "zh".to_string()
            } else {
                l.to_string()
            }
        });

        // For Canary, translation is done by setting target_language
        // When translate is true, we set target_language to "en" (English)
        // When translate is false, target_language defaults to source language
        let target_lang = if translate {
            Some("en".to_string())
        } else {
            None
        };

        let params = CanaryParams {
            language: lang,
            target_language: target_lang,
            ..Default::default()
        };

        let result = self.engine.transcribe_with(audio, &params)?;

        let segments = result.segments.map(|segs| {
            segs
                .into_iter()
                .map(|s| TranscriptionSegment {
                    text: s.text,
                    start: s.start,
                    end: s.end,
                })
                .collect()
        });

        Ok(TranscriptionResult {
            text: result.text,
            language: None,
            duration: 0.0,
            segments,
            language_probability: None,
        })
    }

}
