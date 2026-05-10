// serve command - start HTTP server

use crate::server;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub async fn run(
    host: String,
    port: u16,
    engine: &str,
    model: Option<String>,
    vad_threshold: f32,
    language: &str,
) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    // Build router with CORS (body limit is set in create_app)
    let app = server::create_app(engine, model, vad_threshold, language)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    info!("Starting HTTP server");
    info!("  Engine: {}", engine);
    info!("  Language: {}", language);
    info!("  VAD threshold: {}", vad_threshold);
    info!("\nEndpoints:");
    info!("  GET  /api/health                  - Health check");
    info!("  GET  /api/models                 - List available models");
    info!("  GET  /api/models/downloaded      - List downloaded models");
    info!("  POST /api/transcribe             - Transcribe audio (JSON body)");
    info!("  POST /api/transcribe/stream      - Stream transcription (SSE)");
    info!("\nPress Ctrl+C to stop the server\n");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            info!("Shutting down server...");
        })
        .await?;

    Ok(())
}
