use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub airport_id: String,
    pub fuel_price: u32,
    pub cargo_prices: HashMap<String, u32>,
    pub last_updated: SystemTime,
}

impl Market {
    pub fn new(airport_id: &str, fuel_price: u32) -> Self {
        Self {
            airport_id: airport_id.to_string(),
            fuel_price,
            cargo_prices: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }

    pub fn set_cargo_price(&mut self, cargo_id: &str, price: u32) {
        self.cargo_prices.insert(cargo_id.to_string(), price);
        self.last_updated = SystemTime::now();
    }

    pub fn get_cargo_price(&self, cargo_id: &str) -> Option<u32> {
        self.cargo_prices.get(cargo_id).copied()
    }

    pub fn update_fuel_price(&mut self, new_price: u32) {
        self.fuel_price = new_price;
        self.last_updated = SystemTime::now();
    }

    pub fn get_all_cargo_prices(&self) -> &HashMap<String, u32> {
        &self.cargo_prices
    }
}