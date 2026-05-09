// download command - download a model

use crate::models::registry::ModelRegistry;
use anyhow::Result;
use futures_util::StreamExt;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use tar::Archive;

pub async fn run(model_id: &str) -> Result<()> {
    println!("=== Downloading model: {} ===\n", model_id);

    // Find model
    let model = ModelRegistry::get(model_id)
        .ok_or_else(|| anyhow::anyhow!("Model '{}' not found. Run `handy-cli list-models` to see available models.", model_id))?;

    let url = model.url
        .ok_or_else(|| anyhow::anyhow!("Model '{}' has no download URL", model_id))?;

    println!("Downloading {} from {}", model.name, url);
    println!("Size: {} MB", model.size_mb);

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("handy-cli")
        .join("models");

    // Create cache directory
    std::fs::create_dir_all(&cache_dir)?;

    let dest_dir = cache_dir.join(model_id);
    let dest_file = cache_dir.join(format!("{}.tar.gz", model_id));

    // Download with progress
    println!("\nDownloading...");
    let response = reqwest::get(&url).await?;
    let total_size = response.content_length().unwrap_or(0);

    let mut file = std::fs::File::create(&dest_file)?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        std::io::Write::write_all(&mut file, &chunk)?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let percentage = (downloaded as f64 / total_size as f64) * 100.0;
            print!("\rProgress: {:.1}% ({:.1} MB / {:.1} MB)", 
                   percentage, 
                   downloaded as f64 / 1_000_000.0, 
                   total_size as f64 / 1_000_000.0);
        }
    }
    println!("\n\nDownload complete!");

    // Extract tar.gz
    println!("Extracting to {:?}...", dest_dir);
    
    // Remove existing directory if it exists
    if dest_dir.exists() {
        std::fs::remove_dir_all(&dest_dir)?;
    }
    std::fs::create_dir_all(&dest_dir)?;

    let tar_gz = std::fs::File::open(&dest_file)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&dest_dir)?;

    // Remove the tar.gz file
    std::fs::remove_file(&dest_file)?;

    println!("\n✓ Model '{}' downloaded and extracted successfully!", model_id);
    println!("  Model path: {:?}", dest_dir);

    Ok(())
}
