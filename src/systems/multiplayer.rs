use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::{Airport, CargoType, Market, Player},
    systems::GameStatistics,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRoom {
    pub id: Uuid,
    pub name: String,
    pub host_player_id: Uuid,
    pub max_players: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub game_status: GameStatus,
    pub shared_state: SharedGameState,
    pub players: HashMap<Uuid, PlayerGameState>,
    pub player_statistics: HashMap<Uuid, GameStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum GameStatus {
    #[default]
    WaitingForPlayers,
    InProgress,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedGameState {
    pub turn_number: u32,
    pub markets: HashMap<String, Market>,
    pub airports: HashMap<String, Airport>,
    pub cargo_types: HashMap<String, CargoType>,
    pub world_time: chrono::DateTime<chrono::Utc>,
    pub last_market_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameState {
    pub player_id: Uuid,
    pub player_name: String,
    pub player: Player,
    pub is_online: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSession {
    pub player_id: Uuid,
    pub player_name: String,
    pub game_room_id: Option<Uuid>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl GameRoom {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        host_player_id: Uuid,
        host_player_name: String,
        max_players: usize,
        airports: HashMap<String, Airport>,
        cargo_types: HashMap<String, CargoType>,
    ) -> Self {
        let room_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Create initial shared state
        let mut markets = HashMap::new();
        for airport_id in airports.keys() {
            let mut market = Market::new(airport_id, 50); // Default fuel price
            // Set initial cargo prices
            for (cargo_type_id, cargo_type) in &cargo_types {
                market.set_cargo_price(cargo_type_id, cargo_type.base_price);
            }
            markets.insert(airport_id.clone(), market);
        }

        let shared_state = SharedGameState {
            turn_number: 1,
            markets,
            airports,
            cargo_types,
            world_time: now,
            last_market_update: now,
        };

        // Create host player state
        let host_player = Player::new(5000, "JFK", 200, 1000, 15.0);
        let host_player_state = PlayerGameState {
            player_id: host_player_id,
            player_name: host_player_name,
            player: host_player,
            is_online: true,
            last_seen: now,
            joined_at: now,
        };

        let mut players = HashMap::new();
        players.insert(host_player_id, host_player_state);

        let mut player_statistics = HashMap::new();
        player_statistics.insert(host_player_id, GameStatistics::new());

        Self {
            id: room_id,
            name,
            host_player_id,
            max_players,
            created_at: now,
            game_status: GameStatus::WaitingForPlayers,
            shared_state,
            players,
            player_statistics,
        }
    }

    pub fn add_player(
        &mut self,
        player_id: Uuid,
        player_name: String,
        starting_airport: Option<String>,
    ) -> Result<(), String> {
        if self.players.len() >= self.max_players {
            return Err("Room is full".to_string());
        }

        if self.players.contains_key(&player_id) {
            return Err("Player already in room".to_string());
        }

        // Check for duplicate names
        for player_state in self.players.values() {
            if player_state.player_name == player_name {
                return Err("Player name already taken in this room".to_string());
            }
        }

        let starting_airport = starting_airport.unwrap_or_else(|| "JFK".to_string());
        let player = Player::new(5000, &starting_airport, 200, 1000, 15.0);
        let now = chrono::Utc::now();

        let player_state = PlayerGameState {
            player_id,
            player_name,
            player,
            is_online: true,
            last_seen: now,
            joined_at: now,
        };

        self.players.insert(player_id, player_state);
        self.player_statistics
            .insert(player_id, GameStatistics::new());

        Ok(())
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> Result<(), String> {
        if !self.players.contains_key(&player_id) {
            return Err("Player not in room".to_string());
        }

        self.players.remove(&player_id);
        self.player_statistics.remove(&player_id);

        // If host leaves and there are other players, transfer host to first remaining player
        if self.host_player_id == player_id && !self.players.is_empty() {
            self.host_player_id = *self.players.keys().next().unwrap();
        }

        Ok(())
    }

    pub fn get_player(&self, player_id: &Uuid) -> Option<&PlayerGameState> {
        self.players.get(player_id)
    }

    pub fn get_player_mut(&mut self, player_id: &Uuid) -> Option<&mut PlayerGameState> {
        self.players.get_mut(player_id)
    }

    pub fn update_player_activity(&mut self, player_id: &Uuid) {
        if let Some(player_state) = self.players.get_mut(player_id) {
            player_state.last_seen = chrono::Utc::now();
            player_state.is_online = true;
        }
    }

    pub fn get_current_market(&self, airport_id: &str) -> Option<&Market> {
        self.shared_state.markets.get(airport_id)
    }

    pub fn advance_turn(&mut self) {
        self.shared_state.turn_number += 1;
        self.shared_state.world_time = chrono::Utc::now();

        // TODO: Add event system integration
        // TODO: Update market prices based on global player activity
    }

    #[allow(dead_code)]
    pub fn start_game(&mut self) -> Result<(), String> {
        if self.players.is_empty() {
            return Err("Need at least 1 player to start".to_string());
        }

        self.game_status = GameStatus::InProgress;
        Ok(())
    }

    pub fn is_joinable(&self) -> bool {
        matches!(self.game_status, GameStatus::WaitingForPlayers)
            && self.players.len() < self.max_players
    }
}
