//! Route definitions for the Axum server

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use super::{handlers, state::AppState};

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // Health endpoints
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::ready_check))
        // Config endpoints
        .route("/config", get(handlers::get_config))
        .route("/config", post(handlers::update_config))
        // Command CRUD endpoints
        .route("/commands", get(handlers::list_commands))
        .route("/commands", post(handlers::create_command))
        .route("/commands/:id", get(handlers::get_command))
        .route("/commands/:id", put(handlers::update_command))
        .route("/commands/:id", delete(handlers::delete_command))
        // Workflow CRUD endpoints
        .route("/workflows", get(handlers::list_workflows))
        .route("/workflows", post(handlers::create_workflow))
        .route("/workflows/:id", get(handlers::get_workflow))
        .route("/workflows/:id", put(handlers::update_workflow))
        .route("/workflows/:id", delete(handlers::delete_workflow))
        // Script CRUD endpoints
        .route("/scripts", get(handlers::list_scripts))
        .route("/scripts", post(handlers::create_script))
        .route("/scripts/:id", get(handlers::get_script))
        .route("/scripts/:id", put(handlers::update_script))
        .route("/scripts/:id", delete(handlers::delete_script))
        // Tailscale endpoints
        .route("/tailscale/status", get(handlers::get_tailscale_status))
        .route("/tailscale/config", post(handlers::update_tailscale_config))
        // TTS endpoints
        .route("/tts/speak", post(handlers::speak))
        .route("/voice/test", post(handlers::test_voice))
        .route("/voice/process", post(handlers::process_voice_command))
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
        .route("/dictation/backspace", post(handlers::backspace_dictation))
        .route("/dictation/test-keyboard", get(handlers::test_keyboard));

    // For SPA: serve static files, but fallback to index.html for client-side routing
    let serve_dir = ServeDir::new("frontend/dist")
        .not_found_service(ServeFile::new("frontend/dist/index.html"));

    Router::new()
        .nest("/api", api_routes) // API routes have priority
        .fallback_service(serve_dir) // Serve static files, fallback to index.html for SPA routes
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
