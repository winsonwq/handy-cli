// Audio device enumeration

use cpal::Device;

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

impl AudioDevice {
    pub fn from_cpal(device: &Device, is_default: bool) -> Self {
        let name = device
            .name()
            .unwrap_or_else(|_| "Unknown".to_string());
        Self { name, is_default }
    }

    /// List all available input audio devices
    pub fn list_input_devices() -> Vec<Self> {
        let mut devices = Vec::new();

        if let Ok(host) = cpal::default_host().try_lock() {
            let default = cpal::default_input_device();

            for device in host.input_devices().into_iter().flatten() {
                let is_default = default
                    .as_ref()
                    .map(|d| d.name().ok() == device.name().ok())
                    .unwrap_or(false);

                devices.push(AudioDevice::from_cpal(&device, is_default));
            }
        }

        devices
    }
}
