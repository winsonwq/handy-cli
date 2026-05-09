// Transcription module - handles ASR engines
//
// Uses transcribe-rs for actual transcription.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod sensevoice;
pub mod whisper;

pub use sensevoice::SenseVoiceTranscriber;
pub use whisper::WhisperTranscriber;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration: f32,
    pub segments: Option<Vec<TranscriptionSegment>>,
    pub language_probability: Option<f32>,
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
    Parakeet,
    Moonshine,
    MoonshineStreaming,
    GigaAM,
    Canary,
    Cohere,
}

impl EngineType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "whisper" => Some(EngineType::Whisper),
            "sensevoice" => Some(EngineType::SenseVoice),
            "parakeet" => Some(EngineType::Parakeet),
            "moonshine" => Some(EngineType::Moonshine),
            "moonshine-streaming" | "moonshinestreaming" => Some(EngineType::MoonshineStreaming),
            "gigaam" | "giga_am" => Some(EngineType::GigaAM),
            "canary" => Some(EngineType::Canary),
            "cohere" => Some(EngineType::Cohere),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EngineType::Whisper => "whisper",
            EngineType::SenseVoice => "sensevoice",
            EngineType::Parakeet => "parakeet",
            EngineType::Moonshine => "moonshine",
            EngineType::MoonshineStreaming => "moonshine-streaming",
            EngineType::GigaAM => "gigaam",
            EngineType::Canary => "canary",
            EngineType::Cohere => "cohere",
        }
    }
}

impl TryFrom<&str> for EngineType {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        EngineType::from_str(s)
            .ok_or_else(|| anyhow::anyhow!("Unknown engine: {}", s))
    }
}

impl From<crate::models::registry::EngineType> for EngineType {
    fn from(e: crate::models::registry::EngineType) -> Self {
        match e {
            crate::models::registry::EngineType::Whisper => EngineType::Whisper,
            crate::models::registry::EngineType::SenseVoice => EngineType::SenseVoice,
            crate::models::registry::EngineType::Parakeet => EngineType::Parakeet,
            crate::models::registry::EngineType::Moonshine => EngineType::Moonshine,
            crate::models::registry::EngineType::MoonshineStreaming => EngineType::MoonshineStreaming,
            crate::models::registry::EngineType::GigaAM => EngineType::GigaAM,
            crate::models::registry::EngineType::Canary => EngineType::Canary,
            crate::models::registry::EngineType::Cohere => EngineType::Cohere,
        }
    }
}

/// Trait for transcription engines
pub trait Transcriber {
    fn transcribe(&mut self, audio: &[f32], language: Option<&str>) -> Result<TranscriptionResult>;
    fn engine_type(&self) -> EngineType;
}
