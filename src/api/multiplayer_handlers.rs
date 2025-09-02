use axum::{
    Json as JsonExtract,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

use crate::api::{models::*, multiplayer_service::MultiplayerGameService};

pub async fn create_room(
    State(service): State<MultiplayerGameService>,
    JsonExtract(request): JsonExtract<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.create_room(request.name, request.host_player_name, request.max_players) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "CreateRoomError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn list_rooms(
    State(service): State<MultiplayerGameService>,
) -> Result<Json<Vec<RoomInfo>>, (StatusCode, Json<ErrorResponse>)> {
    match service.list_rooms() {
        Ok(rooms) => Ok(Json(rooms)),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "ListRoomsError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn join_room(
    State(service): State<MultiplayerGameService>,
    Path(room_id): Path<Uuid>,
    JsonExtract(request): JsonExtract<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.join_room(room_id, request.player_name, request.starting_airport) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "JoinRoomError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn leave_room(
    State(service): State<MultiplayerGameService>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<LeaveRoomResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.leave_room(room_id, player_id) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "LeaveRoomError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn get_room_state(
    State(service): State<MultiplayerGameService>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MultiplayerGameStateResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.get_room_state(room_id, player_id) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "GetRoomStateError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn player_travel(
    State(service): State<MultiplayerGameService>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
    JsonExtract(request): JsonExtract<TravelRequest>,
) -> Result<Json<PlayerTravelResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.player_travel(room_id, player_id, request.destination) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "PlayerTravelError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn player_trade(
    State(service): State<MultiplayerGameService>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
    JsonExtract(request): JsonExtract<TradeRequest>,
) -> Result<Json<PlayerTradeResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.player_trade(room_id, player_id, request) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "PlayerTradeError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn player_buy_fuel(
    State(service): State<MultiplayerGameService>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
    JsonExtract(request): JsonExtract<FuelRequest>,
) -> Result<Json<PlayerFuelResponse>, (StatusCode, Json<ErrorResponse>)> {
    match service.player_buy_fuel(room_id, player_id, request) {
        Ok(response) => Ok(Json(response)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "PlayerFuelError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}

pub async fn find_player_sessions(
    State(service): State<MultiplayerGameService>,
    Path(player_name): Path<String>,
) -> Result<Json<Vec<PlayerSessionInfo>>, (StatusCode, Json<ErrorResponse>)> {
    match service.find_player_sessions(&player_name) {
        Ok(sessions) => Ok(Json(sessions)),
        Err(error) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "FindPlayerSessionsError".to_string(),
                message: error,
                details: None,
            }),
        )),
    }
}
