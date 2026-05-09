// Transcription module - handles ASR engines
//
// Uses transcribe-rs for actual transcription.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod sensevoice;

pub use sensevoice::SenseVoiceTranscriber;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub segments: Option<Vec<TranscriptionSegment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    Whisper,
    SenseVoice,
}

impl TryFrom<&str> for EngineType {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "whisper" => Ok(EngineType::Whisper),
            "sensevoice" => Ok(EngineType::SenseVoice),
            _ => Err(anyhow::anyhow!("Unknown engine: {}", s)),
        }
    }
}

/// Trait for transcription engines
pub trait Transcriber {
    fn transcribe(&mut self, audio: &[f32], language: Option<&str>) -> Result<TranscriptionResult>;
    fn engine_type(&self) -> EngineType;
}
