use crate::models::CargoType;
use std::collections::HashMap;

pub fn get_default_cargo_types() -> HashMap<String, CargoType> {
    let mut cargo_types = HashMap::new();

    cargo_types.insert(
        "electronics".to_string(),
        CargoType::new("electronics", "Electronics", 500, 1, 0.4),
    );

    cargo_types.insert(
        "food".to_string(),
        CargoType::new("food", "Food & Beverages", 100, 2, 0.2),
    );

    cargo_types.insert(
        "textiles".to_string(),
        CargoType::new("textiles", "Textiles", 200, 3, 0.25),
    );

    cargo_types.insert(
        "industrial".to_string(),
        CargoType::new("industrial", "Industrial Parts", 300, 5, 0.3),
    );

    cargo_types.insert(
        "luxury".to_string(),
        CargoType::new("luxury", "Luxury Goods", 1000, 1, 0.5),
    );

    cargo_types.insert(
        "materials".to_string(),
        CargoType::new("materials", "Raw Materials", 50, 4, 0.15),
    );

    cargo_types
}
