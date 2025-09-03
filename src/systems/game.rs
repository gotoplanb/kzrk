use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    config::GameConfig,
    models::{Airport, CargoType, GameStats, Market, MessageBoard, Player},
    systems::{
        MarketSystem,
        events::{EventSystem, MarketEvent},
    },
};

// Use a string key for JSON serialization compatibility
pub type DistanceCache = HashMap<String, f64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub airports: HashMap<String, Airport>,
    pub cargo_types: HashMap<String, CargoType>,
    pub markets: HashMap<String, Market>,
    pub distance_cache: DistanceCache,
    pub turn_number: u32,
    pub cheat_mode: bool,
    pub stats: GameStats,
    pub win_condition_money: u32,
    pub active_events: Vec<MarketEvent>,
    pub message_board: MessageBoard,
}

impl GameState {
    pub fn new(
        airports: HashMap<String, Airport>,
        cargo_types: HashMap<String, CargoType>,
    ) -> Self {
        Self::new_with_config(airports, cargo_types, GameConfig::default())
    }

    pub fn new_with_config(
        airports: HashMap<String, Airport>,
        cargo_types: HashMap<String, CargoType>,
        config: GameConfig,
    ) -> Self {
        // Check for cheat mode via environment variable
        let cheat_mode = std::env::var("KZRK_CHEAT")
            .map(|v| v.to_lowercase())
            .map(|v| v == "1" || v == "true" || v == "on")
            .unwrap_or(false);

        let mut game_state = Self {
            player: Player::new(
                config.starting_money,
                &config.starting_airport,
                config.max_fuel,
                config.max_cargo_weight,
                config.fuel_efficiency,
            ),
            airports: airports.clone(),
            cargo_types,
            markets: HashMap::new(),
            distance_cache: HashMap::new(),
            turn_number: 1,
            cheat_mode,
            stats: GameStats::new(config.starting_money),
            win_condition_money: config.win_condition_money,
            active_events: Vec::new(),
            message_board: MessageBoard::new(50),
        };

        // Initialize starting airport in stats
        game_state
            .stats
            .airports_visited
            .push(config.starting_airport.clone());

        // Apply starting fuel percentage
        game_state.player.fuel = (config.max_fuel as f32 * config.starting_fuel_percentage) as u32;

        // Pre-calculate all distances and initialize markets
        game_state.initialize_distance_cache();
        game_state.initialize_markets();

        game_state
    }

    fn initialize_distance_cache(&mut self) {
        let airport_ids: Vec<String> = self.airports.keys().cloned().collect();

        for i in 0..airport_ids.len() {
            for j in i..airport_ids.len() {
                let id1 = &airport_ids[i];
                let id2 = &airport_ids[j];

                if i == j {
                    let key = format!("{}-{}", id1, id2);
                    self.distance_cache.insert(key, 0.0);
                } else {
                    let airport1 = &self.airports[id1];
                    let airport2 = &self.airports[id2];
                    let distance = airport1.distance_to(airport2);

                    // Store both directions
                    let key1 = format!("{}-{}", id1, id2);
                    let key2 = format!("{}-{}", id2, id1);
                    self.distance_cache.insert(key1, distance);
                    self.distance_cache.insert(key2, distance);
                }
            }
        }
    }

    fn initialize_markets(&mut self) {
        let mut rng = rand::thread_rng();
        self.markets =
            MarketSystem::initialize_all_markets(&self.airports, &self.cargo_types, &mut rng);
    }

    #[allow(dead_code)]
    pub fn get_distance(&self, from: &str, to: &str) -> Option<f64> {
        let key = format!("{}-{}", from, to);
        self.distance_cache.get(&key).copied()
    }

    pub fn get_current_airport(&self) -> Option<&Airport> {
        self.airports.get(&self.player.current_airport)
    }

    pub fn get_current_market(&self) -> Option<&Market> {
        self.markets.get(&self.player.current_airport)
    }

    #[allow(dead_code)]
    pub fn get_current_market_mut(&mut self) -> Option<&mut Market> {
        self.markets.get_mut(&self.player.current_airport)
    }

    pub fn get_available_destinations(&self) -> Vec<&Airport> {
        self.airports
            .values()
            .filter(|airport| airport.id != self.player.current_airport)
            .collect()
    }

    pub fn advance_turn(&mut self) {
        self.turn_number += 1;

        // Process market events
        self.process_market_events();

        // Chance to generate new event
        self.maybe_generate_event();
    }

    fn process_market_events(&mut self) {
        // Update event durations
        let expired_messages = EventSystem::update_events(&mut self.active_events);

        // Re-apply active events to markets
        for event in &self.active_events {
            if let Some(market) = self.markets.get_mut(&event.affected_airport) {
                EventSystem::apply_event_to_market(event, market);
            }
        }

        // Could store expired messages for display if needed
        for _ in expired_messages {
            // Events expired silently for now
        }
    }

    fn maybe_generate_event(&mut self) {
        let mut rng = rand::thread_rng();
        if let Some(new_event) =
            EventSystem::generate_random_event(&self.airports, &self.cargo_types, &mut rng)
        {
            // Apply the event to the affected market immediately
            if let Some(market) = self.markets.get_mut(&new_event.affected_airport) {
                EventSystem::apply_event_to_market(&new_event, market);
            }

            self.active_events.push(new_event);
        }
    }

    pub fn is_game_won(&self) -> bool {
        self.player.money >= self.win_condition_money
    }

    pub fn can_player_continue(&self) -> bool {
        // Player can continue if they have fuel or money to buy fuel
        if self.player.fuel > 0 {
            return true;
        }

        if let Some(market) = self.get_current_market() {
            self.player.can_afford(market.fuel_price)
        } else {
            false
        }
    }

    pub fn refresh_current_market(&mut self) {
        let current_airport_id = self.player.current_airport.clone();
        if let Some(airport) = self.airports.get(&current_airport_id)
            && let Some(market) = self.markets.get_mut(&current_airport_id)
        {
            let mut rng = rand::thread_rng();
            MarketSystem::update_market_prices(market, airport, &self.cargo_types, &mut rng);
        }
    }

    #[allow(dead_code)]
    pub fn refresh_all_markets(&mut self) {
        let mut rng = rand::thread_rng();
        for (airport_id, market) in self.markets.iter_mut() {
            if let Some(airport) = self.airports.get(airport_id) {
                MarketSystem::update_market_prices(market, airport, &self.cargo_types, &mut rng);
            }
        }
    }
}
