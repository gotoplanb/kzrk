use crate::models::{Airport, CargoType, Market, Player};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub airports: HashMap<String, Airport>,
    pub cargo_types: HashMap<String, CargoType>,
    pub markets: HashMap<String, Market>,
    pub distance_cache: HashMap<(String, String), f64>,
    pub turn_number: u32,
}

impl GameState {
    pub fn new(
        airports: HashMap<String, Airport>,
        cargo_types: HashMap<String, CargoType>,
    ) -> Self {
        let mut game_state = Self {
            player: Player::new(10000, "ORD", 100, 500, 10.0), // $10k, ORD, 100 fuel, 500kg cargo, 10km per fuel
            airports: airports.clone(),
            cargo_types,
            markets: HashMap::new(),
            distance_cache: HashMap::new(),
            turn_number: 1,
        };

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
                    self.distance_cache.insert((id1.clone(), id2.clone()), 0.0);
                } else {
                    let airport1 = &self.airports[id1];
                    let airport2 = &self.airports[id2];
                    let distance = airport1.distance_to(airport2);
                    
                    // Store both directions
                    self.distance_cache.insert((id1.clone(), id2.clone()), distance);
                    self.distance_cache.insert((id2.clone(), id1.clone()), distance);
                }
            }
        }
    }

    fn initialize_markets(&mut self) {
        for airport_id in self.airports.keys() {
            let airport = &self.airports[airport_id];
            let fuel_price = (airport.base_fuel_price as f32 * airport.market_profile.fuel_modifier) as u32;
            self.markets.insert(airport_id.clone(), Market::new(airport_id, fuel_price));
        }
    }

    pub fn get_distance(&self, from: &str, to: &str) -> Option<f64> {
        self.distance_cache.get(&(from.to_string(), to.to_string())).copied()
    }

    pub fn get_current_airport(&self) -> Option<&Airport> {
        self.airports.get(&self.player.current_airport)
    }

    pub fn get_current_market(&self) -> Option<&Market> {
        self.markets.get(&self.player.current_airport)
    }

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
    }

    pub fn is_game_won(&self) -> bool {
        self.player.money >= 100000 // Win condition: $100k
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
}