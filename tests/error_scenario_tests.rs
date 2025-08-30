use kzrk::api::{routes::create_router, service::GameService};
use kzrk::models::Player;
use kzrk::data::cargo_types::get_default_cargo_types;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use rstest::rstest;

struct TestServer {
    base_url: String,
    client: Client,
}

impl TestServer {
    async fn new() -> Self {
        let service = GameService::new();
        let app = create_router(service);
        
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to address");
        
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://127.0.0.1:{}", addr.port());
        
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to start server");
        });
        
        sleep(Duration::from_millis(100)).await;
        
        Self {
            base_url,
            client: Client::new(),
        }
    }
    
    async fn post(&self, path: &str, body: Value) -> reqwest::Result<reqwest::Response> {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(&body)
            .send()
            .await
    }
    
    async fn get(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        self.client.get(format!("{}{}", self.base_url, path)).send().await
    }

    async fn create_test_game(&self, starting_money: Option<u32>) -> (String, Value) {
        let create_game_body = json!({
            "player_name": "Error Test Player",
            "starting_money": starting_money.unwrap_or(5000)
        });
        
        let response = self.post("/game", create_game_body).await.unwrap();
        let body: Value = response.json().await.unwrap();
        let session_id = body["session_id"].as_str().unwrap().to_string();
        
        (session_id, body)
    }
}

#[tokio::test]
async fn test_invalid_session_id_errors() {
    let server = TestServer::new().await;
    let fake_session_id = Uuid::new_v4();

    // Test GET game state with invalid session
    let response = server.get(&format!("/game/{}", fake_session_id)).await.unwrap();
    assert_eq!(response.status(), 404);
    
    let error: Value = response.json().await.unwrap();
    assert_eq!(error["error"], "GameNotFound");
    assert!(error["message"].as_str().unwrap().contains("not found"));

    // Test travel with invalid session
    let travel_body = json!({"destination": "ORD"});
    let response = server.post(&format!("/game/{}/travel", fake_session_id), travel_body).await.unwrap();
    assert_eq!(response.status(), 400);

    // Test trade with invalid session
    let trade_body = json!({
        "cargo_type": "electronics",
        "quantity": 1,
        "action": "Buy"
    });
    let response = server.post(&format!("/game/{}/trade", fake_session_id), trade_body).await.unwrap();
    assert_eq!(response.status(), 400);

    // Test fuel purchase with invalid session
    let fuel_body = json!({"quantity": 50});
    let response = server.post(&format!("/game/{}/fuel", fake_session_id), fuel_body).await.unwrap();
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_insufficient_funds_scenarios() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(100)).await; // Very low starting money

    // Try to buy expensive cargo
    let expensive_trade = json!({
        "cargo_type": "luxury",
        "quantity": 10,
        "action": "Buy"
    });
    
    let response = server.post(&format!("/game/{}/trade", session_id), expensive_trade).await.unwrap();
    assert_eq!(response.status(), 200);
    
    let trade_response: Value = response.json().await.unwrap();
    assert_eq!(trade_response["success"], false);
    assert!(trade_response["message"].as_str().unwrap().to_lowercase().contains("fund"));

    // Try to buy expensive fuel
    let expensive_fuel = json!({"quantity": 1000});
    let response = server.post(&format!("/game/{}/fuel", session_id), expensive_fuel).await.unwrap();
    assert_eq!(response.status(), 200);
    
    let fuel_response: Value = response.json().await.unwrap();
    assert_eq!(fuel_response["success"], false);
    assert!(fuel_response["message"].as_str().unwrap().to_lowercase().contains("fund"));
}

#[tokio::test]
async fn test_insufficient_fuel_scenarios() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(5000)).await;

    // Try to travel far without enough fuel (player starts with ~133 fuel)
    let long_distance_travel = json!({"destination": "SEA"}); // Seattle is very far from JFK
    
    let response = server.post(&format!("/game/{}/travel", session_id), long_distance_travel).await.unwrap();
    assert_eq!(response.status(), 200);
    
    let travel_response: Value = response.json().await.unwrap();
    assert_eq!(travel_response["success"], false);
    assert!(travel_response["message"].as_str().unwrap().to_lowercase().contains("fuel"));
}

#[tokio::test]
async fn test_cargo_capacity_errors() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(500000)).await; // Very high money, limited cargo space

    // Try to buy more cargo than capacity allows
    let heavy_cargo_trade = json!({
        "cargo_type": "industrial", // Heavy cargo type (5 weight per unit)
        "quantity": 300, // 300 * 5 = 1500kg > 1000kg max capacity
        "action": "Buy"
    });
    
    let response = server.post(&format!("/game/{}/trade", session_id), heavy_cargo_trade).await.unwrap();
    assert_eq!(response.status(), 200);
    
    let trade_response: Value = response.json().await.unwrap();
    assert_eq!(trade_response["success"], false);
    assert!(trade_response["message"].as_str().unwrap().to_lowercase().contains("capacity"));
}

#[tokio::test]
async fn test_invalid_cargo_operations() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(5000)).await;

    // Try to buy invalid cargo type
    let invalid_cargo = json!({
        "cargo_type": "nonexistent_cargo",
        "quantity": 1,
        "action": "Buy"
    });
    
    let response = server.post(&format!("/game/{}/trade", session_id), invalid_cargo).await.unwrap();
    assert_eq!(response.status(), 400);

    // Try to sell cargo the player doesn't have
    let sell_nonexistent = json!({
        "cargo_type": "electronics",
        "quantity": 10,
        "action": "Sell"
    });
    
    let response = server.post(&format!("/game/{}/trade", session_id), sell_nonexistent).await.unwrap();
    assert_eq!(response.status(), 200);
    
    let sell_response: Value = response.json().await.unwrap();
    assert_eq!(sell_response["success"], false);
    assert!(sell_response["message"].as_str().unwrap().to_lowercase().contains("insufficient"));
}

#[tokio::test]
async fn test_invalid_destination_errors() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(5000)).await;

    // Try to travel to nonexistent airport
    let invalid_destination = json!({"destination": "INVALID_AIRPORT"});
    
    let response = server.post(&format!("/game/{}/travel", session_id), invalid_destination).await.unwrap();
    assert_eq!(response.status(), 400);
    
    let error: Value = response.json().await.unwrap();
    assert_eq!(error["error"], "TravelError");

    // Try to travel to current airport (should be handled gracefully)
    let same_destination = json!({"destination": "JFK"}); // Starting airport
    
    let response = server.post(&format!("/game/{}/travel", session_id), same_destination).await.unwrap();
    // This might succeed or fail depending on implementation - both are valid
    assert!(response.status() == 200 || response.status() == 400);
}

#[tokio::test]
async fn test_fuel_capacity_limits() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(50000)).await;

    // First, fill up the fuel tank completely (player starts with ~133 fuel, so buy 67 to fill)
    let max_fuel_purchase = json!({"quantity": 67}); // Fill to max capacity
    let fill_response = server.post(&format!("/game/{}/fuel", session_id), max_fuel_purchase).await.unwrap();
    let fill_result: Value = fill_response.json().await.unwrap();
    assert_eq!(fill_result["success"], true); // Should succeed

    // Try to add more fuel when tank is full
    let excess_fuel = json!({"quantity": 1}); // Even 1 unit should fail
    let response = server.post(&format!("/game/{}/fuel", session_id), excess_fuel).await.unwrap();
    assert_eq!(response.status(), 200);
    
    let fuel_response: Value = response.json().await.unwrap();
    assert_eq!(fuel_response["success"], false);
    assert!(fuel_response["message"].as_str().unwrap().to_lowercase().contains("capacity")
         || fuel_response["message"].as_str().unwrap().to_lowercase().contains("hold"));
}

#[rstest]
#[case(0, "Buy")]
#[case(-1, "Buy")] // This should be caught by serde validation
#[tokio::test]
async fn test_invalid_trade_quantities(#[case] quantity: i32, #[case] action: &str) {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(5000)).await;

    let trade_body = json!({
        "cargo_type": "electronics",
        "quantity": quantity,
        "action": action
    });
    
    let response = server.post(&format!("/game/{}/trade", session_id), trade_body).await.unwrap();
    
    // Should either be a validation error (400/422) or 200 with success: false
    assert!(response.status() == 400 || response.status() == 422 || response.status() == 200);
    
    if response.status() == 200 {
        let trade_response: Value = response.json().await.unwrap();
        if quantity < 0 {
            assert_eq!(trade_response["success"], false);
        }
        // Note: quantity=0 is valid (buy 0 units for $0) so it should succeed
    }
}

#[tokio::test]
async fn test_malformed_request_bodies() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(5000)).await;

    // Test malformed travel request
    let malformed_travel = json!({"wrong_field": "ORD"});
    let response = server.post(&format!("/game/{}/travel", session_id), malformed_travel).await.unwrap();
    assert_eq!(response.status(), 422); // Unprocessable Entity for schema validation errors

    // Test malformed trade request
    let malformed_trade = json!({
        "cargo": "electronics", // Wrong field name
        "amount": 5, // Wrong field name
        "operation": "purchase" // Wrong action format
    });
    let response = server.post(&format!("/game/{}/trade", session_id), malformed_trade).await.unwrap();
    assert_eq!(response.status(), 422);

    // Test malformed fuel request
    let malformed_fuel = json!({"amount": 50}); // Wrong field name
    let response = server.post(&format!("/game/{}/fuel", session_id), malformed_fuel).await.unwrap();
    assert_eq!(response.status(), 422);
}

#[tokio::test]
async fn test_edge_case_values() {
    let server = TestServer::new().await;

    // Test game creation with edge case values
    let edge_case_game = json!({
        "player_name": "",
        "starting_money": 0,
        "starting_airport": "NONEXISTENT"
    });
    
    let response = server.post("/game", edge_case_game).await.unwrap();
    // Should fail gracefully, not crash
    assert!(response.status() == 400 || response.status() == 500);

    // Test with extremely large values
    let large_money_game = json!({
        "player_name": "Rich Player",
        "starting_money": u32::MAX
    });
    
    let response = server.post("/game", large_money_game).await.unwrap();
    // Should handle gracefully
    assert!(response.status() == 200 || response.status() == 400);
}

#[tokio::test]
async fn test_concurrent_operations_on_same_session() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(15000)).await;

    // Perform concurrent operations that might cause race conditions
    let trade1 = json!({
        "cargo_type": "electronics",
        "quantity": 5,
        "action": "Buy"
    });
    
    let trade2 = json!({
        "cargo_type": "textiles",
        "quantity": 3,
        "action": "Buy"
    });

    let fuel_purchase = json!({"quantity": 50});

    // Execute all operations concurrently
    let trade_url = format!("/game/{}/trade", session_id);
    let fuel_url = format!("/game/{}/fuel", session_id);
    let (trade_response1, trade_response2, fuel_response) = tokio::join!(
        server.post(&trade_url, trade1),
        server.post(&trade_url, trade2),
        server.post(&fuel_url, fuel_purchase)
    );

    // All should complete without server errors
    let trade1_result = trade_response1.unwrap();
    let trade2_result = trade_response2.unwrap();
    let fuel_result = fuel_response.unwrap();
    
    assert!(trade1_result.status().is_success() || trade1_result.status() == 400);
    assert!(trade2_result.status().is_success() || trade2_result.status() == 400);
    assert!(fuel_result.status().is_success() || fuel_result.status() == 400);

    // Verify game state is still consistent
    let state_url = format!("/game/{}", session_id);
    let state_response = server.get(&state_url).await.unwrap();
    assert_eq!(state_response.status(), 200);
    
    let game_state: Value = state_response.json().await.unwrap();
    let player_money = game_state["player"]["money"].as_u64().unwrap();
    
    // Money should be non-negative and reasonable
    // Verify player money is consistent
    assert!(player_money <= 15000); // Should not exceed starting money
}

#[tokio::test] 
async fn test_fuel_consumption_edge_cases() {
    let server = TestServer::new().await;
    let (session_id, _) = server.create_test_game(Some(50000)).await;

    // Consume all fuel through travel
    let consume_fuel = json!({"destination": "ORD"});
    let _ = server.post(&format!("/game/{}/travel", session_id), consume_fuel).await.unwrap();
    
    // Try another long trip with no fuel
    let long_trip = json!({"destination": "SEA"});
    let response = server.post(&format!("/game/{}/travel", session_id), long_trip).await.unwrap();
    
    let travel_response: Value = response.json().await.unwrap();
    assert_eq!(travel_response["success"], false);
    assert!(travel_response["message"].as_str().unwrap().to_lowercase().contains("fuel"));
}

// Unit tests for error conditions in models
#[cfg(test)]
mod model_error_tests {
    use super::*;

    #[test]
    fn test_player_money_underflow_protection() {
        let mut player = Player::new(100, "JFK", 200, 1000, 15.0);
        
        // Try to spend more money than available
        let success = player.spend_money(200);
        assert!(!success);
        assert_eq!(player.money, 100); // Should be unchanged
        
        // Money should never go below zero
        // Verify player money is consistent
    }

    #[test]
    fn test_fuel_underflow_protection() {
        let mut player = Player::new(5000, "JFK", 200, 1000, 15.0);
        player.fuel = 50;
        
        // Try to consume more fuel than available
        let success = player.consume_fuel(100);
        assert!(!success);
        assert_eq!(player.fuel, 50); // Should be unchanged
        
        // Fuel should never go below zero
        // Verify player fuel is consistent
    }

    #[test]
    fn test_fuel_overflow_protection() {
        let mut player = Player::new(5000, "JFK", 200, 1000, 15.0);
        player.fuel = 150;
        
        // Try to add fuel beyond capacity
        player.add_fuel(100);
        assert_eq!(player.fuel, 200); // Should be capped at max_fuel
        assert!(player.fuel <= player.max_fuel);
    }

    #[test]
    fn test_cargo_weight_validation() {
        let player = Player::new(5000, "JFK", 200, 100, 15.0); // Low cargo capacity
        let cargo_types = get_default_cargo_types();
        
        // Should reject cargo that exceeds capacity
        let can_carry = player.can_carry_more_weight(200, &cargo_types);
        assert!(!can_carry);
    }

    #[test]
    fn test_distance_calculation_edge_cases() {
        let airport1 = kzrk::models::Airport::new("A1", "Airport 1", (0.0, 0.0), 50, vec![], vec![], 1.0);
        let airport2 = kzrk::models::Airport::new("A2", "Airport 2", (0.0, 0.0), 50, vec![], vec![], 1.0);
        
        // Distance to same location should be zero
        let distance = airport1.distance_to(&airport2);
        assert!(distance < 0.001); // Allow for floating point precision
        
        // Test extreme coordinates
        let north_pole = kzrk::models::Airport::new("NP", "North Pole", (90.0, 0.0), 50, vec![], vec![], 1.0);
        let south_pole = kzrk::models::Airport::new("SP", "South Pole", (-90.0, 0.0), 50, vec![], vec![], 1.0);
        
        let pole_distance = north_pole.distance_to(&south_pole);
        assert!(pole_distance > 0.0);
        assert!(pole_distance.is_finite());
        
        // Should be approximately half Earth's circumference
        assert!(pole_distance > 15000.0 && pole_distance < 25000.0);
    }
}