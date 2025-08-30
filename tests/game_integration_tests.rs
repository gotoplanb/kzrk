use kzrk::data::cargo_types::get_default_cargo_types;
use kzrk::models::market::Market;
use kzrk::models::player::Player;

#[test]
fn test_player_market_interaction() {
    // Create a player and market
    let mut player = Player::new(1000, "JFK", 100, 500, 10.0);
    let mut market = Market::new("JFK", 50);

    // Set up some cargo prices
    market.set_cargo_price("electronics", 200);
    market.set_cargo_price("textiles", 150);

    // Test fuel purchase
    let fuel_cost = market.fuel_price * 20; // 20 units of fuel
    assert!(player.can_afford(fuel_cost));
    assert!(player.spend_money(fuel_cost));
    player.add_fuel(20);

    // Verify player state after fuel purchase
    assert_eq!(player.money, 1000 - fuel_cost);
    assert_eq!(player.fuel, 86); // Started with 66, added 20
}

#[test]
fn test_player_cargo_weight_limits() {
    let player = Player::new(1000, "JFK", 100, 100, 10.0); // Low cargo limit
    let cargo_types = get_default_cargo_types();

    // Test that we respect cargo weight limits
    assert!(player.can_carry_more_weight(50, &cargo_types));
    assert!(player.can_carry_more_weight(100, &cargo_types));
    assert!(!player.can_carry_more_weight(101, &cargo_types));
}

#[test]
fn test_fuel_travel_calculations() {
    let player = Player::new(1000, "JFK", 100, 500, 10.0);

    // Test various distances
    assert!(player.can_travel_distance(100.0)); // Needs 10 fuel, has 66
    assert!(player.can_travel_distance(500.0)); // Needs 50 fuel, has 66
    assert!(player.can_travel_distance(660.0)); // Needs 66 fuel, has exactly 66
    assert!(!player.can_travel_distance(670.0)); // Needs 67 fuel, has 66
}

#[test]
fn test_market_price_consistency() {
    let mut market1 = Market::new("JFK", 100);
    let mut market2 = Market::new("LAX", 120);

    // Set different prices in different markets
    market1.set_cargo_price("electronics", 200);
    market2.set_cargo_price("electronics", 300);

    // Verify markets maintain separate pricing
    assert_eq!(market1.get_cargo_price("electronics"), Some(200));
    assert_eq!(market2.get_cargo_price("electronics"), Some(300));
    assert_eq!(market1.fuel_price, 100);
    assert_eq!(market2.fuel_price, 120);
}

#[test]
fn test_complete_trading_scenario() {
    let mut player = Player::new(5000, "NYC", 200, 1000, 15.0); // More starting money
    let mut origin_market = Market::new("NYC", 20); // Cheaper fuel
    let mut destination_market = Market::new("LAX", 30);

    // Set up profitable trade route
    origin_market.set_cargo_price("electronics", 100); // Buy low
    destination_market.set_cargo_price("electronics", 300); // Sell high

    // Player buys fuel and cargo at origin
    let fuel_cost = origin_market.fuel_price * 20; // Buy 20 units of fuel
    let cargo_cost = origin_market.get_cargo_price("electronics").unwrap() * 3; // Buy 3 units
    let total_cost = fuel_cost + cargo_cost;

    // Ensure the transaction is affordable
    assert!(player.can_afford(total_cost));
    assert!(player.spend_money(total_cost));
    player.add_fuel(20);

    // Simulate travel (consume fuel for distance)
    let travel_distance = 300.0;
    let fuel_needed = player.fuel_needed_for_distance(travel_distance);
    assert!(player.can_travel_distance(travel_distance));
    assert!(player.consume_fuel(fuel_needed));

    // Verify player state after travel
    assert_eq!(player.money, 5000 - total_cost);
    assert!(player.fuel < 200); // Should have consumed some fuel

    // At destination, sell cargo
    let sell_price = destination_market.get_cargo_price("electronics").unwrap() * 3;
    player.earn_money(sell_price);

    // Verify profit was made
    let expected_profit = (300 - 100) * 3; // 200 profit per unit * 3 units = 600
    let final_money_change = player.money as i32 - 5000;
    let expected_final_change = expected_profit - fuel_cost as i32;
    assert_eq!(final_money_change, expected_final_change);
}
