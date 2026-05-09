// Model manager - handles downloading and managing models
//
// Features:
// - SHA256 verification
// - Resume interrupted downloads
// - Progress events via channels
// - Custom model auto-discovery
// - Model deletion

use anyhow::{Context, Result};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tar::Archive;
use flate2::read::GzDecoder;
use tokio::sync::mpsc;

pub use crate::models::registry::{DownloadProgress, ModelInfo};
use crate::models::registry::ModelRegistry;

pub type ProgressSender = mpsc::UnboundedSender<DownloadProgress>;
pub type ProgressReceiver = mpsc::UnboundedReceiver<DownloadProgress>;

/// RAII guard that cleans up download state when dropped
struct DownloadCleanup<'a> {
    models: &'a Mutex<HashMap<String, ModelInfo>>,
    cancel_flags: &'a Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    model_id: String,
    disarmed: bool,
}

impl<'a> Drop for DownloadCleanup<'a> {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }
        {
            let mut models = self.models.lock().unwrap();
            if let Some(model) = models.get_mut(&self.model_id) {
                model.is_downloading = false;
            }
        }
        self.cancel_flags.lock().unwrap().remove(&self.model_id);
    }
}

pub struct ModelManager {
    cache_dir: PathBuf,
    download_url_base: String,
    models: Mutex<HashMap<String, ModelInfo>>,
    cancel_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    extracting_models: Mutex<Vec<String>>,
}

impl ModelManager {
    pub fn new(cache_dir: PathBuf, download_url_base: String) -> Self {
        // Initialize models from registry
        let mut models = HashMap::new();
        for model in ModelRegistry::available_models() {
            models.insert(model.id.clone(), model);
        }

        let manager = Self {
            cache_dir,
            download_url_base,
            models: Mutex::new(models),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
            extracting_models: Mutex::new(Vec::new()),
        };

        // Discover custom models and update download status
        manager.discover_custom_models();
        manager.update_download_status();

        manager
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Get all available models (with updated download status)
    pub fn get_available_models(&self) -> Vec<ModelInfo> {
        let models = self.models.lock().unwrap();
        models.values().cloned().collect()
    }

    /// Get model info by ID
    pub fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let models = self.models.lock().unwrap();
        models.get(model_id).cloned()
    }

    /// Get model path if downloaded
    pub fn get_model_path(&self, model_id: &str) -> Option<PathBuf> {
        let model = self.get_model_info(model_id)?;
        if !model.is_downloaded {
            return None;
        }

        let model_path = self.cache_dir.join(&model.filename);
        let partial_path = self.cache_dir.join(format!("{}.partial", model.filename));

        if model.is_directory {
            if model_path.exists() && model_path.is_dir() && !partial_path.exists() {
                Some(model_path)
            } else {
                None
            }
        } else {
            if model_path.exists() && !partial_path.exists() {
                Some(model_path)
            } else {
                None
            }
        }
    }

    /// Update download status for all models
    fn update_download_status(&self) {
        let mut models = self.models.lock().unwrap();

        for model in models.values_mut() {
            if model.is_directory {
                let model_path = self.cache_dir.join(&model.filename);
                let partial_path = self.cache_dir.join(format!("{}.partial", model.filename));

                // Clean up interrupted extractions
                let extracting_path = self.cache_dir.join(format!("{}.extracting", model.filename));
                let is_extracting = {
                    let extracting = self.extracting_models.lock().unwrap();
                    extracting.contains(&model.id)
                };
                if extracting_path.exists() && !is_extracting {
                    let _ = std::fs::remove_dir_all(&extracting_path);
                }

                model.is_downloaded = model_path.exists() && model_path.is_dir();
                model.is_downloading = false;

                if partial_path.exists() {
                    model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
                } else {
                    model.partial_size = 0;
                }
            } else {
                let model_path = self.cache_dir.join(&model.filename);
                let partial_path = self.cache_dir.join(format!("{}.partial", model.filename));

                model.is_downloaded = model_path.exists();
                model.is_downloading = false;

                if partial_path.exists() {
                    model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
                } else {
                    model.partial_size = 0;
                }
            }
        }
    }

    /// Discover custom Whisper models (.bin files) in the models directory
    fn discover_custom_models(&self) {
        if !self.cache_dir.exists() {
            return;
        }

        let predefined_filenames: Vec<String> = ModelRegistry::predefined_filenames();

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                let filename = match path.file_name().and_then(|s| s.to_str()) {
                    Some(name) => name.to_string(),
                    None => continue,
                };

                // Skip hidden files
                if filename.starts_with('.') {
                    continue;
                }

                // Only process .bin files
                if !filename.ends_with(".bin") {
                    continue;
                }

                // Skip predefined models
                if predefined_filenames.contains(&filename) {
                    continue;
                }

                let model_id = filename.trim_end_matches(".bin").to_string();

                // Skip if already exists
                {
                    let models = self.models.lock().unwrap();
                    if models.contains_key(&model_id) {
                        continue;
                    }
                }

                // Generate display name
                let display_name = model_id
                    .replace(['-', '_'], " ")
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let size_mb = path.metadata()
                    .map(|m| m.len() / (1024 * 1024))
                    .unwrap_or(0);

                let mut models = self.models.lock().unwrap();
                models.insert(
                    model_id.clone(),
                    ModelInfo {
                        id: model_id,
                        name: display_name,
                        description: "Custom model".to_string(),
                        filename,
                        url: None,
                        sha256: None,
                        size_mb,
                        is_downloaded: true,
                        is_downloading: false,
                        partial_size: 0,
                        is_directory: false,
                        engine_type: crate::models::registry::EngineType::Whisper,
                        accuracy_score: 0.0,
                        speed_score: 0.0,
                        supports_translation: false,
                        is_recommended: false,
                        supported_languages: vec![],
                        supports_language_selection: true,
                        is_custom: true,
                    },
                );
            }
        }
    }

    /// Verify SHA256 of a file
    fn verify_sha256(path: &PathBuf, expected: Option<&str>, model_id: &str) -> Result<()> {
        let Some(expected) = expected else {
            return Ok(());
        };

        let actual = Self::compute_sha256(path)
            .with_context(|| format!("Failed to compute SHA256 for {}", model_id))?;

        if actual == expected {
            tracing::info!("SHA256 verified for model {}", model_id);
            Ok(())
        } else {
            let _ = std::fs::remove_file(path);
            anyhow::bail!(
                "SHA256 mismatch for model {}: expected {}, got {}",
                model_id, expected, actual
            );
        }
    }

    /// Compute SHA256 of a file
    fn compute_sha256(path: &PathBuf) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Download a model with progress reporting
    pub async fn download_model(
        &self,
        model_id: &str,
        progress_tx: Option<ProgressSender>,
    ) -> Result<()> {
        let model_info = {
            let models = self.models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info = model_info
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        let url = model_info.url
            .ok_or_else(|| anyhow::anyhow!("No download URL for model: {}", model_id))?;

        let model_path = self.cache_dir.join(&model_info.filename);
        let partial_path = self.cache_dir.join(format!("{}.partial", model_info.filename));

        // Create cache directory
        std::fs::create_dir_all(&self.cache_dir)?;

        // Check if already complete
        if model_path.exists() {
            if partial_path.exists() {
                let _ = std::fs::remove_file(&partial_path);
            }
            self.update_download_status();
            return Ok(());
        }

        // Determine resume position
        let mut resume_from = if partial_path.exists() {
            let size = partial_path.metadata()?.len();
            tracing::info!("Resuming download of {} from byte {}", model_id, size);
            size
        } else {
            0
        };

        // Mark as downloading
        {
            let mut models = self.models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = true;
            }
        }

        // Create cancellation flag
        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut flags = self.cancel_flags.lock().unwrap();
            flags.insert(model_id.to_string(), cancel_flag.clone());
        }

        let mut cleanup = DownloadCleanup {
            models: &self.models,
            cancel_flags: &self.cancel_flags,
            model_id: model_id.to_string(),
            disarmed: false,
        };

        // Build HTTP client
        let client = reqwest::Client::new();
        let mut request = client.get(&url);

        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let mut response = request.send().await?;

        // Handle server not supporting range requests
        if resume_from > 0 && response.status() == reqwest::StatusCode::OK {
            tracing::warn!("Server doesn't support range requests, restarting download");
            drop(response);
            let _ = std::fs::remove_file(&partial_path);
            resume_from = 0;
            response = client.get(&url).send().await?;
        }

        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(anyhow::anyhow!("Download failed: HTTP {}", response.status()));
        }

        let total_size = if resume_from > 0 {
            resume_from + response.content_length().unwrap_or(0)
        } else {
            response.content_length().unwrap_or(0)
        };

        let mut downloaded = resume_from;
        let mut stream = response.bytes_stream();

        // Open file
        let mut file = if resume_from > 0 {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&partial_path)?
        } else {
            std::fs::File::create(&partial_path)?
        };

        // Send initial progress
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(DownloadProgress {
                model_id: model_id.to_string(),
                downloaded,
                total: total_size,
                percentage: if total_size > 0 {
                    (downloaded as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                },
            });
        }

        let throttle_duration = Duration::from_millis(100);
        let mut last_emit = Instant::now();

        // Download loop
        while let Some(chunk) = stream.next().await {
            if cancel_flag.load(Ordering::Relaxed) {
                tracing::info!("Download cancelled: {}", model_id);
                return Ok(());
            }

            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if last_emit.elapsed() >= throttle_duration {
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(DownloadProgress {
                        model_id: model_id.to_string(),
                        downloaded,
                        total: total_size,
                        percentage: if total_size > 0 {
                            (downloaded as f64 / total_size as f64) * 100.0
                        } else {
                            0.0
                        },
                    });
                }
                last_emit = Instant::now();
            }
        }

        file.flush()?;
        drop(file);

        // Verify size
        if total_size > 0 {
            let actual_size = partial_path.metadata()?.len();
            if actual_size != total_size {
                let _ = std::fs::remove_file(&partial_path);
                anyhow::bail!(
                    "Download incomplete: expected {} bytes, got {}",
                    total_size, actual_size
                );
            }
        }

        // Verify SHA256
        tracing::info!("Verifying SHA256 for {}...", model_id);
        let verify_path = partial_path.clone();
        let verify_expected = model_info.sha256.clone();
        let verify_model_id = model_id.to_string();

        tokio::task::spawn_blocking(move || {
            Self::verify_sha256(&verify_path, verify_expected.as_deref(), &verify_model_id)
        })
        .await??
        ;

        // Extract if directory-based model
        if model_info.is_directory {
            // Mark as extracting
            {
                let mut extracting = self.extracting_models.lock().unwrap();
                extracting.push(model_id.to_string());
            }

            let temp_extract_dir = self.cache_dir.join(format!("{}.extracting", model_info.filename));
            let final_model_dir = self.cache_dir.join(&model_info.filename);

            if temp_extract_dir.exists() {
                let _ = std::fs::remove_dir_all(&temp_extract_dir);
            }
            std::fs::create_dir_all(&temp_extract_dir)?;

            // Extract tar.gz
            let tar_gz = File::open(&partial_path)?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);

            archive.unpack(&temp_extract_dir)
                .map_err(|e| {
                    let _ = std::fs::remove_dir_all(&temp_extract_dir);
                    let _ = std::fs::remove_file(&partial_path);
                    e
                })?;

            // Find extracted directory
            let extracted_dirs: Vec<_> = std::fs::read_dir(&temp_extract_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                .collect();

            if extracted_dirs.len() == 1 {
                let source_dir = extracted_dirs[0].path();
                if final_model_dir.exists() {
                    std::fs::remove_dir_all(&final_model_dir)?;
                }
                std::fs::rename(&source_dir, &final_model_dir)?;
                let _ = std::fs::remove_dir_all(&temp_extract_dir);
            } else {
                if final_model_dir.exists() {
                    std::fs::remove_dir_all(&final_model_dir)?;
                }
                std::fs::rename(&temp_extract_dir, &final_model_dir)?;
            }

            // Remove from extracting
            {
                let mut extracting = self.extracting_models.lock().unwrap();
                extracting.retain(|id| id != model_id);
            }

            // Remove downloaded tar.gz
            let _ = std::fs::remove_file(&partial_path);
        } else {
            // Move partial to final
            std::fs::rename(&partial_path, &model_path)?;
        }

        // Update state
        cleanup.disarmed = true;
        {
            let mut models = self.models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = false;
                model.is_downloaded = true;
                model.partial_size = 0;
            }
        }
        self.cancel_flags.lock().unwrap().remove(model_id);

        // Send completion
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(DownloadProgress {
                model_id: model_id.to_string(),
                downloaded,
                total: total_size,
                percentage: 100.0,
            });
        }

        tracing::info!("Successfully downloaded model {} to {:?}", model_id, model_path);
        Ok(())
    }

    /// Cancel an ongoing download
    pub fn cancel_download(&self, model_id: &str) -> Result<()> {
        let flags = self.cancel_flags.lock().unwrap();
        if let Some(flag) = flags.get(model_id) {
            flag.store(true, Ordering::Relaxed);
            tracing::info!("Cancellation requested for: {}", model_id);
        }

        let mut models = self.models.lock().unwrap();
        if let Some(model) = models.get_mut(model_id) {
            model.is_downloading = false;
        }

        Ok(())
    }

    /// Delete a model
    pub fn delete_model(&self, model_id: &str) -> Result<()> {
        let model_info = {
            let models = self.models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info = model_info
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        let model_path = self.cache_dir.join(&model_info.filename);
        let partial_path = self.cache_dir.join(format!("{}.partial", model_info.filename));

        let mut deleted = false;

        if model_info.is_directory {
            if model_path.exists() && model_path.is_dir() {
                std::fs::remove_dir_all(&model_path)?;
                deleted = true;
            }
        } else {
            if model_path.exists() {
                std::fs::remove_file(&model_path)?;
                deleted = true;
            }
        }

        if partial_path.exists() {
            std::fs::remove_file(&partial_path)?;
            deleted = true;
        }

        if !deleted {
            anyhow::bail!("No model files found to delete");
        }

        // Remove custom models from list
        if model_info.is_custom {
            let mut models = self.models.lock().unwrap();
            models.remove(model_id);
        }

        self.update_download_status();
        tracing::info!("Deleted model: {}", model_id);
        Ok(())
    }

    /// List downloaded models
    pub fn list_downloaded(&self) -> Vec<ModelInfo> {
        let models = self.models.lock().unwrap();
        models
            .values()
            .filter(|m| m.is_downloaded)
            .cloned()
            .collect()
    }

    /// Check if model is downloaded
    pub fn is_downloaded(&self, model_id: &str) -> bool {
        self.get_model_info(model_id)
            .map(|m| m.is_downloaded)
            .unwrap_or(false)
    }

    /// Get models by engine type
    pub fn get_models_by_engine(&self, engine: &str) -> Vec<ModelInfo> {
        let models = self.models.lock().unwrap();
        models
            .values()
            .filter(|m| {
                let engine_str = match m.engine_type {
                    crate::models::registry::EngineType::Whisper => "whisper",
                    crate::models::registry::EngineType::SenseVoice => "sensevoice",
                    crate::models::registry::EngineType::Parakeet => "parakeet",
                    crate::models::registry::EngineType::Moonshine => "moonshine",
                    crate::models::registry::EngineType::MoonshineStreaming => "moonshine-streaming",
                    crate::models::registry::EngineType::GigaAM => "gigaam",
                    crate::models::registry::EngineType::Canary => "canary",
                    crate::models::registry::EngineType::Cohere => "cohere",
                };
                engine_str.eq_ignore_ascii_case(engine)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_sha256.txt");

        std::fs::write(&test_file, "hello world").unwrap();

        let hash = ModelManager::compute_sha256(&test_file).unwrap();
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");

        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_verify_sha256_skip_on_none() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_verify.txt");

        std::fs::write(&test_file, "test data").unwrap();
        let result = ModelManager::verify_sha256(&test_file, None, "test");
        assert!(result.is_ok());

        std::fs::remove_file(test_file).ok();
    }
}
