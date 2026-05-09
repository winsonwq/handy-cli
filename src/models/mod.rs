// Model management for handy-cli

pub mod manager;
pub mod registry;

pub use manager::ModelManager;
pub use registry::{ModelInfo, ModelRegistry, EngineType};
