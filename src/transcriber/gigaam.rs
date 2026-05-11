// GigaAM transcription engine using transcribe-rs
// Supports GigaAM v3 for Russian speech recognition

use super::{TranscriptionResult, TranscriptionSegment, Transcriber};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::gigaam::{GigaAMModel, GigaAMParams};
use transcribe_rs::onnx::Quantization;

pub struct GigaAMTranscriber {
    engine: GigaAMModel,
}

impl GigaAMTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let engine = GigaAMModel::load(model_dir, &Quantization::Int8)?;
        Ok(Self { engine })
    }
}

impl Transcriber for GigaAMTranscriber {
    fn transcribe(&mut self, audio: &[f32], _language: Option<&str>, _translate: bool) -> Result<TranscriptionResult> {
        // GigaAM is Russian-only
        let params = GigaAMParams::default();
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
            language: Some("ru".to_string()),
            duration: 0.0,
            segments,
            language_probability: None,
        })
    }

}
