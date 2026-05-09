// list_models command - list available models

use anyhow::Result;
use crate::models::registry::ModelRegistry;

pub async fn run() -> Result<()> {
    println!("=== Available Models ===\n");

    let models = ModelRegistry::available_models();

    for model in &models {
        let recommended = if model.is_recommended { " [recommended]" } else { "" };
        let engine = format!("{:?}", model.engine_type);
        println!("{} ({})", model.name, engine);
        println!("  ID: {}", model.id);
        println!("  Size: {} MB", model.size_mb);
        println!("  Accuracy: {:.0}%", model.accuracy_score * 100.0);
        println!("  Speed: {:.0}%", model.speed_score * 100.0);
        println!("  {}", model.description);
        println!("{}", recommended);
        println!();
    }

    Ok(())
}
