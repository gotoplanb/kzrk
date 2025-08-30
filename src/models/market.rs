use std::{collections::HashMap, time::SystemTime};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_creation() {
        let market = Market::new("JFK", 100);
        assert_eq!(market.airport_id, "JFK");
        assert_eq!(market.fuel_price, 100);
        assert!(market.cargo_prices.is_empty());
    }

    #[test]
    fn test_set_and_get_cargo_price() {
        let mut market = Market::new("JFK", 100);
        market.set_cargo_price("electronics", 500);

        assert_eq!(market.get_cargo_price("electronics"), Some(500));
        assert_eq!(market.get_cargo_price("nonexistent"), None);
    }

    #[test]
    fn test_update_fuel_price() {
        let mut market = Market::new("JFK", 100);
        market.update_fuel_price(120);
        assert_eq!(market.fuel_price, 120);
    }

    #[test]
    fn test_get_all_cargo_prices() {
        let mut market = Market::new("JFK", 100);
        market.set_cargo_price("electronics", 500);
        market.set_cargo_price("textiles", 300);

        let all_prices = market.get_all_cargo_prices();
        assert_eq!(all_prices.len(), 2);
        assert_eq!(all_prices.get("electronics"), Some(&500));
        assert_eq!(all_prices.get("textiles"), Some(&300));
    }

    #[test]
    fn test_last_updated_changes() {
        let mut market = Market::new("JFK", 100);
        let initial_time = market.last_updated;

        // Small delay to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        market.set_cargo_price("electronics", 500);
        assert!(market.last_updated > initial_time);

        let second_update_time = market.last_updated;
        std::thread::sleep(std::time::Duration::from_millis(1));

        market.update_fuel_price(120);
        assert!(market.last_updated > second_update_time);
    }
}
