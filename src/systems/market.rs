use crate::models::{Airport, CargoType, Market};
use rand::Rng;
use std::collections::HashMap;

pub struct MarketSystem;

impl MarketSystem {
    pub fn generate_market_prices(
        airport: &Airport,
        cargo_types: &HashMap<String, CargoType>,
        rng: &mut impl Rng,
    ) -> HashMap<String, u32> {
        let mut prices = HashMap::new();

        for (cargo_id, cargo_type) in cargo_types {
            let base_price = cargo_type.base_price;
            let volatility = cargo_type.volatility;

            // Base price fluctuation (-volatility to +volatility)
            let price_modifier = 1.0 + rng.gen_range(-volatility..volatility);

            // Apply airport market profile modifiers
            let profile_modifier = if airport.market_profile.produces.contains(cargo_id) {
                // Airport produces this cargo - lower buy prices (0.7-0.9x base)
                rng.gen_range(0.7..0.9)
            } else if airport.market_profile.consumes.contains(cargo_id) {
                // Airport consumes this cargo - higher sell prices (1.1-1.4x base)
                rng.gen_range(1.1..1.4)
            } else {
                // Neutral cargo - normal price range (0.9-1.1x base)
                rng.gen_range(0.9..1.1)
            };

            let final_price = (base_price as f32 * price_modifier * profile_modifier) as u32;
            let final_price = final_price.max(1); // Ensure price is at least $1

            prices.insert(cargo_id.clone(), final_price);
        }

        prices
    }

    pub fn generate_fuel_price(airport: &Airport, rng: &mut impl Rng) -> u32 {
        let base_price = airport.base_fuel_price;
        let modifier = airport.market_profile.fuel_modifier;

        // Add some randomness (±15%)
        let random_modifier = rng.gen_range(0.85..1.15);

        let final_price = (base_price as f32 * modifier * random_modifier) as u32;
        final_price.max(1) // Ensure price is at least $1
    }

    pub fn update_market_prices(
        market: &mut Market,
        airport: &Airport,
        cargo_types: &HashMap<String, CargoType>,
        rng: &mut impl Rng,
    ) {
        // Update cargo prices
        let new_cargo_prices = Self::generate_market_prices(airport, cargo_types, rng);
        for (cargo_id, price) in new_cargo_prices {
            market.set_cargo_price(&cargo_id, price);
        }

        // Update fuel price
        let new_fuel_price = Self::generate_fuel_price(airport, rng);
        market.update_fuel_price(new_fuel_price);
    }

    pub fn initialize_all_markets(
        airports: &HashMap<String, Airport>,
        cargo_types: &HashMap<String, CargoType>,
        rng: &mut impl Rng,
    ) -> HashMap<String, Market> {
        let mut markets = HashMap::new();

        for (airport_id, airport) in airports {
            let fuel_price = Self::generate_fuel_price(airport, rng);
            let mut market = Market::new(airport_id, fuel_price);

            // Generate initial cargo prices
            let cargo_prices = Self::generate_market_prices(airport, cargo_types, rng);
            for (cargo_id, price) in cargo_prices {
                market.set_cargo_price(&cargo_id, price);
            }

            markets.insert(airport_id.clone(), market);
        }

        markets
    }
}
