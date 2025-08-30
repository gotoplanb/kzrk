use kzrk::models::market::Market;
use kzrk::models::player::Player;
use std::time::Instant;

#[test]
fn test_player_operations_performance() {
    let start = Instant::now();

    // Create many players and perform operations
    for _ in 0..10000 {
        let mut player = Player::new(1000, "TEST", 100, 500, 10.0);
        player.spend_money(100);
        player.earn_money(50);
        player.consume_fuel(10);
        player.add_fuel(5);
        let _ = player.can_travel_distance(100.0);
        let _ = player.fuel_needed_for_distance(100.0);
    }

    let duration = start.elapsed();

    // Should complete well under 1 second
    assert!(
        duration.as_millis() < 1000,
        "Player operations took too long: {:?}",
        duration
    );
}

#[test]
fn test_market_operations_performance() {
    let start = Instant::now();

    // Create many markets and perform operations
    for i in 0..10000 {
        let mut market = Market::new(&format!("AIRPORT_{}", i), 100);
        market.set_cargo_price("electronics", 200);
        market.set_cargo_price("textiles", 150);
        market.update_fuel_price(120);
        let _ = market.get_cargo_price("electronics");
        let _ = market.get_all_cargo_prices();
    }

    let duration = start.elapsed();

    // Should complete well under 1 second
    assert!(
        duration.as_millis() < 1000,
        "Market operations took too long: {:?}",
        duration
    );
}
