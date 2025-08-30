mod api;
mod data;
mod models;
mod systems;
mod ui;

use std::env;

use api::{routes::create_router, service::GameService};
use tower_http::cors::CorsLayer;
use tracing::{Level, info};
use ui::TerminalUI;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "api" {
        run_api_server().await;
    } else {
        run_cli_game();
    }
}

async fn run_api_server() {
    info!("Starting KZRK Game API server...");

    let service = GameService::new();
    let app = create_router(service).layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    info!("API server running on http://127.0.0.1:3000");
    info!("Endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /game - Create new game");
    info!("  GET  /game/:session_id - Get game state");
    info!("  POST /game/:session_id/travel - Travel to destination");
    info!("  POST /game/:session_id/trade - Buy/sell cargo");
    info!("  POST /game/:session_id/fuel - Buy fuel");
    info!("  GET  /airports - List available airports");
    info!("  GET  /cargo - List available cargo types");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

fn run_cli_game() {
    println!("Starting KZRK CLI game...");
    TerminalUI::run_game_loop();
}
