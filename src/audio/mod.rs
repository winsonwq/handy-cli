// Audio capture module

pub mod capture;
pub mod device;
pub mod manager;

pub use capture::AudioCapture;
pub use device::AudioDevice;
pub use manager::AudioCaptureManager;
