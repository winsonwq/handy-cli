// HTTP server module

pub mod handlers;

use crate::server::handlers::{
    health, list_downloaded_models, list_models, transcribe, transcribe_stream, RouterState,
};
use axum::extract::DefaultBodyLimit;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn create_app(
    engine: &str,
    model: Option<String>,
    vad_threshold: f32,
    language: &str,
) -> Router {
    let state = Arc::new(RouterState::new(
        engine.to_string(),
        model,
        vad_threshold,
        language.to_string(),
    ));

    // Override default 2MB body limit to 100MB for large audio uploads
    let body_limit = DefaultBodyLimit::max(100 * 1024 * 1024);

    Router::new()
        // Health and model endpoints
        .route("/api/health", get(health))
        .route("/api/models", get(list_models))
        .route("/api/models/downloaded", get(list_downloaded_models))
        // Transcription endpoints with increased body limit
        .route(
            "/api/transcribe",
            post(transcribe).layer(body_limit),
        )
        .route(
            "/api/transcribe/stream",
            post(transcribe_stream).layer(body_limit),
        )
        .with_state(state)
}
