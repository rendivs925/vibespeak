//! Route definitions for the Axum server

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use super::{handlers, state::AppState};

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // Health endpoints
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::ready_check))
        // Config endpoints
        .route("/config", get(handlers::get_config))
        .route("/config", post(handlers::update_config))
        // Tailscale endpoints
        .route("/tailscale/status", get(handlers::get_tailscale_status))
        .route("/tailscale/config", post(handlers::update_tailscale_config))
        // TTS endpoints
        .route("/tts/speak", post(handlers::speak))
        .route("/voice/test", post(handlers::test_voice))
        // Remote control endpoints
        .route("/remote/command", post(handlers::execute_remote_command))
        .route("/remote/mouse", post(handlers::handle_mouse_event))
        // Screen sharing endpoints
        .route("/screen/offer", post(handlers::create_screen_offer))
        .route("/screen/answer", post(handlers::handle_screen_answer))
        // Dictation endpoints
        .route("/dictation/start", post(handlers::start_dictation))
        .route("/dictation/stop", post(handlers::stop_dictation))
        .route("/dictation/insert", post(handlers::insert_dictation))
        .route("/dictation/type", post(handlers::type_dictation))
        .route("/dictation/test-keyboard", get(handlers::test_keyboard));

    Router::new()
        .nest("/api", api_routes)
        .fallback_service(vibespeak_frontend::create_leptos_router().into_service()) // Leptos handles all non-API routes
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
