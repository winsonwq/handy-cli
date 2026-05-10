// HTTP server module

pub mod handlers;

use crate::server::handlers::{
    audio_start, audio_status, audio_stop, health, list_downloaded_models, list_models,
    transcribe, transcribe_stream, RouterState,
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
        // Health and model endpoints
        .route("/api/health", get(health))
        .route("/api/models", get(list_models))
        .route("/api/models/downloaded", get(list_downloaded_models))
        // Transcription endpoints
        .route("/api/transcribe", post(transcribe))
        .route("/api/transcribe/stream", post(transcribe_stream))
        // Audio recording endpoints
        .route("/api/audio/start", post(audio_start))
        .route("/api/audio/stop", post(audio_stop))
        .route("/api/audio/status", get(audio_status))
        .with_state(state)
}
