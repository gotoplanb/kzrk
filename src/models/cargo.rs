use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoType {
    pub id: String,
    pub name: String,
    pub base_price: u32,
    pub weight_per_unit: u32,
    pub volatility: f32,
}

impl CargoType {
    pub fn new(
        id: &str,
        name: &str,
        base_price: u32,
        weight_per_unit: u32,
        volatility: f32,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base_price,
            weight_per_unit,
            volatility,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CargoInventory {
    inventory: HashMap<String, u32>,
}

impl CargoInventory {
    pub fn new() -> Self {
        Self {
            inventory: HashMap::new(),
        }
    }

    pub fn get_quantity(&self, cargo_id: &str) -> u32 {
        self.inventory.get(cargo_id).copied().unwrap_or(0)
    }

    pub fn add_cargo(&mut self, cargo_id: &str, quantity: u32) {
        *self.inventory.entry(cargo_id.to_string()).or_insert(0) += quantity;
    }

    pub fn remove_cargo(&mut self, cargo_id: &str, quantity: u32) -> bool {
        if let Some(current) = self.inventory.get_mut(cargo_id)
            && *current >= quantity
        {
            *current -= quantity;
            if *current == 0 {
                self.inventory.remove(cargo_id);
            }
            return true;
        }
        false
    }

    pub fn total_weight(&self, cargo_types: &HashMap<String, CargoType>) -> u32 {
        self.inventory
            .iter()
            .map(|(cargo_id, quantity)| {
                cargo_types
                    .get(cargo_id)
                    .map(|cargo_type| cargo_type.weight_per_unit * quantity)
                    .unwrap_or(0)
            })
            .sum()
    }

    pub fn get_all_cargo(&self) -> &HashMap<String, u32> {
        &self.inventory
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inventory.is_empty()
    }
}
