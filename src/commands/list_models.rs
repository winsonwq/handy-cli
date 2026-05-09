// list_models command - list available models

use crate::models::ModelManager;
use anyhow::Result;
use std::path::PathBuf;

pub async fn run(engine_filter: Option<&str>) -> Result<()> {
    println!("=== Available Models ===\n");

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("handy-cli")
        .join("models");

    let download_url = "https://blob.handy.computer".to_string();

    let manager = ModelManager::new(cache_dir, download_url);

    let models = if let Some(engine) = engine_filter {
        manager.get_models_by_engine(engine)
    } else {
        manager.get_available_models()
    };

    // Group by engine type
    let mut current_engine = String::new();

    for model in &models {
        let engine_str = match model.engine_type {
            crate::models::registry::EngineType::Whisper => "Whisper",
            crate::models::registry::EngineType::SenseVoice => "SenseVoice",
            crate::models::registry::EngineType::Parakeet => "Parakeet",
            crate::models::registry::EngineType::Moonshine => "Moonshine",
            crate::models::registry::EngineType::MoonshineStreaming => "Moonshine (Streaming)",
            crate::models::registry::EngineType::GigaAM => "GigaAM",
            crate::models::registry::EngineType::Canary => "Canary",
            crate::models::registry::EngineType::Cohere => "Cohere",
        };

        if engine_str != current_engine {
            if !current_engine.is_empty() {
                println!();
            }
            current_engine = engine_str.to_string();
            println!("--- {} ---", engine_str);
        }

        let status = if model.is_downloaded {
            "[downloaded]"
        } else {
            ""
        };

        let recommended = if model.is_recommended { " ★" } else { "" };

        println!(
            "  {}{}{}",
            model.name,
            recommended,
            status
        );
        println!("    ID: {}", model.id);
        println!("    Size: {} MB", model.size_mb);
        println!("    Accuracy: {:.0}%", model.accuracy_score * 100.0);
        println!("    Speed: {:.0}%", model.speed_score * 100.0);

        // Show supported languages
        if !model.supported_languages.is_empty() {
            let lang_count = model.supported_languages.len();
            if lang_count <= 10 {
                println!("    Languages: {}", model.supported_languages.join(", "));
            } else {
                println!("    Languages: {} languages", lang_count);
            }
        }

        if model.supports_translation {
            println!("    + Translation to English");
        }

        println!("    {}", model.description);
        println!();
    }

    // Print download status summary
    let downloaded_count = models.iter().filter(|m| m.is_downloaded).count();
    let total_count = models.len();
    println!("({} of {} models downloaded)", downloaded_count, total_count);

    Ok(())
}
