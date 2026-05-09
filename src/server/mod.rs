// HTTP server module

pub mod handlers;

use crate::server::handlers::{
    health, list_downloaded_models, list_models, transcribe, RouterState,
};
use axum::{routing::{get, post}, Router};
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

    Router::new()
        .route("/api/health", get(health))
        .route("/api/models", get(list_models))
        .route("/api/models/downloaded", get(list_downloaded_models))
        .route("/api/transcribe", post(transcribe))
        .with_state(state)
}
