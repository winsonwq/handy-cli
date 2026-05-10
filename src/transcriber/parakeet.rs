// Parakeet transcription engine using transcribe-rs
// Supports Parakeet V2 and V3 models for fast English/European language transcription

use super::{EngineType, TranscriptionResult, TranscriptionSegment, Transcriber};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::parakeet::{ParakeetModel, ParakeetParams};
use transcribe_rs::onnx::Quantization;

pub struct ParakeetTranscriber {
    engine: ParakeetModel,
}

impl ParakeetTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let engine = ParakeetModel::load(model_dir, &Quantization::Int8)?;
        Ok(Self { engine })
    }
}

impl Transcriber for ParakeetTranscriber {
    fn transcribe(&mut self, audio: &[f32], _language: Option<&str>, _translate: bool) -> Result<TranscriptionResult> {
        // Parakeet doesn't support language selection - it auto-detects
        let params = ParakeetParams::default();
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

    fn engine_type(&self) -> EngineType {
        EngineType::Parakeet
    }
}
