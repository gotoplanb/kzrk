mod data;
mod models;
mod systems;
mod ui;

use data::{get_default_airports, get_default_cargo_types};
use systems::GameState;

fn main() {
    println!("=== KZRK Aviation Trading Game ===");
    
    // Initialize game data
    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();
    
    // Create game state
    let game_state = GameState::new(airports, cargo_types);
    
    // Basic status display for now
    println!("Game initialized successfully!");
    println!("Player starting at: {}", game_state.player.current_airport);
    println!("Starting money: ${}", game_state.player.money);
    println!("Starting fuel: {}/{}", game_state.player.fuel, game_state.player.max_fuel);
    println!("Available airports: {}", game_state.airports.len());
    println!("Available cargo types: {}", game_state.cargo_types.len());
    
    if let Some(current_airport) = game_state.get_current_airport() {
        println!("Current location: {}", current_airport.name);
    }
}
