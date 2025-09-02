#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{
    api::models::*,
    data::{airports::get_default_airports, cargo_types::get_default_cargo_types},
    models::Player,
    systems::{GameState, GameStatistics},
};

pub type GameSessions = Arc<Mutex<HashMap<Uuid, GameState>>>;
pub type GameStatsStorage = Arc<Mutex<HashMap<Uuid, GameStatistics>>>;

#[derive(Clone)]
pub struct GameService {
    sessions: GameSessions,
    statistics: GameStatsStorage,
}

impl Default for GameService {
    fn default() -> Self {
        Self::new()
    }
}

impl GameService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            statistics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_game(&self, request: CreateGameRequest) -> Result<CreateGameResponse, String> {
        let session_id = Uuid::new_v4();

        let starting_money = request.starting_money.unwrap_or(5000);
        let starting_airport = request
            .starting_airport
            .unwrap_or_else(|| "JFK".to_string());

        let airports = get_default_airports();
        let cargo_types = get_default_cargo_types();

        let mut game_state = GameState::new(airports, cargo_types);
        game_state.player = Player::new(starting_money, &starting_airport, 200, 1000, 15.0);

        let game_state_response = self.build_game_state_response(&game_state, session_id)?;

        // Store the game state
        {
            let mut sessions = self
                .sessions
                .lock()
                .map_err(|_| "Failed to acquire session lock")?;
            sessions.insert(session_id, game_state);
        }

        // Initialize statistics
        {
            let mut stats = self
                .statistics
                .lock()
                .map_err(|_| "Failed to acquire statistics lock")?;
            stats.insert(session_id, GameStatistics::new());
        }

        Ok(CreateGameResponse {
            session_id,
            player_name: request.player_name,
            game_state: game_state_response,
        })
    }

    pub fn get_game_state(&self, session_id: Uuid) -> Result<GameStateResponse, String> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| "Failed to acquire session lock")?;
        let game_state = sessions.get(&session_id).ok_or("Game session not found")?;

        self.build_game_state_response(game_state, session_id)
    }

    pub fn travel(
        &self,
        session_id: Uuid,
        request: TravelRequest,
    ) -> Result<TravelResponse, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Failed to acquire session lock")?;
        let game_state = sessions
            .get_mut(&session_id)
            .ok_or("Game session not found")?;

        // Get destination airport
        let destination_airport = game_state
            .airports
            .get(&request.destination)
            .ok_or("Destination airport not found")?;

        // Calculate distance and fuel required
        let current_airport = game_state
            .airports
            .get(&game_state.player.current_airport)
            .ok_or("Current airport not found")?;

        let distance = current_airport.distance_to(destination_airport);
        let fuel_required = game_state.player.fuel_needed_for_distance(distance);

        // Check if travel is possible
        if !game_state.player.can_travel_distance(distance) {
            return Ok(TravelResponse {
                success: false,
                message: format!(
                    "Insufficient fuel. Need {} units, have {}",
                    fuel_required, game_state.player.fuel
                ),
                fuel_consumed: None,
                new_location: None,
                game_state: None,
            });
        }

        // Get destination name for response message
        let destination_name = destination_airport.name.clone();

        // Perform travel
        game_state.player.consume_fuel(fuel_required);
        game_state.player.current_airport = request.destination.clone();

        // Update statistics
        {
            let mut stats = self
                .statistics
                .lock()
                .map_err(|_| "Failed to acquire statistics lock")?;
            if let Some(game_stats) = stats.get_mut(&session_id) {
                game_stats.record_travel(&request.destination, distance);
            }
        }

        // Advance turn and potentially generate events
        self.advance_turn(game_state);

        let new_game_state = self.build_game_state_response(game_state, session_id)?;

        Ok(TravelResponse {
            success: true,
            message: format!("Traveled to {} ({})", destination_name, request.destination),
            fuel_consumed: Some(fuel_required),
            new_location: Some(request.destination),
            game_state: Some(new_game_state),
        })
    }

    pub fn trade(&self, session_id: Uuid, request: TradeRequest) -> Result<TradeResponse, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Failed to acquire session lock")?;
        let game_state = sessions
            .get_mut(&session_id)
            .ok_or("Game session not found")?;

        let current_market = game_state
            .get_current_market()
            .ok_or("No market available at current location")?;

        let cargo_price = current_market
            .get_cargo_price(&request.cargo_type)
            .ok_or("Cargo type not available at this market")?;

        let transaction_amount = cargo_price * request.quantity;

        match request.action {
            TradeAction::Buy => {
                // Check if player can afford
                if !game_state.player.can_afford(transaction_amount) {
                    return Ok(TradeResponse {
                        success: false,
                        message: "Insufficient funds".to_string(),
                        transaction_amount: None,
                        new_money: None,
                        new_inventory: None,
                        game_state: None,
                    });
                }

                // Check cargo capacity
                let cargo_type = game_state
                    .cargo_types
                    .get(&request.cargo_type)
                    .ok_or("Invalid cargo type")?;
                let additional_weight = cargo_type.weight_per_unit * request.quantity;

                if !game_state
                    .player
                    .can_carry_more_weight(additional_weight, &game_state.cargo_types)
                {
                    return Ok(TradeResponse {
                        success: false,
                        message: "Insufficient cargo capacity".to_string(),
                        transaction_amount: None,
                        new_money: None,
                        new_inventory: None,
                        game_state: None,
                    });
                }

                // Execute purchase
                game_state.player.spend_money(transaction_amount);
                game_state
                    .player
                    .cargo_inventory
                    .add_cargo(&request.cargo_type, request.quantity);

                // Update statistics
                {
                    let mut stats = self
                        .statistics
                        .lock()
                        .map_err(|_| "Failed to acquire statistics lock")?;
                    if let Some(game_stats) = stats.get_mut(&session_id) {
                        game_stats.record_cargo_purchase(transaction_amount);
                    }
                }
            },
            TradeAction::Sell => {
                // Check if player has enough cargo
                if game_state
                    .player
                    .cargo_inventory
                    .get_quantity(&request.cargo_type)
                    < request.quantity
                {
                    return Ok(TradeResponse {
                        success: false,
                        message: "Insufficient cargo to sell".to_string(),
                        transaction_amount: None,
                        new_money: None,
                        new_inventory: None,
                        game_state: None,
                    });
                }

                // Execute sale
                game_state
                    .player
                    .cargo_inventory
                    .remove_cargo(&request.cargo_type, request.quantity);
                game_state.player.earn_money(transaction_amount);

                // Update statistics
                {
                    let mut stats = self
                        .statistics
                        .lock()
                        .map_err(|_| "Failed to acquire statistics lock")?;
                    if let Some(game_stats) = stats.get_mut(&session_id) {
                        game_stats.record_sale(&request.cargo_type, transaction_amount);
                    }
                }
            },
        }

        let new_inventory = {
            let mut inv = HashMap::new();
            for cargo_id in &[
                "electronics",
                "food",
                "textiles",
                "industrial",
                "luxury",
                "materials",
            ] {
                let qty = game_state.player.cargo_inventory.get_quantity(cargo_id);
                if qty > 0 {
                    inv.insert(cargo_id.to_string(), qty);
                }
            }
            inv
        };
        let new_game_state = self.build_game_state_response(game_state, session_id)?;

        Ok(TradeResponse {
            success: true,
            message: format!(
                "Successfully {:?}ed {} units of {}",
                request.action, request.quantity, request.cargo_type
            ),
            transaction_amount: Some(transaction_amount),
            new_money: Some(game_state.player.money),
            new_inventory: Some(new_inventory),
            game_state: Some(new_game_state),
        })
    }

    pub fn buy_fuel(&self, session_id: Uuid, request: FuelRequest) -> Result<FuelResponse, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "Failed to acquire session lock")?;
        let game_state = sessions
            .get_mut(&session_id)
            .ok_or("Game session not found")?;

        let current_market = game_state
            .get_current_market()
            .ok_or("No market available at current location")?;

        let fuel_cost = current_market.fuel_price * request.quantity;

        // Check if player can afford
        if !game_state.player.can_afford(fuel_cost) {
            return Ok(FuelResponse {
                success: false,
                message: "Insufficient funds for fuel purchase".to_string(),
                cost: None,
                new_fuel: None,
                new_money: None,
                game_state: None,
            });
        }

        // Check if fuel tank has capacity
        let space_available = game_state.player.max_fuel - game_state.player.fuel;
        if request.quantity > space_available {
            return Ok(FuelResponse {
                success: false,
                message: format!("Fuel tank can only hold {} more units", space_available),
                cost: None,
                new_fuel: None,
                new_money: None,
                game_state: None,
            });
        }

        // Execute fuel purchase
        game_state.player.spend_money(fuel_cost);
        game_state.player.add_fuel(request.quantity);

        // Update statistics
        {
            let mut stats = self
                .statistics
                .lock()
                .map_err(|_| "Failed to acquire statistics lock")?;
            if let Some(game_stats) = stats.get_mut(&session_id) {
                game_stats.record_fuel_purchase(request.quantity, fuel_cost);
            }
        }

        let new_game_state = self.build_game_state_response(game_state, session_id)?;

        Ok(FuelResponse {
            success: true,
            message: format!(
                "Purchased {} units of fuel for ${}",
                request.quantity, fuel_cost
            ),
            cost: Some(fuel_cost),
            new_fuel: Some(game_state.player.fuel),
            new_money: Some(game_state.player.money),
            game_state: Some(new_game_state),
        })
    }

    fn build_game_state_response(
        &self,
        game_state: &GameState,
        session_id: Uuid,
    ) -> Result<GameStateResponse, String> {
        let current_airport = game_state
            .airports
            .get(&game_state.player.current_airport)
            .ok_or("Current airport not found")?;

        let current_market = game_state
            .get_current_market()
            .ok_or("Current market not found")?;

        // Build available destinations
        let mut destinations = Vec::new();
        for (airport_id, airport) in &game_state.airports {
            if airport_id != &game_state.player.current_airport {
                let distance = current_airport.distance_to(airport);
                let fuel_required = game_state.player.fuel_needed_for_distance(distance);
                let can_travel = game_state.player.can_travel_distance(distance);
                let fuel_price = game_state
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

        // Build active events - for now, return empty since we haven't added events to GameState
        let active_events: Vec<EventInfo> = vec![];

        // Get statistics
        let statistics = {
            let stats = self
                .statistics
                .lock()
                .map_err(|_| "Failed to acquire statistics lock")?;
            if let Some(game_stats) = stats.get(&session_id) {
                StatisticsInfo {
                    total_revenue: game_stats.total_revenue,
                    total_expenses: game_stats.total_expenses,
                    net_profit: game_stats.net_profit,
                    cargo_trades: game_stats.cargo_trades,
                    fuel_purchased: game_stats.fuel_purchased,
                    distances_traveled: game_stats.distances_traveled,
                    airports_visited: game_stats.airports_visited.clone(),
                    best_single_trade: game_stats.best_single_trade,
                    most_profitable_cargo: game_stats.most_profitable_cargo.clone(),
                    efficiency_score: game_stats.efficiency_score,
                }
            } else {
                StatisticsInfo {
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
        };

        Ok(GameStateResponse {
            player: PlayerInfo {
                id: None,
                name: "Player".to_string(), // TODO: Get actual player name
                money: game_state.player.money,
                current_airport: game_state.player.current_airport.clone(),
                fuel: game_state.player.fuel,
                max_fuel: game_state.player.max_fuel,
                cargo_inventory: {
                    let mut inv = HashMap::new();
                    for cargo_id in &[
                        "electronics",
                        "food",
                        "textiles",
                        "industrial",
                        "luxury",
                        "materials",
                    ] {
                        let qty = game_state.player.cargo_inventory.get_quantity(cargo_id);
                        if qty > 0 {
                            inv.insert(cargo_id.to_string(), qty);
                        }
                    }
                    inv
                },
                cargo_weight: game_state
                    .player
                    .current_cargo_weight(&game_state.cargo_types),
                max_cargo_weight: game_state.player.max_cargo_weight,
                fuel_efficiency: game_state.player.fuel_efficiency,
                is_online: None,
                last_seen: None,
                is_host: None,
            },
            current_market: MarketInfo {
                airport_id: current_market.airport_id.clone(),
                airport_name: current_airport.name.clone(),
                fuel_price: current_market.fuel_price,
                cargo_prices: current_market.cargo_prices.clone(),
                last_updated: current_market.last_updated,
            },
            available_destinations: destinations,
            active_events,
            statistics,
            turn_number: game_state.turn_number,
        })
    }

    fn advance_turn(&self, game_state: &mut GameState) {
        game_state.turn_number += 1;

        // For now, just advance the turn - we can add events later
        // TODO: Add event system integration
    }
}
