// Audio capture manager for async-safe recording
// Manages audio capture in a separate thread with channel communication

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::{mpsc, oneshot};

/// Audio capture commands
#[derive(Debug)]
pub enum AudioCommand {
    Start {
        sample_rate: u32,
        response: oneshot::Sender<Result<()>>,
    },
    Stop {
        response: oneshot::Sender<Result<Vec<f32>>>,
    },
    GetBuffer {
        response: oneshot::Sender<Vec<f32>>,
    },
    IsRecording {
        response: oneshot::Sender<bool>,
    },
}

/// Audio capture manager that runs in a separate thread
pub struct AudioCaptureManager {
    sender: mpsc::Sender<AudioCommand>,
}

impl AudioCaptureManager {
    /// Create a new audio capture manager and start the background thread
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<AudioCommand>(10);

        // Spawn the audio capture thread
        thread::spawn(move || {
            Self::run_capture_loop(receiver);
        });

        Self { sender }
    }

    /// Start recording audio
    pub async fn start(&self, sample_rate: u32) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(AudioCommand::Start {
                sample_rate,
                response,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Audio capture thread closed"))?;
        receiver.await.map_err(|_| anyhow::anyhow!("Audio capture thread closed"))?
    }

    /// Stop recording and return collected samples
    pub async fn stop(&self) -> Result<Vec<f32>> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(AudioCommand::Stop { response })
            .await
            .map_err(|_| anyhow::anyhow!("Audio capture thread closed"))?;
        receiver.await.map_err(|_| anyhow::anyhow!("Audio capture thread closed"))?
    }

    /// Get current buffer contents
    pub async fn get_buffer(&self) -> Vec<f32> {
        let (response, receiver) = oneshot::channel();
        let _ = self
            .sender
            .send(AudioCommand::GetBuffer { response })
            .await;
        receiver.await.unwrap_or_default()
    }

    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        let (response, receiver) = oneshot::channel();
        let _ = self
            .sender
            .send(AudioCommand::IsRecording { response })
            .await;
        receiver.await.unwrap_or(false)
    }

    /// Background thread that manages audio capture
    fn run_capture_loop(mut receiver: mpsc::Receiver<AudioCommand>) {
        let mut capture: Option<InnerCapture> = None;

        // Use a simple loop that blocks on receiving
        while let Some(cmd) = receiver.blocking_recv() {
            match cmd {
                AudioCommand::Start {
                    sample_rate,
                    response,
                } => {
                    if capture.is_some() {
                        let _ = response.send(Err(anyhow::anyhow!("Already recording")));
                        continue;
                    }

                    match InnerCapture::new(sample_rate) {
                        Ok(mut cap) => {
                            if let Err(e) = cap.start() {
                                let _ = response.send(Err(e));
                            } else {
                                capture = Some(cap);
                                let _ = response.send(Ok(()));
                            }
                        }
                        Err(e) => {
                            let _ = response.send(Err(e));
                        }
                    }
                }
                AudioCommand::Stop { response } => {
                    if let Some(mut cap) = capture.take() {
                        let samples = cap.stop();
                        let _ = response.send(Ok(samples));
                    } else {
                        let _ = response.send(Err(anyhow::anyhow!("Not recording")));
                    }
                }
                AudioCommand::GetBuffer { response } => {
                    if let Some(ref cap) = capture {
                        let _ = response.send(cap.get_buffer());
                    } else {
                        let _ = response.send(Vec::new());
                    }
                }
                AudioCommand::IsRecording { response } => {
                    let _ = response.send(capture.is_some());
                }
            }
        }
    }
}

/// Inner capture implementation (runs in a single thread)
struct InnerCapture {
    sample_rate: u32,
    channels: u16,
    buffer: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    stream: Option<cpal::Stream>,
}

impl InnerCapture {
    fn new(sample_rate: u32) -> Result<Self> {
        Ok(Self {
            sample_rate,
            channels: 1,
            buffer: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            stream: None,
        })
    }

    fn start(&mut self) -> Result<()> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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
                        if let Ok(mut buf) = buffer.lock() {
                            // Convert stereo to mono if needed
                            for chunk in data.chunks(channels as usize) {
                                let sample: f32 = if channels as usize == chunk.len() || chunk.len() == 1 {
                                    chunk.iter().sum::<f32>() / chunk.len() as f32
                                } else {
                                    chunk.iter().sum::<f32>() / channels as f32
                                };
                                buf.push(sample);
                            }
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

    fn stop(&mut self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        self.stream = None;

        let samples = {
            let mut buffer = self.buffer.lock().unwrap();
            let mut samples = Vec::new();
            std::mem::swap(&mut samples, &mut buffer);
            samples
        };
        tracing::info!("Audio capture stopped, {} samples", samples.len());
        samples
    }

    fn get_buffer(&self) -> Vec<f32> {
        self.buffer.lock().unwrap().clone()
    }
}

impl Drop for InnerCapture {
    fn drop(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);
    }
}
