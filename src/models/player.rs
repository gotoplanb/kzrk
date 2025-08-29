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
            fuel: max_fuel, // Start with full tank
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