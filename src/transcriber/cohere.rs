// Cohere transcription engine using transcribe-rs
// Large multilingual model with high accuracy

use super::{TranscriptionResult, TranscriptionSegment, Transcriber};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::cohere::{CohereModel, CohereParams};
use transcribe_rs::onnx::Quantization;

pub struct CohereTranscriber {
    engine: CohereModel,
}

impl CohereTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let engine = CohereModel::load(model_dir, &Quantization::Int8)?;
        Ok(Self { engine })
    }
}

impl Transcriber for CohereTranscriber {
    fn transcribe(&mut self, audio: &[f32], language: Option<&str>, translate: bool) -> Result<TranscriptionResult> {
        let lang = language.map(|l| {
            if l == "zh-Hans" || l == "zh-Hant" {
                "zh".to_string()
            } else {
                l.to_string()
            }
        });

        let params = CohereParams {
            language: lang,
            translate,
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
