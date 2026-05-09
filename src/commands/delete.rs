// delete command - delete a downloaded model

use crate::models::ModelManager;
use anyhow::Result;
use std::path::PathBuf;

pub async fn run(model_id: &str) -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("handy-cli")
        .join("models");

    let download_url = "https://blob.handy.computer".to_string();

    let manager = ModelManager::new(cache_dir, download_url);

    // Check if model exists
    let model_info = manager.get_model_info(model_id);
    let model = match model_info {
        Some(m) => m,
        None => {
            anyhow::bail!("Model '{}' not found.", model_id);
        }
    };

    // Check if model is downloaded
    if !model.is_downloaded {
        if model.is_custom {
            anyhow::bail!("Model '{}' is not downloaded.", model_id);
        } else {
            anyhow::bail!(
                "Model '{}' is not downloaded. Custom models cannot be re-downloaded.",
                model_id
            );
        }
    }

    println!("=== Delete Model: {} ===\n", model.name);
    println!("Model ID: {}", model_id);
    println!("Path: {:?}", manager.get_model_path(model_id));
    println!();

    // Delete the model
    manager.delete_model(model_id)?;

    println!("✓ Model '{}' deleted successfully!", model_id);

    Ok(())
}
