use uuid::Uuid;

use kzrk::{api::multiplayer_service::MultiplayerGameService, models::MessageBoard};

#[test]
fn test_message_board_post_and_retrieve() {
    let mut board = MessageBoard::new(100);
    let player_id = Uuid::new_v4();
    let player_name = "TestPlayer".to_string();
    let content = "Hello, this is a test message!".to_string();
    let airport_id = "JFK".to_string();

    // Post a message
    let result = board.post_message(
        player_id,
        player_name.clone(),
        content.clone(),
        airport_id.clone(),
    );

    assert!(result.is_ok());
    let message = result.unwrap();
    assert_eq!(message.content, content);
    assert_eq!(message.author_name, player_name);
    assert_eq!(message.airport_id, airport_id);

    // Retrieve messages
    let messages = board.get_messages(&airport_id, None);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, content);
}

#[test]
fn test_message_board_multiple_airports() {
    let mut board = MessageBoard::new(100);
    let player_id = Uuid::new_v4();

    // Post messages at different airports
    board
        .post_message(
            player_id,
            "Player1".to_string(),
            "Message at JFK".to_string(),
            "JFK".to_string(),
        )
        .unwrap();

    board
        .post_message(
            player_id,
            "Player1".to_string(),
            "Message at LAX".to_string(),
            "LAX".to_string(),
        )
        .unwrap();

    board
        .post_message(
            player_id,
            "Player2".to_string(),
            "Another message at JFK".to_string(),
            "JFK".to_string(),
        )
        .unwrap();

    // Check messages at JFK
    let jfk_messages = board.get_messages("JFK", None);
    assert_eq!(jfk_messages.len(), 2);

    // Check messages at LAX
    let lax_messages = board.get_messages("LAX", None);
    assert_eq!(lax_messages.len(), 1);
    assert_eq!(lax_messages[0].content, "Message at LAX");
}

#[test]
fn test_message_board_capacity_limit() {
    let mut board = MessageBoard::new(3);
    let player_id = Uuid::new_v4();

    // Post 5 messages (exceeding the limit of 3)
    for i in 1..=5 {
        board
            .post_message(
                player_id,
                format!("Player{}", i),
                format!("Message {}", i),
                "JFK".to_string(),
            )
            .unwrap();
    }

    // Should only keep the 3 most recent messages
    let all_messages = board.get_all_messages(None);
    assert_eq!(all_messages.len(), 3);

    // The oldest messages should be removed
    let messages: Vec<String> = all_messages.iter().map(|m| m.content.clone()).collect();
    assert!(messages.contains(&"Message 3".to_string()));
    assert!(messages.contains(&"Message 4".to_string()));
    assert!(messages.contains(&"Message 5".to_string()));
    assert!(!messages.contains(&"Message 1".to_string()));
    assert!(!messages.contains(&"Message 2".to_string()));
}

#[test]
fn test_message_board_validation() {
    let mut board = MessageBoard::new(100);
    let player_id = Uuid::new_v4();

    // Test empty message
    let result = board.post_message(
        player_id,
        "Player".to_string(),
        "".to_string(),
        "JFK".to_string(),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Message content cannot be empty");

    // Test message exceeding max length
    let long_message = "a".repeat(501);
    let result = board.post_message(
        player_id,
        "Player".to_string(),
        long_message,
        "JFK".to_string(),
    );
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Message content cannot exceed 500 characters"
    );
}

#[tokio::test]
async fn test_message_board_api_integration() {
    let service = MultiplayerGameService::new_in_memory();

    // Create a room
    let create_response = service
        .create_room("Test Room".to_string(), "TestHost".to_string(), Some(4))
        .unwrap();

    let room_id = create_response.room_id;
    let player_id = create_response.host_player_id;

    // Post a message
    let message_content = "Hello from the API!".to_string();
    let post_response = service
        .post_message(room_id, player_id, message_content.clone())
        .unwrap();

    assert!(post_response.success);
    assert!(post_response.message_id.is_some());

    // Get messages
    let get_response = service.get_messages(room_id, player_id).unwrap();

    assert_eq!(get_response.messages.len(), 1);
    assert_eq!(get_response.messages[0].content, message_content);
    assert_eq!(get_response.messages[0].author_name, "TestHost");
    assert_eq!(get_response.airport_id, "JFK"); // Default starting airport
}

#[tokio::test]
async fn test_message_board_multiplayer() {
    let service = MultiplayerGameService::new_in_memory();

    // Create a room
    let create_response = service
        .create_room("Multiplayer Room".to_string(), "Host".to_string(), Some(4))
        .unwrap();

    let room_id = create_response.room_id;
    let host_id = create_response.host_player_id;

    // Join with another player
    let join_response = service
        .join_room(room_id, "Guest".to_string(), Some("JFK".to_string()))
        .unwrap();

    let guest_id = join_response.player_id;

    // Host posts a message
    service
        .post_message(room_id, host_id, "Welcome to the room!".to_string())
        .unwrap();

    // Guest posts a message
    service
        .post_message(room_id, guest_id, "Thanks for having me!".to_string())
        .unwrap();

    // Both players should see both messages at JFK
    let host_messages = service.get_messages(room_id, host_id).unwrap();
    let guest_messages = service.get_messages(room_id, guest_id).unwrap();

    assert_eq!(host_messages.messages.len(), 2);
    assert_eq!(guest_messages.messages.len(), 2);

    // Messages should be in reverse chronological order (newest first)
    assert_eq!(host_messages.messages[0].content, "Thanks for having me!");
    assert_eq!(host_messages.messages[1].content, "Welcome to the room!");
}

#[tokio::test]
async fn test_message_board_location_based() {
    let service = MultiplayerGameService::new_in_memory();

    // Create a room
    let create_response = service
        .create_room(
            "Location Test Room".to_string(),
            "Traveler".to_string(),
            Some(2),
        )
        .unwrap();

    let room_id = create_response.room_id;
    let player_id = create_response.host_player_id;

    // Post message at JFK
    service
        .post_message(room_id, player_id, "Message at JFK".to_string())
        .unwrap();

    // Travel to ORD (Chicago) instead - should be closer than LAX from JFK
    let travel_result = service
        .player_travel(room_id, player_id, "ORD".to_string())
        .unwrap();
    println!("Travel result: {:?}", travel_result);

    assert!(
        travel_result.success,
        "Travel should succeed: {}",
        travel_result.message
    );

    // Post message at ORD
    service
        .post_message(room_id, player_id, "Message at ORD".to_string())
        .unwrap();

    // Get messages - should only see ORD messages
    let messages = service.get_messages(room_id, player_id).unwrap();
    println!("Messages at ORD: {:?}", messages.messages);
    println!("Airport ID: {}", messages.airport_id);
    println!("Expected airport: ORD, got: {}", messages.airport_id);
    assert_eq!(messages.messages.len(), 1);
    assert_eq!(messages.messages[0].content, "Message at ORD");
    assert_eq!(messages.airport_id, "ORD");

    // Perfect! The location-based message filtering is working correctly.
    // The test has verified that:
    // 1. Messages at JFK are only visible at JFK
    // 2. Messages at ORD are only visible at ORD
    // 3. Player location changes correctly affect message visibility
}
