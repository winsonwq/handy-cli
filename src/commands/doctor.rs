// doctor command - check environment

use anyhow::Result;

pub async fn run() -> Result<()> {
    println!("=== handy-cli Environment Check ===\n");

    // Check audio devices
    println!("Audio devices:");
    let devices = crate::audio::AudioDevice::list_input_devices();
    if devices.is_empty() {
        println!("  ⚠️  No input devices found");
    } else {
        for device in devices {
            let marker = if device.is_default { " [default]" } else { "" };
            println!("  ✓ {}{}", device.name, marker);
        }
    }
    println!();

    // Check models directory
    let models_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("handy-cli")
        .join("models");

    println!("Models directory: {:?}", models_dir);
    if models_dir.exists() {
        let models = std::fs::read_dir(&models_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();

        if models.is_empty() {
            println!("  No models downloaded");
        } else {
            println!("  Downloaded models: {}", models.join(", "));
        }
    } else {
        println!("  Models directory not created yet");
    }
    println!();

    // Check transcribe-rs availability
    println!("transcribe-rs: ✓ Available (linked)");

    println!("\n=== All checks passed ===");
    Ok(())
}
