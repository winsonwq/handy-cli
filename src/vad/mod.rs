// Voice Activity Detection module using vad-rs

#[allow(dead_code)]
pub mod processor {
    use anyhow::Result;
    use vad_rs::Vad;

    pub struct VadProcessor {
        vad: Vad,
        threshold: f32,
    }

    impl VadProcessor {
        pub fn new(model_path: &std::path::Path, threshold: f32) -> Result<Self> {
            let vad = Vad::new(model_path, 16000)
                .map_err(|e| anyhow::anyhow!("Failed to load VAD model: {}", e))?;
            Ok(Self { vad, threshold })
        }

        /// Process audio samples and return whether speech is detected
        pub fn is_speech(&mut self, samples: &[f32]) -> bool {
            match self.vad.compute(samples) {
                Ok(result) => result.prob > self.threshold,
                Err(_) => false,
            }
        }

        /// Reset VAD state
        pub fn reset(&mut self) {
            // vad-rs maintains state internally via h/c tensors
        }
    }
}
