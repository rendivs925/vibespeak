// Presentation layer modules
// Provides CLI, web, and API interfaces for the voice automation system

// Old warp-based modules (deprecated - migration to Axum in progress)
// pub mod api;
// pub mod web;

// New Axum-based server
pub mod axum_server;
pub mod cli;
