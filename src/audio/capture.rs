// Audio capture using cpal

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    sample_rate: u32,
    channels: u16,
    buffer: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    stream: Option<cpal::Stream>,
}

impl AudioCapture {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 1,
            buffer: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            stream: None,
        }
    }

    /// Start recording
    pub fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device
            .default_input_config()
            .map_err(|e| anyhow::anyhow!("Failed to get default input config: {}", e))?;

        let sample_rate = self.sample_rate;
        let channels = self.channels;

        let buffer = self.buffer.clone();
        let is_recording = self.is_recording.clone();
        is_recording.store(true, Ordering::SeqCst);

        let err_fn = |err| tracing::error!("Audio stream error: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_recording.load(Ordering::SeqCst) {
                        let mut buffer = buffer.lock().unwrap();
                        // Convert stereo to mono if needed
                        for chunk in data.chunks(channels as usize) {
                            let sample: f32 = if channels as usize == chunk.len() || chunk.len() == 1 {
                                chunk.iter().sum::<f32>() / chunk.len() as f32
                            } else {
                                chunk.iter().sum::<f32>() / channels as f32
                            };
                            buffer.push(sample);
                        }
                    }
                };

                device.build_input_stream(&config.into(), data_callback, err_fn, None)?
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported sample format: only F32 supported"));
            }
        };

        stream.play()?;
        self.stream = Some(stream);

        tracing::info!("Audio capture started at {}Hz", sample_rate);
        Ok(())
    }

    /// Stop recording and return the collected samples
    pub fn stop(&mut self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        self.stream = None;

        let mut buffer = self.buffer.lock().unwrap();
        let samples = buffer.clone();
        buffer.clear();

        tracing::info!("Audio capture stopped, {} samples", samples.len());
        samples
    }

    /// Get current buffer contents
    pub fn get_buffer(&self) -> Vec<f32> {
        self.buffer.lock().unwrap().clone()
    }

    /// Clear buffer
    pub fn clear_buffer(&self) {
        self.buffer.lock().unwrap().clear();
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new(16000)
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);
    }
}
