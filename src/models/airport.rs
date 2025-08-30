use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketProfile {
    pub produces: Vec<String>, // Cargo types with lower buy prices
    pub consumes: Vec<String>, // Cargo types with higher sell prices
    pub fuel_modifier: f32,    // Multiplier for base fuel price (1.0 = normal)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airport {
    pub id: String,
    pub name: String,
    pub coordinates: (f64, f64),
    pub base_fuel_price: u32,
    pub market_profile: MarketProfile,
}

pub struct AirportConfig {
    pub id: String,
    pub name: String,
    pub coordinates: (f64, f64),
    pub base_fuel_price: u32,
    pub produces: Vec<String>,
    pub consumes: Vec<String>,
    pub fuel_modifier: f32,
}

impl Airport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        name: &str,
        coordinates: (f64, f64),
        base_fuel_price: u32,
        produces: Vec<String>,
        consumes: Vec<String>,
        fuel_modifier: f32,
    ) -> Self {
        let config = AirportConfig {
            id: id.to_string(),
            name: name.to_string(),
            coordinates,
            base_fuel_price,
            produces,
            consumes,
            fuel_modifier,
        };
        Self::from_config(config)
    }

    pub fn from_config(config: AirportConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            coordinates: config.coordinates,
            base_fuel_price: config.base_fuel_price,
            market_profile: MarketProfile {
                produces: config.produces,
                consumes: config.consumes,
                fuel_modifier: config.fuel_modifier,
            },
        }
    }

    pub fn distance_to(&self, other: &Airport) -> f64 {
        let (lat1, lon1) = self.coordinates;
        let (lat2, lon2) = other.coordinates;

        // Haversine formula for distance between two points on Earth
        let r = 6371.0; // Earth's radius in kilometers
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let lat1 = lat1.to_radians();
        let lat2 = lat2.to_radians();

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
}
