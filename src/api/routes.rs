use crate::api::handlers;
use crate::api::service::GameService;
use axum::{
    Router,
    routing::{get, post},
};

pub fn create_router(service: GameService) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))

        // Game management
        .route("/game", post(handlers::create_game))
        .route("/game/:session_id", get(handlers::get_game_state))

        // Game actions
        .route("/game/:session_id/travel", post(handlers::travel))
        .route("/game/:session_id/trade", post(handlers::trade))
        .route("/game/:session_id/fuel", post(handlers::buy_fuel))

        // Reference data
        .route("/airports", get(handlers::get_available_airports))
        .route("/cargo", get(handlers::get_available_cargo))

        // Add the service as state
        .with_state(service)
}
