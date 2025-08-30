use super::cargo::CargoInventory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub money: u32,
    pub current_airport: String,
    pub fuel: u32,
    pub max_fuel: u32,
    pub cargo_inventory: CargoInventory,
    pub max_cargo_weight: u32,
    pub fuel_efficiency: f32,
}

impl Player {
    pub fn new(
        starting_money: u32,
        starting_airport: &str,
        max_fuel: u32,
        max_cargo_weight: u32,
        fuel_efficiency: f32,
    ) -> Self {
        Self {
            money: starting_money,
            current_airport: starting_airport.to_string(),
            fuel: max_fuel * 2 / 3, // Start with 2/3 tank
            max_fuel,
            cargo_inventory: CargoInventory::new(),
            max_cargo_weight,
            fuel_efficiency,
        }
    }

    pub fn can_afford(&self, cost: u32) -> bool {
        self.money >= cost
    }

    pub fn spend_money(&mut self, amount: u32) -> bool {
        if self.can_afford(amount) {
            self.money -= amount;
            true
        } else {
            false
        }
    }

    pub fn earn_money(&mut self, amount: u32) {
        self.money += amount;
    }

    pub fn can_carry_more_weight(&self, additional_weight: u32, cargo_types: &std::collections::HashMap<String, super::cargo::CargoType>) -> bool {
        let current_weight = self.cargo_inventory.total_weight(cargo_types);
        current_weight + additional_weight <= self.max_cargo_weight
    }

    pub fn consume_fuel(&mut self, amount: u32) -> bool {
        if self.fuel >= amount {
            self.fuel -= amount;
            true
        } else {
            false
        }
    }

    pub fn add_fuel(&mut self, amount: u32) {
        self.fuel = (self.fuel + amount).min(self.max_fuel);
    }

    pub fn fuel_needed_for_distance(&self, distance: f64) -> u32 {
        (distance / self.fuel_efficiency as f64).ceil() as u32
    }

    pub fn can_travel_distance(&self, distance: f64) -> bool {
        let fuel_needed = self.fuel_needed_for_distance(distance);
        self.fuel >= fuel_needed
    }

    pub fn current_cargo_weight(&self, cargo_types: &std::collections::HashMap<String, super::cargo::CargoType>) -> u32 {
        self.cargo_inventory.total_weight(cargo_types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_player() -> Player {
        Player::new(1000, "TEST", 100, 500, 10.0)
    }

    #[test]
    fn test_player_creation() {
        let player = create_test_player();
        assert_eq!(player.money, 1000);
        assert_eq!(player.current_airport, "TEST");
        assert_eq!(player.fuel, 66); // 2/3 of max_fuel (100)
        assert_eq!(player.max_fuel, 100);
        assert_eq!(player.max_cargo_weight, 500);
        assert_eq!(player.fuel_efficiency, 10.0);
    }

    #[test]
    fn test_can_afford() {
        let player = create_test_player();
        assert!(player.can_afford(500));
        assert!(player.can_afford(1000));
        assert!(!player.can_afford(1001));
    }

    #[test]
    fn test_spend_money() {
        let mut player = create_test_player();
        assert!(player.spend_money(500));
        assert_eq!(player.money, 500);
        
        assert!(!player.spend_money(600));
        assert_eq!(player.money, 500); // Money shouldn't change on failed spend
    }

    #[test]
    fn test_earn_money() {
        let mut player = create_test_player();
        player.earn_money(500);
        assert_eq!(player.money, 1500);
    }

    #[test]
    fn test_consume_fuel() {
        let mut player = create_test_player();
        assert!(player.consume_fuel(30));
        assert_eq!(player.fuel, 36);
        
        assert!(!player.consume_fuel(50));
        assert_eq!(player.fuel, 36); // Fuel shouldn't change on failed consume
    }

    #[test]
    fn test_add_fuel() {
        let mut player = create_test_player();
        player.add_fuel(20);
        assert_eq!(player.fuel, 86);
        
        // Test fuel cap
        player.add_fuel(50);
        assert_eq!(player.fuel, 100); // Should cap at max_fuel
    }

    #[test]
    fn test_fuel_calculations() {
        let player = create_test_player();
        
        let fuel_needed = player.fuel_needed_for_distance(100.0);
        assert_eq!(fuel_needed, 10); // 100.0 / 10.0 = 10
        
        assert!(player.can_travel_distance(100.0));
        assert!(player.can_travel_distance(660.0)); // Just within fuel limit
        assert!(!player.can_travel_distance(670.0)); // Beyond fuel limit
    }

    #[test]
    fn test_can_carry_more_weight() {
        let player = create_test_player();
        let cargo_types = HashMap::new(); // Empty cargo types for simplicity
        
        // With empty inventory, should be able to carry up to max weight
        assert!(player.can_carry_more_weight(500, &cargo_types));
        assert!(!player.can_carry_more_weight(501, &cargo_types));
    }
}