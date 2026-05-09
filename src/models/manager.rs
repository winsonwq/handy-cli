// Model manager - handles downloading and managing models

use anyhow::Result;
use futures_util::StreamExt;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub struct ModelManager {
    cache_dir: PathBuf,
    download_url: String,
}

impl ModelManager {
    pub fn new(cache_dir: PathBuf, download_url: String) -> Self {
        Self {
            cache_dir,
            download_url,
        }
    }

    /// Get model path if downloaded
    pub fn model_path(&self, model_id: &str) -> Option<PathBuf> {
        let path = self.cache_dir.join(model_id);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Download a model
    pub async fn download(&self, model_id: &str, url: &str) -> Result<PathBuf> {
        let dest_dir = self.cache_dir.join(model_id);
        let dest_file = dest_dir.with_extension("tar.gz");

        // Create cache directory
        tokio::fs::create_dir_all(&self.cache_dir).await?;

        info!("Downloading model {} from {}", model_id, url);

        // Download with progress
        let response = reqwest::get(url).await?;
        let total_size = response.content_length().unwrap_or(0);

        let mut file = tokio::fs::File::create(&dest_file).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let percentage = (downloaded as f64 / total_size as f64) * 100.0;
                info!("Downloaded {} / {} ({:.1}%)", downloaded, total_size, percentage);
            }
        }

        info!("Extracting model to {:?}", dest_dir);

        // Extract tar.gz
        let tar_gz = std::fs::File::open(&dest_file)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&dest_dir)?;

        // Remove the tar.gz file
        tokio::fs::remove_file(&dest_file).await?;

        info!("Model {} downloaded and extracted to {:?}", model_id, dest_dir);
        Ok(dest_dir)
    }

    /// List downloaded models
    pub fn list_downloaded(&self) -> Vec<PathBuf> {
        if !self.cache_dir.exists() {
            return vec![];
        }

        std::fs::read_dir(&self.cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if model is downloaded
    pub fn is_downloaded(&self, model_id: &str) -> bool {
        self.model_path(model_id).is_some()
    }
}
