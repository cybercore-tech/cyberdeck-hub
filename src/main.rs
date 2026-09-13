//! # CYBERDECK Core
//!
//! The entry point and primary controller for the CYBERDECK system: the
//! central hub for every tool built in this ecosystem (live status, quick
//! links, wiki cross-links), plus the original hardware-diagnostic scan
//! suite and its reports viewer.

mod cybergrid;
mod dispatcher;
mod hub;
mod modules;
mod parser;
mod routes;
mod types;
mod views;

use crate::parser::parse_cyberdeck_script;
use crate::types::CyberdeckState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Cyberdeck System...");
    let state = Arc::new(Mutex::new(CyberdeckState {
        display_active: false,
        active_modules: Vec::new(),
        stealth_mode: false,
        reports_generated: 0,
        execution_log: Vec::new(),
    }));

    let env_script = std::env::var("Cyberdeck_ENV").unwrap_or_else(|_| "init_display".to_string());
    let commands = parse_cyberdeck_script(&env_script);
    for cmd in commands {
        dispatcher::execute_cyberdeck_command(cmd, &state).await;
    }

    let app = Router::new()
        .route("/", get(routes::get_index_page))
        .route("/partial/hub", get(routes::partial_hub))
        .route("/scans", get(routes::scans_view))
        .route("/reports", get(routes::reports_view))
        .route("/reports/*path", get(routes::report_view))
        .route("/api/reports/*path", get(routes::report_fragment))
        .route("/cyberdeck/api/state", get(routes::get_cyberdeck_state))
        .route("/cyberdeck/api/command", post(routes::post_cyberdeck_command))
        .route("/cyberdeck/api/action", post(routes::post_cyberdeck_action))
        .route("/cyberdeck/api/list", get(routes::list_diagnostics))
        .route("/api/cybergrid/themes", get(cybergrid::list_themes))
        .route("/api/cybergrid/css/:name", get(cybergrid::theme_css))
        .route("/vendor/tokens.css", get(routes::tokens_css))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let port = std::env::var("CYBERDECK_PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    println!("[Cyberdeck Core] Hub active on http://127.0.0.1:{}", port);
    axum::serve(listener, app).await.unwrap();
}
