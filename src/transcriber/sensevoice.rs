// SenseVoice transcription engine using transcribe-rs

use super::{Transcriber, TranscriptionResult, TranscriptionSegment};
use anyhow::Result;
use std::path::Path;
use transcribe_rs::onnx::sense_voice::SenseVoiceModel;
use transcribe_rs::onnx::Quantization;
use transcribe_rs::{SpeechModel, TranscribeOptions};

pub struct SenseVoiceTranscriber {
    engine: SenseVoiceModel,
}

impl SenseVoiceTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let engine = SenseVoiceModel::load(model_dir, &Quantization::Int8)?;
        Ok(Self { engine })
    }
}

impl Transcriber for SenseVoiceTranscriber {
    fn transcribe(
        &mut self,
        audio: &[f32],
        language: Option<&str>,
        _translate: bool,
    ) -> Result<TranscriptionResult> {
        // Note: SenseVoice doesn't support translation, so we ignore the translate parameter
        let options = TranscribeOptions {
            language: language.map(|l| match l {
                "zh-Hans" | "zh-Hant" => "zh".to_string(),
                s => s.to_string(),
            }),
            translate: false,
            leading_silence_ms: None,
            trailing_silence_ms: None,
        };

        let result = self.engine.transcribe(audio, &options)?;

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
            language: None,
            duration: 0.0,
            segments,
            language_probability: None,
        })
    }
}
