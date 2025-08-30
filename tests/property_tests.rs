use kzrk::{
    data::{airports::get_default_airports, cargo_types::get_default_cargo_types},
    models::{Airport, Player},
};
use proptest::prelude::*;

// Property-based tests for game mechanics consistency

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_fuel_consumption_is_consistent(
            distance in 0.0f64..10000.0,
            efficiency in 1.0f64..50.0
        ) {
            let player = Player::new(5000, "JFK", 1000, 1000, efficiency as f32);

            let fuel_needed1 = player.fuel_needed_for_distance(distance);
            let fuel_needed2 = player.fuel_needed_for_distance(distance);

            // Fuel calculation should be deterministic
            prop_assert_eq!(fuel_needed1, fuel_needed2);

            // Fuel needed should be proportional to distance
            if distance > 0.0 {
                prop_assert!(fuel_needed1 > 0);
                prop_assert_eq!(fuel_needed1, (distance / efficiency).ceil() as u32);
            } else {
                prop_assert_eq!(fuel_needed1, 0);
            }
        }

        #[test]
        fn test_distance_calculation_properties(
            lat1 in -90.0f64..90.0,
            lon1 in -180.0f64..180.0,
            lat2 in -90.0f64..90.0,
            lon2 in -180.0f64..180.0
        ) {
            let airport1 = Airport::new("A1", "Airport 1", (lat1, lon1), 50, vec![], vec![], 1.0);
            let airport2 = Airport::new("A2", "Airport 2", (lat2, lon2), 50, vec![], vec![], 1.0);

            let distance_a_to_b = airport1.distance_to(&airport2);
            let distance_b_to_a = airport2.distance_to(&airport1);

            // Distance should be symmetric
            prop_assert!((distance_a_to_b - distance_b_to_a).abs() < 0.001);

            // Distance should be non-negative
            prop_assert!(distance_a_to_b >= 0.0);

            // Distance to self should be zero
            let distance_to_self = airport1.distance_to(&airport1);
            prop_assert!(distance_to_self < 0.001); // Allow for floating point precision
        }

        #[test]
        fn test_player_money_transactions(
            initial_money in 0u32..100000,
            spend_amount in 0u32..50000,
            earn_amount in 0u32..50000
        ) {
            let mut player = Player::new(initial_money, "JFK", 100, 1000, 10.0);
            let original_money = player.money;

            // Test spending money
            if spend_amount <= original_money {
                let success = player.spend_money(spend_amount);
                prop_assert!(success);
                prop_assert_eq!(player.money, original_money - spend_amount);
            } else {
                let success = player.spend_money(spend_amount);
                prop_assert!(!success);
                prop_assert_eq!(player.money, original_money); // Money should be unchanged
            }

            // Test earning money
            let money_before_earning = player.money;
            player.earn_money(earn_amount);
            prop_assert_eq!(player.money, money_before_earning + earn_amount);

            // Money should be consistent after operations
        }

        #[test]
        fn test_fuel_management_properties(
            max_fuel in 50u32..1000,
            initial_fuel_ratio in 0.1f32..1.0,
            fuel_to_add in 0u32..200,
            fuel_to_consume in 0u32..200
        ) {
            let initial_fuel = (max_fuel as f32 * initial_fuel_ratio) as u32;
            let mut player = Player::new(5000, "JFK", max_fuel, 1000, 10.0);
            player.fuel = initial_fuel;

            // Test adding fuel
            let fuel_before_add = player.fuel;
            player.add_fuel(fuel_to_add);

            // Fuel should not exceed max_fuel
            prop_assert!(player.fuel <= max_fuel);

            // Fuel should increase by the minimum of fuel_to_add and available capacity
            let expected_fuel = std::cmp::min(fuel_before_add + fuel_to_add, max_fuel);
            prop_assert_eq!(player.fuel, expected_fuel);

            // Test consuming fuel
            let fuel_before_consume = player.fuel;
            let success = player.consume_fuel(fuel_to_consume);

            if fuel_to_consume <= fuel_before_consume {
                prop_assert!(success);
                prop_assert_eq!(player.fuel, fuel_before_consume - fuel_to_consume);
            } else {
                prop_assert!(!success);
                prop_assert_eq!(player.fuel, fuel_before_consume); // Unchanged on failure
            }

            // Fuel should be consistent
        }

        #[test]
        fn test_cargo_weight_calculations(
            max_cargo_weight in 100u32..5000,
            quantities in prop::collection::vec(1u32..50, 0..6)
        ) {
            let player = Player::new(5000, "JFK", 200, max_cargo_weight, 10.0);
            let cargo_types = get_default_cargo_types();
            let cargo_ids: Vec<String> = cargo_types.keys().cloned().collect();

            if cargo_ids.is_empty() {
                return Ok(());
            }

            let mut total_expected_weight = 0u32;

            // Calculate expected weight for various quantities
            for (i, &quantity) in quantities.iter().enumerate() {
                if i >= cargo_ids.len() {
                    break;
                }

                let cargo_id = &cargo_ids[i];
                if let Some(cargo_type) = cargo_types.get(cargo_id) {
                    let weight_for_cargo = cargo_type.weight_per_unit * quantity;
                    total_expected_weight = total_expected_weight.saturating_add(weight_for_cargo);

                    // Test individual cargo weight check
                    let can_carry = player.can_carry_more_weight(weight_for_cargo, &cargo_types);

                    if player.current_cargo_weight(&cargo_types) + weight_for_cargo <= max_cargo_weight {
                        prop_assert!(can_carry);
                    } else {
                        prop_assert!(!can_carry);
                    }
                }
            }
        }

        #[test]
        fn test_travel_distance_fuel_relationship(
            efficiency in 1.0f32..50.0,
            distance in 0.0f64..5000.0
        ) {
            let player = Player::new(5000, "JFK", 1000, 1000, efficiency);

            let fuel_needed = player.fuel_needed_for_distance(distance);
            let can_travel = player.can_travel_distance(distance);

            // If we have enough fuel, we should be able to travel
            if fuel_needed <= player.fuel {
                prop_assert!(can_travel);
            } else {
                prop_assert!(!can_travel);
            }

            // Fuel needed should increase with distance (for positive distances)
            if distance > 0.0 {
                let longer_distance = distance * 1.5;
                let fuel_for_longer = player.fuel_needed_for_distance(longer_distance);
                prop_assert!(fuel_for_longer >= fuel_needed);
            }
        }

        #[test]
        fn test_airport_coordinates_validity(
            latitude in -90.0f64..90.0,
            longitude in -180.0f64..180.0
        ) {
            let airport = Airport::new(
                "TEST",
                "Test Airport",
                (latitude, longitude),
                50,
                vec![],
                vec![],
                1.0
            );

            // Coordinates should be preserved
            prop_assert_eq!(airport.coordinates.0, latitude);
            prop_assert_eq!(airport.coordinates.1, longitude);

            // Test distance to a known point (JFK coordinates)
            let jfk = Airport::new("JFK", "JFK", (40.6413, -73.7781), 80, vec![], vec![], 1.0);
            let distance = airport.distance_to(&jfk);

            // Distance should be finite and non-negative
            prop_assert!(distance.is_finite());
            prop_assert!(distance >= 0.0);

            // Maximum distance on Earth should be roughly half the circumference
            prop_assert!(distance <= 20037.5); // Approximately half Earth's circumference in km
        }
    }

    #[test]
    fn test_market_price_boundaries() {
        let cargo_types = get_default_cargo_types();

        // Test that all cargo types have reasonable base prices and weights
        for (id, cargo_type) in cargo_types {
            assert!(
                cargo_type.base_price > 0,
                "Cargo {} has zero base price",
                id
            );
            assert!(
                cargo_type.base_price < 10000,
                "Cargo {} has unreasonably high base price: {}",
                id,
                cargo_type.base_price
            );

            assert!(
                cargo_type.weight_per_unit > 0,
                "Cargo {} has zero weight",
                id
            );
            assert!(
                cargo_type.weight_per_unit < 100,
                "Cargo {} has unreasonably high weight: {}",
                id,
                cargo_type.weight_per_unit
            );

            assert!(
                cargo_type.volatility >= 0.0,
                "Cargo {} has negative volatility",
                id
            );
            assert!(
                cargo_type.volatility <= 1.0,
                "Cargo {} has volatility > 1.0: {}",
                id,
                cargo_type.volatility
            );
        }
    }

    #[test]
    fn test_airport_data_consistency() {
        let airports = get_default_airports();

        // Test that all airports have valid data
        for (id, airport) in airports {
            assert!(!airport.name.is_empty(), "Airport {} has empty name", id);
            assert_eq!(airport.id, id, "Airport ID mismatch for {}", id);

            // Coordinates should be within valid Earth bounds
            assert!(
                airport.coordinates.0 >= -90.0 && airport.coordinates.0 <= 90.0,
                "Airport {} has invalid latitude: {}",
                id,
                airport.coordinates.0
            );
            assert!(
                airport.coordinates.1 >= -180.0 && airport.coordinates.1 <= 180.0,
                "Airport {} has invalid longitude: {}",
                id,
                airport.coordinates.1
            );

            // Fuel prices should be reasonable
            assert!(
                airport.base_fuel_price > 0,
                "Airport {} has zero fuel price",
                id
            );
            assert!(
                airport.base_fuel_price < 500,
                "Airport {} has unreasonably high fuel price: {}",
                id,
                airport.base_fuel_price
            );

            // Fuel modifier should be positive
            assert!(
                airport.market_profile.fuel_modifier > 0.0,
                "Airport {} has non-positive fuel modifier",
                id
            );
            assert!(
                airport.market_profile.fuel_modifier < 5.0,
                "Airport {} has extremely high fuel modifier: {}",
                id,
                airport.market_profile.fuel_modifier
            );
        }
    }
}
