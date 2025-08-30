use std::collections::HashMap;

use crate::models::{CargoType, Market, Player};

#[derive(Debug, Clone)]
pub enum TradingError {
    InsufficientFunds,
    InsufficientCargo,
    InsufficientCapacity,
    CargoNotAvailable,
    InvalidQuantity,
}

pub struct TradingSystem;

impl TradingSystem {
    pub fn buy_cargo(
        player: &mut Player,
        market: &Market,
        cargo_types: &HashMap<String, CargoType>,
        cargo_id: &str,
        quantity: u32,
    ) -> Result<u32, TradingError> {
        if quantity == 0 {
            return Err(TradingError::InvalidQuantity);
        }

        // Check if cargo type exists and is available in market
        let cargo_type = cargo_types
            .get(cargo_id)
            .ok_or(TradingError::CargoNotAvailable)?;
        let unit_price = market
            .get_cargo_price(cargo_id)
            .ok_or(TradingError::CargoNotAvailable)?;

        let total_cost = unit_price * quantity;
        let total_weight = cargo_type.weight_per_unit * quantity;

        // Check if player can afford it
        if !player.can_afford(total_cost) {
            return Err(TradingError::InsufficientFunds);
        }

        // Check if player can carry the weight
        if !player.can_carry_more_weight(total_weight, cargo_types) {
            return Err(TradingError::InsufficientCapacity);
        }

        // Execute the purchase
        if player.spend_money(total_cost) {
            player.cargo_inventory.add_cargo(cargo_id, quantity);
            Ok(total_cost)
        } else {
            Err(TradingError::InsufficientFunds)
        }
    }

    pub fn sell_cargo(
        player: &mut Player,
        market: &Market,
        cargo_id: &str,
        quantity: u32,
    ) -> Result<u32, TradingError> {
        if quantity == 0 {
            return Err(TradingError::InvalidQuantity);
        }

        // Check if player has enough cargo
        let player_quantity = player.cargo_inventory.get_quantity(cargo_id);
        if player_quantity < quantity {
            return Err(TradingError::InsufficientCargo);
        }

        // Check if market has a price for this cargo
        let unit_price = market
            .get_cargo_price(cargo_id)
            .ok_or(TradingError::CargoNotAvailable)?;

        let total_revenue = unit_price * quantity;

        // Execute the sale
        if player.cargo_inventory.remove_cargo(cargo_id, quantity) {
            player.earn_money(total_revenue);
            Ok(total_revenue)
        } else {
            Err(TradingError::InsufficientCargo)
        }
    }

    pub fn buy_fuel(
        player: &mut Player,
        market: &Market,
        quantity: u32,
    ) -> Result<u32, TradingError> {
        if quantity == 0 {
            return Err(TradingError::InvalidQuantity);
        }

        let unit_price = market.fuel_price;
        let total_cost = unit_price * quantity;

        // Check if player can afford it
        if !player.can_afford(total_cost) {
            return Err(TradingError::InsufficientFunds);
        }

        // Check if fuel tank has capacity
        let available_capacity = player.max_fuel - player.fuel;
        let quantity_to_buy = quantity.min(available_capacity);

        if quantity_to_buy == 0 {
            return Err(TradingError::InsufficientCapacity);
        }

        let actual_cost = unit_price * quantity_to_buy;

        // Execute the purchase
        if player.spend_money(actual_cost) {
            player.add_fuel(quantity_to_buy);
            Ok(actual_cost)
        } else {
            Err(TradingError::InsufficientFunds)
        }
    }

    pub fn get_max_buyable_quantity(
        player: &Player,
        market: &Market,
        cargo_types: &HashMap<String, CargoType>,
        cargo_id: &str,
    ) -> u32 {
        let cargo_type = match cargo_types.get(cargo_id) {
            Some(ct) => ct,
            None => return 0,
        };

        let unit_price = match market.get_cargo_price(cargo_id) {
            Some(price) => price,
            None => return 0,
        };

        if unit_price == 0 {
            return 0;
        }

        // Calculate maximum based on money
        let max_by_money = player.money / unit_price;

        // Calculate maximum based on weight capacity
        let current_weight = player.current_cargo_weight(cargo_types);
        let available_weight = player.max_cargo_weight.saturating_sub(current_weight);
        let max_by_weight = if cargo_type.weight_per_unit > 0 {
            available_weight / cargo_type.weight_per_unit
        } else {
            max_by_money // If weight is 0, no weight constraint
        };

        max_by_money.min(max_by_weight)
    }

    pub fn get_max_fuel_buyable(player: &Player, market: &Market) -> u32 {
        let unit_price = market.fuel_price;
        if unit_price == 0 {
            return 0;
        }

        let max_by_money = player.money / unit_price;
        let max_by_capacity = player.max_fuel - player.fuel;

        max_by_money.min(max_by_capacity)
    }
}
