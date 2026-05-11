// Audio capture module
// Note: Audio capture is currently handled via the HTTP API streaming endpoint
// This module is reserved for future direct audio device capture support

#![allow(dead_code)] // TODO: Remove when implementing audio device capture

pub mod device;

pub use device::AudioDevice;
