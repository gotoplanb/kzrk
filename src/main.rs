mod data;
mod models;
mod systems;
mod ui;

use data::{get_default_airports, get_default_cargo_types};
use systems::{GameState, TradingSystem, TradingError};

fn main() {
    println!("=== KZRK Aviation Trading Game ===");
    
    // Initialize game data
    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();
    
    // Create game state
    let mut game_state = GameState::new(airports, cargo_types);
    
    // Basic status display
    println!("Game initialized successfully!");
    println!("Player starting at: {}", game_state.player.current_airport);
    println!("Starting money: ${}", game_state.player.money);
    println!("Starting fuel: {}/{}", game_state.player.fuel, game_state.player.max_fuel);
    println!("Available airports: {}", game_state.airports.len());
    println!("Available cargo types: {}", game_state.cargo_types.len());
    
    if let Some(current_airport) = game_state.get_current_airport() {
        println!("Current location: {}", current_airport.name);
    }
    
    // Test market system
    println!("\n=== Market System Test ===");
    
    if let Some(market) = game_state.get_current_market() {
        println!("Fuel price at {}: ${}", game_state.player.current_airport, market.fuel_price);
        
        println!("\nCargo prices:");
        for (cargo_id, price) in market.get_all_cargo_prices() {
            if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                println!("  {} ({}): ${}", cargo_type.name, cargo_id, price);
            }
        }
    }
    
    // Test trading system
    println!("\n=== Trading System Test ===");
    
    // Try to buy some electronics
    let electronics_id = "electronics";
    let max_buyable = if let Some(market) = game_state.get_current_market() {
        TradingSystem::get_max_buyable_quantity(
            &game_state.player, 
            market, 
            &game_state.cargo_types, 
            electronics_id
        )
    } else { 0 };
    
    println!("Max buyable electronics: {}", max_buyable);
    
    if max_buyable > 0 {
        let buy_quantity = (max_buyable / 4).max(1); // Buy 25% of max or at least 1
        let market_clone = game_state.get_current_market().cloned();
        if let Some(market) = market_clone {
            match TradingSystem::buy_cargo(
                &mut game_state.player,
                &market,
                &game_state.cargo_types,
                electronics_id,
                buy_quantity,
            ) {
                Ok(cost) => {
                    println!("Successfully bought {} electronics for ${}", buy_quantity, cost);
                    println!("Player money: ${}", game_state.player.money);
                    println!("Player cargo weight: {}/{}", 
                        game_state.player.current_cargo_weight(&game_state.cargo_types),
                        game_state.player.max_cargo_weight
                    );
                },
                Err(e) => println!("Failed to buy electronics: {:?}", e),
            }
        }
    }
    
    // Test fuel purchase
    let max_fuel = if let Some(market) = game_state.get_current_market() {
        TradingSystem::get_max_fuel_buyable(&game_state.player, market)
    } else { 0 };
    
    println!("\nMax fuel buyable: {}", max_fuel);
    
    if max_fuel > 10 {
        let market_clone = game_state.get_current_market().cloned();
        if let Some(market) = market_clone {
            match TradingSystem::buy_fuel(&mut game_state.player, &market, 10) {
                Ok(cost) => {
                    println!("Successfully bought 10 fuel for ${}", cost);
                    println!("Player fuel: {}/{}", game_state.player.fuel, game_state.player.max_fuel);
                },
                Err(e) => println!("Failed to buy fuel: {:?}", e),
            }
        }
    }
}
