#![allow(dead_code)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

use crate::api::{models::*, service::GameService};

pub async fn create_game(
    State(service): State<GameService>,
    Json(request): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.create_game(request) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "GameCreationError".to_string(),
                message: e,
                details: None,
            }),
        )),
    }
}

pub async fn get_game_state(
    State(service): State<GameService>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<GameStateResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.get_game_state(session_id) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "GameNotFound".to_string(),
                message: e,
                details: None,
            }),
        )),
    }
}

pub async fn travel(
    State(service): State<GameService>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<TravelRequest>,
) -> Result<Json<TravelResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.travel(session_id, request) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "TravelError".to_string(),
                message: e,
                details: None,
            }),
        )),
    }
}

pub async fn trade(
    State(service): State<GameService>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<TradeRequest>,
) -> Result<Json<TradeResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.trade(session_id, request) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "TradeError".to_string(),
                message: e,
                details: None,
            }),
        )),
    }
}

pub async fn buy_fuel(
    State(service): State<GameService>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<FuelRequest>,
) -> Result<Json<FuelResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.buy_fuel(session_id, request) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "FuelPurchaseError".to_string(),
                message: e,
                details: None,
            }),
        )),
    }
}

pub async fn health_check() -> Json<SuccessResponse> {
    Json(SuccessResponse {
        message: "KZRK Game API is running".to_string(),
        data: None,
    })
}

pub async fn get_available_airports(
    State(_service): State<GameService>,
) -> Json<Vec<serde_json::Value>> {
    use crate::data::airports::get_default_airports;

    let airports = get_default_airports();
    let airport_list: Vec<serde_json::Value> = airports
        .iter()
        .map(|(id, airport)| {
            serde_json::json!({
                "id": id,
                "name": &airport.name,
                "latitude": airport.coordinates.0,
                "longitude": airport.coordinates.1
            })
        })
        .collect();

    Json(airport_list)
}

pub async fn get_available_cargo(
    State(_service): State<GameService>,
) -> Json<Vec<serde_json::Value>> {
    use crate::data::cargo_types::get_default_cargo_types;

    let cargo_types = get_default_cargo_types();
    let cargo_list: Vec<serde_json::Value> = cargo_types
        .iter()
        .map(|(id, cargo)| {
            serde_json::json!({
                "id": id,
                "name": &cargo.name,
                "base_price": cargo.base_price,
                "weight": cargo.weight_per_unit,
                "volatility": cargo.volatility
            })
        })
        .collect();

    Json(cargo_list)
}
