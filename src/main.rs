mod api;
mod data;
mod models;
mod systems;
mod ui;

use std::env;

use api::{multiplayer_service::MultiplayerGameService, routes::create_multiplayer_router};
use tower_http::cors::CorsLayer;
use tracing::{Level, info};
use ui::TerminalUI;

#[cfg(feature = "gui")]
use ui::egui_app::KzrkEguiApp;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "api" {
        run_api_server().await;
    } else if args.len() > 1 && args[1] == "gui" {
        run_egui_game();
    } else {
        run_cli_game();
    }
}

async fn run_api_server() {
    info!("Starting KZRK Multiplayer Game API server...");

    let service = MultiplayerGameService::new();
    let app = create_multiplayer_router(service).layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    info!("Multiplayer API server running on http://127.0.0.1:3000");
    info!("Endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /rooms - Create new game room");
    info!("  GET  /rooms - List available rooms");
    info!("  POST /rooms/:room_id/join - Join a room");
    info!("  POST /rooms/:room_id/players/:player_id/leave - Leave room");
    info!("  GET  /rooms/:room_id/players/:player_id/state - Get room state");
    info!("  POST /rooms/:room_id/players/:player_id/travel - Travel to destination");
    info!("  POST /rooms/:room_id/players/:player_id/trade - Buy/sell cargo");
    info!("  POST /rooms/:room_id/players/:player_id/fuel - Buy fuel");
    info!("  GET  /airports - List available airports");
    info!("  GET  /cargo - List available cargo types");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[cfg(feature = "gui")]
fn run_egui_game() {
    println!("Starting KZRK GUI game...");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("KZRK Aviation Trading Game"),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "KZRK Aviation Trading",
        options,
        Box::new(|_cc| Ok(Box::new(KzrkEguiApp::new()))),
    ) {
        eprintln!("Failed to run egui app: {}", e);
    }
}

#[cfg(not(feature = "gui"))]
fn run_egui_game() {
    eprintln!("GUI feature not enabled. Compile with --features gui");
    std::process::exit(1);
}

fn run_cli_game() {
    println!("Starting KZRK CLI game...");
    TerminalUI::run_game_loop();
}
