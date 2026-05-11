// Moonshine transcription engine using transcribe-rs
// Supports Moonshine Base and Tiny variants for ultra-fast English transcription

use super::{Transcriber, TranscriptionResult, TranscriptionSegment};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::moonshine::{MoonshineModel, MoonshineParams, MoonshineVariant};
use transcribe_rs::onnx::Quantization;

pub struct MoonshineTranscriber {
    engine: MoonshineModel,
}

impl MoonshineTranscriber {
    /// Create a Moonshine transcriber with specified variant
    pub fn new(model_dir: &Path, variant: MoonshineVariant) -> Result<Self> {
        let engine = MoonshineModel::load(model_dir, variant, &Quantization::Int8)?;
        Ok(Self { engine })
    }

    /// Auto-detect variant based on model directory name
    pub fn new_auto(model_dir: &Path) -> Result<Self> {
        let model_name = model_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let variant = if model_name.contains("base") {
            MoonshineVariant::Base
        } else {
            MoonshineVariant::Tiny
        };

        Self::new(model_dir, variant)
    }
}

impl Transcriber for MoonshineTranscriber {
    fn transcribe(
        &mut self,
        audio: &[f32],
        _language: Option<&str>,
        _translate: bool,
    ) -> Result<TranscriptionResult> {
        // Moonshine is English-only
        let params = MoonshineParams::default();
        let result = self.engine.transcribe_with(audio, &params)?;

        let segments = result.segments.map(|segs| {
            segs.into_iter()
                .map(|s| TranscriptionSegment {
                    text: s.text,
                    start: s.start,
                    end: s.end,
                })
                .collect()
        });

        Ok(TranscriptionResult {
            text: result.text,
            language: Some("en".to_string()),
            duration: 0.0,
            segments,
            language_probability: None,
        })
    }
}
