// Model registry - defines all available models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    Whisper,
    SenseVoice,
    Parakeet,
    Moonshine,
    GigaAM,
    Canary,
    Cohere,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub url: Option<String>,
    pub size_mb: u64,
    pub engine_type: EngineType,
    pub accuracy_score: f32,
    pub speed_score: f32,
    pub supported_languages: Vec<String>,
    pub is_recommended: bool,
}

pub struct ModelRegistry;

impl ModelRegistry {
    /// Get all available models
    pub fn available_models() -> Vec<ModelInfo> {
        vec![
            // SenseVoice models
            ModelInfo {
                id: "sense-voice-int8".to_string(),
                name: "SenseVoice (int8)".to_string(),
                description: "Optimized for Chinese, built-in punctuation".to_string(),
                filename: "sense-voice-int8".to_string(),
                url: Some("https://blob.handy.computer/sense-voice-int8.tar.gz".to_string()),
                size_mb: 232,
                engine_type: EngineType::SenseVoice,
                accuracy_score: 0.85,
                speed_score: 0.80,
                supported_languages: vec!["zh".to_string(), "en".to_string(), "ja".to_string(), "ko".to_string()],
                is_recommended: true,
            },
            // Whisper models
            ModelInfo {
                id: "tiny".to_string(),
                name: "Whisper Tiny".to_string(),
                description: "Fastest, lowest accuracy".to_string(),
                filename: "ggml-tiny.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-tiny.bin".to_string()),
                size_mb: 75,
                engine_type: EngineType::Whisper,
                accuracy_score: 0.50,
                speed_score: 0.95,
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                is_recommended: false,
            },
            ModelInfo {
                id: "base".to_string(),
                name: "Whisper Base".to_string(),
                description: "Good balance".to_string(),
                filename: "ggml-base.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-base.bin".to_string()),
                size_mb: 150,
                engine_type: EngineType::Whisper,
                accuracy_score: 0.65,
                speed_score: 0.85,
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                is_recommended: false,
            },
            ModelInfo {
                id: "small".to_string(),
                name: "Whisper Small".to_string(),
                description: "Recommended for most use cases".to_string(),
                filename: "ggml-small.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-small.bin".to_string()),
                size_mb: 465,
                engine_type: EngineType::Whisper,
                accuracy_score: 0.75,
                speed_score: 0.70,
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                is_recommended: true,
            },
            ModelInfo {
                id: "medium".to_string(),
                name: "Whisper Medium".to_string(),
                description: "Higher accuracy, slower".to_string(),
                filename: "ggml-medium.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-medium.bin".to_string()),
                size_mb: 1500,
                engine_type: EngineType::Whisper,
                accuracy_score: 0.85,
                speed_score: 0.50,
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                is_recommended: false,
            },
            ModelInfo {
                id: "large".to_string(),
                name: "Whisper Large".to_string(),
                description: "Highest accuracy, slowest".to_string(),
                filename: "ggml-large.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-large.bin".to_string()),
                size_mb: 3000,
                engine_type: EngineType::Whisper,
                accuracy_score: 0.95,
                speed_score: 0.30,
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                is_recommended: false,
            },
        ]
    }

    /// Get model by ID
    pub fn get(id: &str) -> Option<ModelInfo> {
        Self::available_models().into_iter().find(|m| m.id == id)
    }

    /// Get models by engine type
    pub fn by_engine(engine: EngineType) -> Vec<ModelInfo> {
        Self::available_models().into_iter().filter(|m| m.engine_type == engine).collect()
    }
}
