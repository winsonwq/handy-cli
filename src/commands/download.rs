// download command - download a model

use crate::models::{DownloadProgress, ModelManager};
use anyhow::Result;

pub async fn run(model_id: &str) -> Result<()> {
    println!("=== Downloading model: {} ===\n", model_id);

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("handy-cli")
        .join("models");

    let download_url = "https://blob.handy.computer".to_string();

    let manager = ModelManager::new(cache_dir, download_url);

    // Check if model exists
    let model_info = manager.get_model_info(model_id);
    let model = match model_info {
        Some(m) => m,
        None => {
            anyhow::bail!("Model '{}' not found. Run `handy-cli list-models` to see available models.", model_id);
        }
    };

    if !model.url.is_some() {
        anyhow::bail!("Model '{}' has no download URL", model_id);
    }

    println!("Downloading {}...", model.name);
    println!("Size: {} MB", model.size_mb);
    println!();

    // Create progress channel
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<DownloadProgress>();

    // Spawn progress printer
    let progress_handle = tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            if progress.total > 0 {
                println!(
                    "\rProgress: {:.1}% ({:.1} MB / {:.1} MB)  ",
                    progress.percentage,
                    progress.downloaded as f64 / 1_000_000.0,
                    progress.total as f64 / 1_000_000.0,
                );
            } else {
                println!("\rDownloaded: {:.1} MB", progress.downloaded as f64 / 1_000_000.0);
            }
        }
    });

    // Download with progress
    manager.download_model(model_id, Some(tx)).await?;

    // Wait for progress to finish
    let _ = progress_handle.await;

    println!("\n\n✓ Model '{}' downloaded successfully!", model_id);
    println!("  Path: {:?}", manager.get_model_path(model_id));

    Ok(())
}
