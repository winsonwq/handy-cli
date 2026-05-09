// Audio device enumeration

use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

impl AudioDevice {
    pub fn from_cpal(device: &cpal::Device, is_default: bool) -> Self {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        Self { name, is_default }
    }

    pub fn list_input_devices() -> Vec<Self> {
        let mut devices = Vec::new();

        let host = cpal::default_host();
        let default = host.default_input_device();

        if let Ok(device_enum) = host.input_devices() {
            for device in device_enum {
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
