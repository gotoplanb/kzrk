use std::collections::HashMap;

use crate::models::Airport;

pub fn get_default_airports() -> HashMap<String, Airport> {
    let mut airports = HashMap::new();

    airports.insert(
        "JFK".to_string(),
        Airport::new(
            "JFK",
            "New York JFK",
            (40.6413, -73.7781),
            80,
            vec!["electronics".to_string(), "luxury".to_string()],
            vec!["food".to_string(), "materials".to_string()],
            1.2,
        ),
    );

    airports.insert(
        "LAX".to_string(),
        Airport::new(
            "LAX",
            "Los Angeles LAX",
            (33.9425, -118.4081),
            75,
            vec!["electronics".to_string(), "textiles".to_string()],
            vec!["industrial".to_string(), "materials".to_string()],
            1.1,
        ),
    );

    airports.insert(
        "MIA".to_string(),
        Airport::new(
            "MIA",
            "Miami MIA",
            (25.7959, -80.2870),
            70,
            vec!["food".to_string(), "luxury".to_string()],
            vec!["electronics".to_string(), "textiles".to_string()],
            0.9,
        ),
    );

    airports.insert(
        "ORD".to_string(),
        Airport::new(
            "ORD",
            "Chicago O'Hare",
            (41.9742, -87.9073),
            65,
            vec!["industrial".to_string(), "food".to_string()],
            vec!["luxury".to_string(), "electronics".to_string()],
            1.0,
        ),
    );

    airports.insert(
        "DEN".to_string(),
        Airport::new(
            "DEN",
            "Denver DEN",
            (39.8561, -104.6737),
            60,
            vec!["materials".to_string(), "industrial".to_string()],
            vec!["luxury".to_string(), "food".to_string()],
            0.8,
        ),
    );

    airports.insert(
        "SEA".to_string(),
        Airport::new(
            "SEA",
            "Seattle SEA",
            (47.4502, -122.3088),
            85,
            vec!["electronics".to_string(), "food".to_string()],
            vec!["textiles".to_string(), "materials".to_string()],
            1.3,
        ),
    );

    airports
}
