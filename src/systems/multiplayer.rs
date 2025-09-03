use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::{Airport, CargoType, Market, MessageBoard, Player},
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
    pub message_board: MessageBoard,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
            message_board: MessageBoard::new(50), // Keep last 50 messages per airport
        }
    }

    #[allow(dead_code)]
    pub fn find_offline_player_by_name(&self, player_name: &str) -> Option<Uuid> {
        for (player_id, player_state) in &self.players {
            if player_state.player_name == player_name && !player_state.is_online {
                return Some(*player_id);
            }
        }
        None
    }

    pub fn add_player(
        &mut self,
        player_id: Uuid,
        player_name: String,
        starting_airport: Option<String>,
    ) -> Result<Uuid, String> {
        if self.players.values().filter(|p| p.is_online).count() >= self.max_players {
            return Err("Room is full".to_string());
        }

        // Check if player already exists - allow rejoining if offline OR if it's been a while since they were last seen
        let mut rejoining_player_id = None;
        let now = chrono::Utc::now();
        for (existing_id, player_state) in self.players.iter() {
            if player_state.player_name == player_name {
                if player_state.is_online {
                    // Check if player has been inactive for more than 5 seconds (likely a stale connection)
                    let inactive_duration = now.signed_duration_since(player_state.last_seen);
                    if inactive_duration.num_seconds() > 5 {
                        // Player seems to have disconnected without leaving - allow rejoin
                        rejoining_player_id = Some(*existing_id);
                        break;
                    } else {
                        // Player is truly online and active
                        return Err("Player name already taken in this room".to_string());
                    }
                } else {
                    // Player exists but is offline - they can rejoin with the same ID
                    rejoining_player_id = Some(*existing_id);
                    break;
                }
            }
        }

        let actual_player_id;

        if let Some(existing_id) = rejoining_player_id {
            // Update existing offline player to be online
            if let Some(player_state) = self.players.get_mut(&existing_id) {
                player_state.is_online = true;
                player_state.last_seen = now;
                // Note: We don't update joined_at to preserve original join time
            }
            actual_player_id = existing_id;
        } else {
            // Check if the requested player_id is already taken
            if self.players.contains_key(&player_id) {
                return Err("Player already in room".to_string());
            }

            // New player joining
            let starting_airport = starting_airport.unwrap_or_else(|| "JFK".to_string());
            let player = Player::new(5000, &starting_airport, 200, 1000, 15.0);

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
            actual_player_id = player_id;
        }

        Ok(actual_player_id)
    }

    pub fn mark_player_offline(&mut self, player_id: Uuid) -> Result<(), String> {
        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.is_online = false;
            player_state.last_seen = chrono::Utc::now();
            Ok(())
        } else {
            Err("Player not in room".to_string())
        }
    }

    #[allow(dead_code)]
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
            && self.players.values().filter(|p| p.is_online).count() < self.max_players
    }
}
