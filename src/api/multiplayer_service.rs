use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{
    api::models::*,
    data::{airports::get_default_airports, cargo_types::get_default_cargo_types},
    systems::{GameRoom, PlayerSession},
};

pub type GameRooms = Arc<Mutex<HashMap<Uuid, GameRoom>>>;
pub type PlayerSessions = Arc<Mutex<HashMap<Uuid, PlayerSession>>>;

#[derive(Clone)]
pub struct MultiplayerGameService {
    rooms: GameRooms,
    player_sessions: PlayerSessions,
}

impl Default for MultiplayerGameService {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiplayerGameService {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            player_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_room(
        &self,
        name: String,
        host_player_name: String,
        max_players: Option<usize>,
    ) -> Result<CreateRoomResponse, String> {
        let host_player_id = Uuid::new_v4();
        let max_players = max_players.unwrap_or(4);

        if !(1..=8).contains(&max_players) {
            return Err("Max players must be between 1 and 8".to_string());
        }

        let airports = get_default_airports();
        let cargo_types = get_default_cargo_types();

        let room = GameRoom::new(
            name.clone(),
            host_player_id,
            host_player_name.clone(),
            max_players,
            airports,
            cargo_types,
        );

        let room_id = room.id;

        // Store the room
        {
            let mut rooms = self
                .rooms
                .lock()
                .map_err(|_| "Failed to acquire rooms lock")?;
            rooms.insert(room_id, room);
        }

        // Create player session for host
        let player_session = PlayerSession {
            player_id: host_player_id,
            player_name: host_player_name.clone(),
            game_room_id: Some(room_id),
            connected_at: chrono::Utc::now(),
        };

        {
            let mut sessions = self
                .player_sessions
                .lock()
                .map_err(|_| "Failed to acquire sessions lock")?;
            sessions.insert(host_player_id, player_session);
        }

        Ok(CreateRoomResponse {
            room_id,
            room_name: name,
            host_player_id,
            host_player_name,
            max_players,
            current_players: 1,
        })
    }

    pub fn list_rooms(&self) -> Result<Vec<RoomInfo>, String> {
        let rooms = self
            .rooms
            .lock()
            .map_err(|_| "Failed to acquire rooms lock")?;

        let room_list = rooms
            .values()
            .map(|room| {
                let host_player = room
                    .players
                    .get(&room.host_player_id)
                    .map(|p| p.player_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                RoomInfo {
                    id: room.id,
                    name: room.name.clone(),
                    host_player_name: host_player,
                    current_players: room.players.len() as u32,
                    max_players: room.max_players as u32,
                    created_at: room.created_at,
                    game_status: room.game_status.clone(),
                    is_joinable: room.is_joinable(),
                }
            })
            .collect();

        Ok(room_list)
    }

    pub fn join_room(
        &self,
        room_id: Uuid,
        player_name: String,
        starting_airport: Option<String>,
    ) -> Result<JoinRoomResponse, String> {
        let player_id = Uuid::new_v4();

        // Update the room
        {
            let mut rooms = self
                .rooms
                .lock()
                .map_err(|_| "Failed to acquire rooms lock")?;
            let room = rooms.get_mut(&room_id).ok_or("Room not found")?;

            if !room.is_joinable() {
                return Err("Room is not joinable".to_string());
            }

            room.add_player(player_id, player_name.clone(), starting_airport)?;
        }

        // Create player session
        let player_session = PlayerSession {
            player_id,
            player_name: player_name.clone(),
            game_room_id: Some(room_id),
            connected_at: chrono::Utc::now(),
        };

        {
            let mut sessions = self
                .player_sessions
                .lock()
                .map_err(|_| "Failed to acquire sessions lock")?;
            sessions.insert(player_id, player_session);
        }

        Ok(JoinRoomResponse {
            room_id,
            player_id,
            player_name,
            success: true,
            message: "Successfully joined room".to_string(),
        })
    }

    pub fn leave_room(&self, room_id: Uuid, player_id: Uuid) -> Result<LeaveRoomResponse, String> {
        // Remove player from room
        {
            let mut rooms = self
                .rooms
                .lock()
                .map_err(|_| "Failed to acquire rooms lock")?;
            if let Some(room) = rooms.get_mut(&room_id) {
                room.remove_player(player_id)?;

                // If room is empty, remove it
                if room.players.is_empty() {
                    rooms.remove(&room_id);
                }
            }
        }

        // Update player session
        {
            let mut sessions = self
                .player_sessions
                .lock()
                .map_err(|_| "Failed to acquire sessions lock")?;
            if let Some(session) = sessions.get_mut(&player_id) {
                session.game_room_id = None;
            }
        }

        Ok(LeaveRoomResponse {
            success: true,
            message: "Successfully left room".to_string(),
        })
    }

    pub fn get_room_state(
        &self,
        room_id: Uuid,
        requesting_player_id: Uuid,
    ) -> Result<MultiplayerGameStateResponse, String> {
        let rooms = self
            .rooms
            .lock()
            .map_err(|_| "Failed to acquire rooms lock")?;
        let _room = rooms.get(&room_id).ok_or("Room not found")?;

        // Update player activity
        drop(rooms);
        {
            let mut rooms = self
                .rooms
                .lock()
                .map_err(|_| "Failed to acquire rooms lock")?;
            if let Some(room) = rooms.get_mut(&room_id) {
                room.update_player_activity(&requesting_player_id);
            }
        }
        let rooms = self
            .rooms
            .lock()
            .map_err(|_| "Failed to acquire rooms lock")?;
        let room = rooms.get(&room_id).ok_or("Room not found")?;

        // Verify player is in room
        if !room.players.contains_key(&requesting_player_id) {
            return Err("Player not in room".to_string());
        }

        self.build_multiplayer_game_state_response(room, requesting_player_id)
    }

    pub fn player_travel(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        destination: String,
    ) -> Result<PlayerTravelResponse, String> {
        let mut rooms = self
            .rooms
            .lock()
            .map_err(|_| "Failed to acquire rooms lock")?;
        let room = rooms.get_mut(&room_id).ok_or("Room not found")?;

        // Get necessary information before mutable borrows
        let destination_airport_name = room
            .shared_state
            .airports
            .get(&destination)
            .ok_or("Destination airport not found")?
            .name
            .clone();

        let (distance, fuel_required) = {
            let player_state = room
                .get_player(&player_id)
                .ok_or("Player not found in room")?;
            let destination_airport = room
                .shared_state
                .airports
                .get(&destination)
                .ok_or("Destination airport not found")?;
            let current_airport = room
                .shared_state
                .airports
                .get(&player_state.player.current_airport)
                .ok_or("Current airport not found")?;

            let distance = current_airport.distance_to(destination_airport);
            let fuel_required = player_state.player.fuel_needed_for_distance(distance);
            (distance, fuel_required)
        };

        // Check if travel is possible
        let can_travel = {
            let player_state = room
                .get_player(&player_id)
                .ok_or("Player not found in room")?;
            player_state.player.can_travel_distance(distance)
        };

        if !can_travel {
            let current_fuel = room.get_player(&player_id).unwrap().player.fuel;
            return Ok(PlayerTravelResponse {
                success: false,
                message: format!(
                    "Insufficient fuel. Need {} units, have {}",
                    fuel_required, current_fuel
                ),
                fuel_consumed: None,
                new_location: None,
            });
        }

        // Perform travel
        {
            let player_state = room
                .get_player_mut(&player_id)
                .ok_or("Player not found in room")?;
            player_state.player.consume_fuel(fuel_required);
            player_state.player.current_airport = destination.clone();
        }

        // Update statistics
        if let Some(stats) = room.player_statistics.get_mut(&player_id) {
            stats.record_travel(&destination, distance);
        }

        // Advance turn and potentially generate events
        room.advance_turn();

        Ok(PlayerTravelResponse {
            success: true,
            message: format!("Traveled to {} ({})", destination_airport_name, destination),
            fuel_consumed: Some(fuel_required),
            new_location: Some(destination),
        })
    }

    pub fn player_trade(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        request: TradeRequest,
    ) -> Result<PlayerTradeResponse, String> {
        let mut rooms = self
            .rooms
            .lock()
            .map_err(|_| "Failed to acquire rooms lock")?;
        let room = rooms.get_mut(&room_id).ok_or("Room not found")?;

        // Get trade information before mutable borrows
        let (
            _cargo_price,
            transaction_amount,
            can_afford,
            cargo_weight_per_unit,
            current_cargo_quantity,
        ) = {
            let player_state = room
                .get_player(&player_id)
                .ok_or("Player not found in room")?;
            let current_market = room
                .get_current_market(&player_state.player.current_airport)
                .ok_or("No market available at current location")?;

            let cargo_price = current_market
                .get_cargo_price(&request.cargo_type)
                .ok_or("Cargo type not available at this market")?;
            let transaction_amount = cargo_price * request.quantity;
            let can_afford = player_state.player.can_afford(transaction_amount);

            let cargo_type = room
                .shared_state
                .cargo_types
                .get(&request.cargo_type)
                .ok_or("Invalid cargo type")?;
            let cargo_weight_per_unit = cargo_type.weight_per_unit;
            let current_cargo_quantity = player_state
                .player
                .cargo_inventory
                .get_quantity(&request.cargo_type);

            (
                cargo_price,
                transaction_amount,
                can_afford,
                cargo_weight_per_unit,
                current_cargo_quantity,
            )
        };

        match request.action {
            TradeAction::Buy => {
                // Check if player can afford
                if !can_afford {
                    return Ok(PlayerTradeResponse {
                        success: false,
                        message: "Insufficient funds".to_string(),
                        transaction_amount: None,
                        new_money: None,
                        new_inventory: None,
                    });
                }

                // Check cargo capacity
                let can_carry = {
                    let player_state = room.get_player(&player_id).unwrap();
                    let additional_weight = cargo_weight_per_unit * request.quantity;
                    player_state
                        .player
                        .can_carry_more_weight(additional_weight, &room.shared_state.cargo_types)
                };

                if !can_carry {
                    return Ok(PlayerTradeResponse {
                        success: false,
                        message: "Insufficient cargo capacity".to_string(),
                        transaction_amount: None,
                        new_money: None,
                        new_inventory: None,
                    });
                }

                // Execute purchase
                let (new_money, new_inventory) = {
                    let player_state = room
                        .get_player_mut(&player_id)
                        .ok_or("Player not found in room")?;
                    player_state.player.spend_money(transaction_amount);
                    player_state
                        .player
                        .cargo_inventory
                        .add_cargo(&request.cargo_type, request.quantity);
                    let new_money = player_state.player.money;
                    let new_inventory = self.build_inventory_map(&player_state.player);
                    (new_money, new_inventory)
                };

                // Update statistics
                if let Some(stats) = room.player_statistics.get_mut(&player_id) {
                    stats.record_cargo_purchase(transaction_amount);
                }

                Ok(PlayerTradeResponse {
                    success: true,
                    message: format!(
                        "Successfully bought {} units of {}",
                        request.quantity, request.cargo_type
                    ),
                    transaction_amount: Some(transaction_amount),
                    new_money: Some(new_money),
                    new_inventory: Some(new_inventory),
                })
            },
            TradeAction::Sell => {
                // Check if player has enough cargo
                if current_cargo_quantity < request.quantity {
                    return Ok(PlayerTradeResponse {
                        success: false,
                        message: "Insufficient cargo to sell".to_string(),
                        transaction_amount: None,
                        new_money: None,
                        new_inventory: None,
                    });
                }

                // Execute sale
                let (new_money, new_inventory) = {
                    let player_state = room
                        .get_player_mut(&player_id)
                        .ok_or("Player not found in room")?;
                    player_state
                        .player
                        .cargo_inventory
                        .remove_cargo(&request.cargo_type, request.quantity);
                    player_state.player.earn_money(transaction_amount);
                    let new_money = player_state.player.money;
                    let new_inventory = self.build_inventory_map(&player_state.player);
                    (new_money, new_inventory)
                };

                // Update statistics
                if let Some(stats) = room.player_statistics.get_mut(&player_id) {
                    stats.record_sale(&request.cargo_type, transaction_amount);
                }

                Ok(PlayerTradeResponse {
                    success: true,
                    message: format!(
                        "Successfully sold {} units of {}",
                        request.quantity, request.cargo_type
                    ),
                    transaction_amount: Some(transaction_amount),
                    new_money: Some(new_money),
                    new_inventory: Some(new_inventory),
                })
            },
        }
    }

    pub fn player_buy_fuel(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        request: FuelRequest,
    ) -> Result<PlayerFuelResponse, String> {
        let mut rooms = self
            .rooms
            .lock()
            .map_err(|_| "Failed to acquire rooms lock")?;
        let room = rooms.get_mut(&room_id).ok_or("Room not found")?;

        // Get fuel cost and check constraints before mutable borrows
        let (fuel_cost, can_afford, space_available) = {
            let player_state = room
                .get_player(&player_id)
                .ok_or("Player not found in room")?;
            let current_market = room
                .get_current_market(&player_state.player.current_airport)
                .ok_or("No market available at current location")?;

            let fuel_cost = current_market.fuel_price * request.quantity;
            let can_afford = player_state.player.can_afford(fuel_cost);
            let space_available = player_state.player.max_fuel - player_state.player.fuel;

            (fuel_cost, can_afford, space_available)
        };

        // Check if player can afford
        if !can_afford {
            return Ok(PlayerFuelResponse {
                success: false,
                message: "Insufficient funds for fuel purchase".to_string(),
                cost: None,
                new_fuel: None,
                new_money: None,
            });
        }

        // Check if fuel tank has capacity
        if request.quantity > space_available {
            return Ok(PlayerFuelResponse {
                success: false,
                message: format!("Fuel tank can only hold {} more units", space_available),
                cost: None,
                new_fuel: None,
                new_money: None,
            });
        }

        // Execute fuel purchase
        let (new_fuel, new_money) = {
            let player_state = room
                .get_player_mut(&player_id)
                .ok_or("Player not found in room")?;
            player_state.player.spend_money(fuel_cost);
            player_state.player.add_fuel(request.quantity);
            (player_state.player.fuel, player_state.player.money)
        };

        // Update statistics
        if let Some(stats) = room.player_statistics.get_mut(&player_id) {
            stats.record_fuel_purchase(request.quantity, fuel_cost);
        }

        Ok(PlayerFuelResponse {
            success: true,
            message: format!(
                "Purchased {} units of fuel for ${}",
                request.quantity, fuel_cost
            ),
            cost: Some(fuel_cost),
            new_fuel: Some(new_fuel),
            new_money: Some(new_money),
        })
    }

    fn build_multiplayer_game_state_response(
        &self,
        room: &GameRoom,
        requesting_player_id: Uuid,
    ) -> Result<MultiplayerGameStateResponse, String> {
        let requesting_player_state = room
            .get_player(&requesting_player_id)
            .ok_or("Requesting player not found")?;

        let current_airport = room
            .shared_state
            .airports
            .get(&requesting_player_state.player.current_airport)
            .ok_or("Current airport not found")?;

        let current_market = room
            .get_current_market(&requesting_player_state.player.current_airport)
            .ok_or("Current market not found")?;

        // Build available destinations
        let mut destinations = Vec::new();
        for (airport_id, airport) in &room.shared_state.airports {
            if airport_id != &requesting_player_state.player.current_airport {
                let distance = current_airport.distance_to(airport);
                let fuel_required = requesting_player_state
                    .player
                    .fuel_needed_for_distance(distance);
                let can_travel = requesting_player_state.player.can_travel_distance(distance);
                let fuel_price = room
                    .shared_state
                    .markets
                    .get(airport_id)
                    .map(|m| m.fuel_price)
                    .unwrap_or(50);

                destinations.push(DestinationInfo {
                    airport_id: airport_id.clone(),
                    airport_name: airport.name.clone(),
                    distance,
                    fuel_required,
                    can_travel,
                    fuel_price,
                });
            }
        }

        // Build player list
        let players = room
            .players
            .values()
            .map(|player_state| PlayerInfo {
                id: Some(player_state.player_id),
                name: player_state.player_name.clone(),
                money: player_state.player.money,
                current_airport: player_state.player.current_airport.clone(),
                fuel: player_state.player.fuel,
                max_fuel: player_state.player.max_fuel,
                cargo_inventory: self.build_inventory_map(&player_state.player),
                cargo_weight: player_state
                    .player
                    .current_cargo_weight(&room.shared_state.cargo_types),
                max_cargo_weight: player_state.player.max_cargo_weight,
                fuel_efficiency: player_state.player.fuel_efficiency,
                is_online: Some(player_state.is_online),
                last_seen: Some(player_state.last_seen),
                is_host: Some(player_state.player_id == room.host_player_id),
            })
            .collect();

        // Get statistics for requesting player
        let statistics = room
            .player_statistics
            .get(&requesting_player_id)
            .map(|stats| StatisticsInfo {
                total_revenue: stats.total_revenue,
                total_expenses: stats.total_expenses,
                net_profit: stats.net_profit,
                cargo_trades: stats.cargo_trades,
                fuel_purchased: stats.fuel_purchased,
                distances_traveled: stats.distances_traveled,
                airports_visited: stats.airports_visited.clone(),
                best_single_trade: stats.best_single_trade,
                most_profitable_cargo: stats.most_profitable_cargo.clone(),
                efficiency_score: stats.efficiency_score,
            })
            .unwrap_or_default();

        Ok(MultiplayerGameStateResponse {
            room_info: RoomInfo {
                id: room.id,
                name: room.name.clone(),
                host_player_name: room
                    .players
                    .get(&room.host_player_id)
                    .map(|p| p.player_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
                current_players: room.players.len() as u32,
                max_players: room.max_players as u32,
                created_at: room.created_at,
                game_status: room.game_status.clone(),
                is_joinable: room.is_joinable(),
            },
            my_player_id: requesting_player_id,
            players,
            current_market: MarketInfo {
                airport_id: current_market.airport_id.clone(),
                airport_name: current_airport.name.clone(),
                fuel_price: current_market.fuel_price,
                cargo_prices: current_market.cargo_prices.clone(),
                last_updated: current_market.last_updated,
            },
            available_destinations: destinations,
            statistics,
            turn_number: room.shared_state.turn_number,
            world_time: room.shared_state.world_time,
        })
    }

    fn build_inventory_map(&self, player: &crate::models::Player) -> HashMap<String, u32> {
        let mut inv = HashMap::new();
        for cargo_id in &[
            "electronics",
            "food",
            "textiles",
            "industrial",
            "luxury",
            "materials",
        ] {
            let qty = player.cargo_inventory.get_quantity(cargo_id);
            if qty > 0 {
                inv.insert(cargo_id.to_string(), qty);
            }
        }
        inv
    }
}

impl Default for StatisticsInfo {
    fn default() -> Self {
        Self {
            total_revenue: 0,
            total_expenses: 0,
            net_profit: 0,
            cargo_trades: 0,
            fuel_purchased: 0,
            distances_traveled: 0.0,
            airports_visited: vec![],
            best_single_trade: 0,
            most_profitable_cargo: String::new(),
            efficiency_score: 0.0,
        }
    }
}
