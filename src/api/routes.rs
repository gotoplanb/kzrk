#![allow(dead_code)]

use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{
    handlers, multiplayer_handlers, multiplayer_service::MultiplayerGameService,
    service::GameService, stateless_handlers,
};

pub fn create_router(service: GameService) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))

        // Single-player game management (backwards compatibility)
        .route("/game", post(handlers::create_game))
        .route("/game/:session_id", get(handlers::get_game_state))

        // Single-player game actions (backwards compatibility)
        .route("/game/:session_id/travel", post(handlers::travel))
        .route("/game/:session_id/trade", post(handlers::trade))
        .route("/game/:session_id/fuel", post(handlers::buy_fuel))

        // Reference data
        .route("/airports", get(handlers::get_available_airports))
        .route("/cargo", get(handlers::get_available_cargo))

        // Add the service as state
        .with_state(service)
}

pub fn create_multiplayer_router(service: MultiplayerGameService) -> Router {
    Router::new()
        // Health check
        .route("/health", get(stateless_handlers::health_check))

        // Multiplayer room management
        .route("/rooms", post(multiplayer_handlers::create_room))
        .route("/rooms", get(multiplayer_handlers::list_rooms))
        .route("/rooms/:room_id/join", post(multiplayer_handlers::join_room))
        .route("/rooms/:room_id/players/:player_id/leave", post(multiplayer_handlers::leave_room))

        // Multiplayer game state
        .route("/rooms/:room_id/players/:player_id/state", get(multiplayer_handlers::get_room_state))

        // Multiplayer player actions
        .route("/rooms/:room_id/players/:player_id/travel", post(multiplayer_handlers::player_travel))
        .route("/rooms/:room_id/players/:player_id/trade", post(multiplayer_handlers::player_trade))
        .route("/rooms/:room_id/players/:player_id/fuel", post(multiplayer_handlers::player_buy_fuel))

        // Session management
        .route("/players/:player_name/sessions", get(multiplayer_handlers::find_player_sessions))

        // Message board endpoints
        .route("/rooms/:room_id/players/:player_id/messages", post(multiplayer_handlers::post_message))
        .route("/rooms/:room_id/players/:player_id/messages", get(multiplayer_handlers::get_messages))

        // Reference data (stateless handlers)
        .route("/airports", get(stateless_handlers::get_available_airports))
        .route("/cargo", get(stateless_handlers::get_available_cargo))

        // Add the service as state
        .with_state(service)
}
