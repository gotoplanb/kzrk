use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub session_id: Uuid,
    pub player_id: String,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub player_name: String,
    pub starting_money: Option<u32>,
    pub starting_airport: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameResponse {
    pub session_id: Uuid,
    pub player_name: String,
    pub game_state: GameStateResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateResponse {
    pub player: PlayerInfo,
    pub current_market: MarketInfo,
    pub available_destinations: Vec<DestinationInfo>,
    pub active_events: Vec<EventInfo>,
    pub statistics: StatisticsInfo,
    pub turn_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub name: String,
    pub money: u32,
    pub current_airport: String,
    pub fuel: u32,
    pub max_fuel: u32,
    pub cargo_inventory: HashMap<String, u32>,
    pub cargo_weight: u32,
    pub max_cargo_weight: u32,
    pub fuel_efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfo {
    pub airport_id: String,
    pub airport_name: String,
    pub fuel_price: u32,
    pub cargo_prices: HashMap<String, u32>,
    pub last_updated: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationInfo {
    pub airport_id: String,
    pub airport_name: String,
    pub distance: f64,
    pub fuel_required: u32,
    pub can_travel: bool,
    pub fuel_price: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub event_type: String,
    pub affected_cargo: String,
    pub affected_airport: String,
    pub price_multiplier: f32,
    pub turns_remaining: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsInfo {
    pub total_revenue: u32,
    pub total_expenses: u32,
    pub net_profit: u32,
    pub cargo_trades: u32,
    pub fuel_purchased: u32,
    pub distances_traveled: f64,
    pub airports_visited: Vec<String>,
    pub best_single_trade: u32,
    pub most_profitable_cargo: String,
    pub efficiency_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRequest {
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelResponse {
    pub success: bool,
    pub message: String,
    pub fuel_consumed: Option<u32>,
    pub new_location: Option<String>,
    pub game_state: Option<GameStateResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRequest {
    pub cargo_type: String,
    pub quantity: u32,
    pub action: TradeAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResponse {
    pub success: bool,
    pub message: String,
    pub transaction_amount: Option<u32>,
    pub new_money: Option<u32>,
    pub new_inventory: Option<HashMap<String, u32>>,
    pub game_state: Option<GameStateResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelRequest {
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelResponse {
    pub success: bool,
    pub message: String,
    pub cost: Option<u32>,
    pub new_fuel: Option<u32>,
    pub new_money: Option<u32>,
    pub game_state: Option<GameStateResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
    pub data: Option<serde_json::Value>,
}