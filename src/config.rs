use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub starting_money: u32,
    pub starting_fuel_percentage: f32,
    pub starting_airport: String,
    pub win_condition_money: u32,
    pub max_fuel: u32,
    pub max_cargo_weight: u32,
    pub fuel_efficiency: f32,
    pub price_volatility_multiplier: f32,
    pub fuel_price_multiplier: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            starting_money: 5000,           // Reduced from 10000 for better difficulty curve
            starting_fuel_percentage: 0.66, // Start with 2/3 tank
            starting_airport: "ORD".to_string(),
            win_condition_money: 100000,
            max_fuel: 150,
            max_cargo_weight: 500,
            fuel_efficiency: 10.0,
            price_volatility_multiplier: 1.0,
            fuel_price_multiplier: 1.0,
        }
    }
}

impl GameConfig {
    pub fn easy() -> Self {
        Self {
            starting_money: 8000,
            starting_fuel_percentage: 1.0, // Full tank
            win_condition_money: 50000,    // Lower win condition
            fuel_price_multiplier: 0.8,    // Cheaper fuel
            ..Self::default()
        }
    }

    pub fn normal() -> Self {
        Self::default()
    }

    pub fn hard() -> Self {
        Self {
            starting_money: 3000,
            starting_fuel_percentage: 0.5,    // Half tank
            win_condition_money: 150000,      // Higher win condition
            price_volatility_multiplier: 1.5, // More volatile prices
            fuel_price_multiplier: 1.3,       // More expensive fuel
            ..Self::default()
        }
    }
}
