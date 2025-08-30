use crate::models::{Airport, Player};
use crate::systems::GameState;

#[derive(Debug, Clone)]
pub enum TravelError {
    InsufficientFuel,
    InvalidDestination,
    SameLocation,
    DestinationNotFound,
}

pub struct TravelSystem;

impl TravelSystem {
    /// Calculate fuel needed to travel between two airports
    pub fn calculate_fuel_needed(
        player: &Player,
        distance: f64,
    ) -> u32 {
        player.fuel_needed_for_distance(distance)
    }
    
    /// Check if player can travel to a destination
    pub fn can_travel_to(
        player: &Player,
        from_airport: &Airport,
        to_airport: &Airport,
    ) -> Result<u32, TravelError> {
        // Check if trying to travel to same location
        if from_airport.id == to_airport.id {
            return Err(TravelError::SameLocation);
        }
        
        let distance = from_airport.distance_to(to_airport);
        let fuel_needed = Self::calculate_fuel_needed(player, distance);
        
        if player.fuel < fuel_needed {
            return Err(TravelError::InsufficientFuel);
        }
        
        Ok(fuel_needed)
    }
    
    /// Execute travel from current location to destination
    pub fn travel_to(
        game_state: &mut GameState,
        destination_id: &str,
    ) -> Result<TravelInfo, TravelError> {
        // Validate destination exists
        let destination_airport = game_state.airports.get(destination_id)
            .ok_or(TravelError::DestinationNotFound)?
            .clone();
        
        // Get current airport
        let current_airport = game_state.get_current_airport()
            .ok_or(TravelError::InvalidDestination)?
            .clone();
        
        // Check if travel is possible (skip fuel check in cheat mode)
        let fuel_needed = if game_state.cheat_mode {
            0 // Always allow travel in cheat mode
        } else {
            Self::can_travel_to(
                &game_state.player,
                &current_airport,
                &destination_airport,
            )?
        };
        
        // Calculate distance for travel info
        let distance = current_airport.distance_to(&destination_airport);
        
        // Execute the travel
        let actual_fuel_consumed = if game_state.cheat_mode {
            0 // Cheat mode: no fuel consumption
        } else {
            game_state.player.consume_fuel(fuel_needed);
            fuel_needed
        };
        game_state.player.current_airport = destination_id.to_string();
        
        // Refresh market prices at new location (simulate market changes over time)
        game_state.refresh_current_market();
        
        // Advance turn
        game_state.advance_turn();
        
        Ok(TravelInfo {
            from: current_airport.name.clone(),
            to: destination_airport.name.clone(),
            distance_km: distance,
            fuel_consumed: actual_fuel_consumed,
            remaining_fuel: game_state.player.fuel,
        })
    }
    
    /// Get all possible destinations from current location
    pub fn get_reachable_destinations(
        game_state: &GameState,
    ) -> Vec<DestinationInfo> {
        let mut destinations = Vec::new();
        
        if let Some(current_airport) = game_state.get_current_airport() {
            for destination in game_state.get_available_destinations() {
                let distance = current_airport.distance_to(destination);
                let fuel_needed = Self::calculate_fuel_needed(&game_state.player, distance);
                let can_afford = game_state.cheat_mode || game_state.player.fuel >= fuel_needed;
                
                destinations.push(DestinationInfo {
                    airport_id: destination.id.clone(),
                    airport_name: destination.name.clone(),
                    distance_km: distance,
                    fuel_needed,
                    can_afford,
                });
            }
        }
        
        // Sort by distance for easier navigation
        destinations.sort_by(|a, b| a.distance_km.partial_cmp(&b.distance_km).unwrap());
        destinations
    }
    
    /// Calculate travel cost in fuel for a given route
    #[allow(dead_code)]
    pub fn calculate_travel_cost(
        game_state: &GameState,
        destination_id: &str,
    ) -> Option<u32> {
        let destination = game_state.airports.get(destination_id)?;
        let current_airport = game_state.get_current_airport()?;
        
        let distance = current_airport.distance_to(destination);
        Some(Self::calculate_fuel_needed(&game_state.player, distance))
    }
}

#[derive(Debug, Clone)]
pub struct TravelInfo {
    pub from: String,
    pub to: String,
    pub distance_km: f64,
    pub fuel_consumed: u32,
    #[allow(dead_code)]
    pub remaining_fuel: u32,
}

#[derive(Debug, Clone)]
pub struct DestinationInfo {
    pub airport_id: String,
    pub airport_name: String,
    pub distance_km: f64,
    pub fuel_needed: u32,
    pub can_afford: bool,
}