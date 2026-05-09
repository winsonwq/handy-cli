// handy-cli - AI transcription CLI tool
//!
//! Extracted from Handy (Tauri app) to create a standalone CLI tool.

mod audio;
mod commands;
mod config;
mod error;
mod models;
mod server;
mod transcriber;
mod vad;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "handy-cli")]
#[command(about = "AI transcription CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server for transcription
    Serve {
        /// Port to listen on
        #[arg(long, default_value_t = 8765)]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Engine to use (whisper, sensevoice, parakeet, moonshine, gigaam, canary, cohere)
        #[arg(long, default_value = "sensevoice")]
        engine: String,

        /// Model to use
        #[arg(long)]
        model: Option<String>,

        /// VAD threshold (0.0-1.0)
        #[arg(long, default_value_t = 0.5)]
        vad_threshold: f32,

        /// Language code (auto for automatic detection)
        #[arg(long, default_value = "auto")]
        language: String,
    },
    /// List available models
    ListModels {
        /// Filter by engine type (whisper, sensevoice, parakeet, moonshine, gigaam, canary, cohere)
        #[arg(long)]
        engine: Option<String>,
    },
    /// Download a model
    Download {
        /// Model ID to download
        #[arg(long)]
        model: String,
    },
    /// Delete a downloaded model
    Delete {
        /// Model ID to delete
        #[arg(long)]
        model: String,
    },
    /// Check environment
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            port,
            host,
            engine,
            model,
            vad_threshold,
            language,
        } => {
            info!("Starting handy-cli server on {}:{}", host, port);
            commands::serve::run(host, port, &engine, model, vad_threshold, &language).await?;
        }
        Commands::ListModels { engine } => {
            commands::list_models::run(engine.as_deref()).await?;
        }
        Commands::Download { model } => {
            commands::download::run(&model).await?;
        }
        Commands::Delete { model } => {
            commands::delete::run(&model).await?;
        }
        Commands::Doctor => {
            commands::doctor::run().await?;
        }
    }

    Ok(())
}
