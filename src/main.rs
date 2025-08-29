mod data;
mod models;
mod systems;
mod ui;

use data::{get_default_airports, get_default_cargo_types};
use systems::{GameState, TradingSystem, TravelSystem};

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
    
    // Test travel system
    println!("\n=== Travel System Test ===");
    
    // Show available destinations
    let destinations = TravelSystem::get_reachable_destinations(&game_state);
    println!("Available destinations from {}:", game_state.player.current_airport);
    
    for dest in &destinations {
        let affordable_indicator = if dest.can_afford { "✓" } else { "✗" };
        println!("  {} {} - {:.0}km, {} fuel needed {}",
            affordable_indicator,
            dest.airport_id,
            dest.distance_km,
            dest.fuel_needed,
            dest.airport_name
        );
    }
    
    // First, try to buy fuel if needed to reach closest destination
    if let Some(closest) = destinations.first() {
        if !closest.can_afford {
            let fuel_needed = closest.fuel_needed.saturating_sub(game_state.player.fuel);
            println!("\nNeed {} more fuel to reach {}. Attempting to buy fuel...", fuel_needed, closest.airport_name);
            
            let market_clone = game_state.get_current_market().cloned();
            if let Some(market) = market_clone {
                match TradingSystem::buy_fuel(&mut game_state.player, &market, fuel_needed) {
                    Ok(cost) => {
                        println!("✓ Bought {} fuel for ${}", fuel_needed, cost);
                        println!("Player fuel: {}/{}, money: ${}", game_state.player.fuel, game_state.player.max_fuel, game_state.player.money);
                    },
                    Err(e) => println!("✗ Failed to buy fuel: {:?}", e),
                }
            }
        }
    }
    
    // Recalculate destinations with new fuel level
    let destinations = TravelSystem::get_reachable_destinations(&game_state);
    
    // Try to travel to the closest affordable destination
    if let Some(destination) = destinations.iter().find(|d| d.can_afford) {
        println!("\nAttempting to travel to {} ({})...", destination.airport_name, destination.airport_id);
        
        match TravelSystem::travel_to(&mut game_state, &destination.airport_id) {
            Ok(travel_info) => {
                println!("✓ Travel successful!");
                println!("  Route: {} → {}", travel_info.from, travel_info.to);
                println!("  Distance: {:.0} km", travel_info.distance_km);
                println!("  Fuel consumed: {}", travel_info.fuel_consumed);
                println!("  Remaining fuel: {}/{}", travel_info.remaining_fuel, game_state.player.max_fuel);
                println!("  Current turn: {}", game_state.turn_number);
                
                // Show new market prices
                if let Some(new_market) = game_state.get_current_market() {
                    println!("\n  New market prices at {}:", game_state.player.current_airport);
                    println!("    Fuel: ${}", new_market.fuel_price);
                    
                    // Show a few cargo prices
                    let mut count = 0;
                    for (cargo_id, price) in new_market.get_all_cargo_prices() {
                        if count < 3 {
                            if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                                println!("    {}: ${}", cargo_type.name, price);
                                count += 1;
                            }
                        }
                    }
                }
            },
            Err(e) => println!("✗ Travel failed: {:?}", e),
        }
    } else {
        println!("\nNo affordable destinations available with current fuel level.");
        
        // Show what we'd need to reach the closest destination
        if let Some(closest) = destinations.first() {
            let fuel_shortage = closest.fuel_needed.saturating_sub(game_state.player.fuel);
            println!("Closest destination: {} ({:.0}km away)", closest.airport_name, closest.distance_km);
            println!("Need {} more fuel to reach it.", fuel_shortage);
            
            // Check if we can buy enough fuel
            if let Some(market) = game_state.get_current_market() {
                let fuel_cost = market.fuel_price * fuel_shortage;
                if game_state.player.can_afford(fuel_cost) {
                    println!("Could buy {} fuel for ${} to reach it.", fuel_shortage, fuel_cost);
                }
            }
        }
    }
}
