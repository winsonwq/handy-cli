// Voice Activity Detection module using vad-rs

use anyhow::Result;
use vad_rs::{Vad, VadEngine, VadMode, VadWebRTMode};

pub struct VadProcessor {
    vad: Vad,
    threshold: f32,
}

impl VadProcessor {
    pub fn new(threshold: f32) -> Result<Self> {
        let vad = Vad::new(
            VadEngine::DEFAULT,
            VadMode::NORMAL,
            16000,
            VadWebRTMode::Unset,
        )?;

        Ok(Self { vad, threshold })
    }

    /// Process audio samples and return whether speech is detected
    pub fn is_speech(&self, samples: &[f32]) -> bool {
        // vad-rs expects i16 samples at 16kHz
        let pcm: Vec<i16> = samples
            .iter()
            .map(|s| ((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16))
            .collect();

        // Process in 30ms windows (480 samples at 16kHz)
        let window_size = (16000 * 30 / 1000) as usize;

        for window in pcm.chunks(window_size) {
            if let Some(prob) = self.vad.predict(window) {
                if prob > self.threshold {
                    return true;
                }
            }
        }

        false
    }

    /// Reset VAD state
    pub fn reset(&mut self) {
        // vad-rs doesn't have explicit reset
    }
}
