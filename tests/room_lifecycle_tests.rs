use tempfile::tempdir;

use kzrk::api::multiplayer_service::MultiplayerGameService;
use kzrk::systems::GameStatus;

/// Test the complete lifecycle of a room from creation to persistence
#[tokio::test]
async fn test_basic_room_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_lifecycle.db");
    let db_path_str = db_path.to_str().unwrap();

    let service = MultiplayerGameService::new_with_db_path(db_path_str);

    // Phase 1: Room Creation
    let create_response = service
        .create_room(
            "Lifecycle Test Room".to_string(),
            "LifecycleHost".to_string(),
            Some(4),
        )
        .expect("Failed to create room");

    let room_id = create_response.room_id;
    let host_id = create_response.host_player_id;

    // Verify room is created correctly
    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].id, room_id);
    assert_eq!(rooms[0].current_players, 1);
    assert_eq!(rooms[0].game_status, GameStatus::WaitingForPlayers);

    // Phase 2: Player Joins
    let player2_response = service
        .join_room(room_id, "Player2".to_string(), Some("LAX".to_string()))
        .expect("Failed for player2 to join");

    // Verify room has both players
    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms[0].current_players, 2);

    // Phase 3: Players Leave
    service
        .leave_room(room_id, player2_response.player_id)
        .expect("Failed for player2 to leave");

    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms[0].current_players, 1);

    // Phase 4: Host Leaves (Room becomes empty)
    service
        .leave_room(room_id, host_id)
        .expect("Failed for host to leave");

    // Room should still exist but be empty
    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].current_players, 0);
    assert_eq!(rooms[0].game_status, GameStatus::WaitingForPlayers);
    assert!(rooms[0].is_joinable);

    // Phase 5: New Player Joins Empty Room
    let new_player_response = service
        .join_room(room_id, "NewPlayer".to_string(), Some("DEN".to_string()))
        .expect("Failed for new player to join empty room");

    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms[0].current_players, 1);

    // Verify new player can get game state
    let final_state = service
        .get_room_state(room_id, new_player_response.player_id)
        .expect("Failed to get final room state");

    assert_eq!(final_state.players.len(), 1);
    assert_eq!(final_state.room_info.id, room_id);
    assert_eq!(final_state.room_info.name, "Lifecycle Test Room");
}

#[tokio::test]
async fn test_multiple_empty_rooms_management() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_multiple.db");
    let db_path_str = db_path.to_str().unwrap();

    let mut room_data = Vec::new();

    let service = MultiplayerGameService::new_with_db_path(db_path_str);

    // Create 3 rooms
    for i in 0..3 {
        let create_response = service
            .create_room(format!("Empty Room {}", i), format!("Host{}", i), Some(4))
            .expect("Failed to create room");

        let room_id = create_response.room_id;
        let host_id = create_response.host_player_id;

        // Join another player
        let player_response = service
            .join_room(room_id, format!("Player{}", i), Some("LAX".to_string()))
            .expect("Failed to join room");

        // Both leave
        service
            .leave_room(room_id, host_id)
            .expect("Host failed to leave");
        service
            .leave_room(room_id, player_response.player_id)
            .expect("Player failed to leave");

        room_data.push((room_id, format!("Empty Room {}", i)));
    }

    // Verify all rooms are empty but still exist
    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms.len(), 3);
    for room in &rooms {
        assert_eq!(room.current_players, 0);
        assert!(room.is_joinable);
    }

    // Test rejoining each empty room
    for (room_id, room_name) in &room_data {
        let join_response = service
            .join_room(
                *room_id,
                format!("Rejoiner for {}", room_name),
                Some("JFK".to_string()),
            )
            .unwrap_or_else(|_| panic!("Failed to rejoin {}", room_name));

        assert_eq!(join_response.room_id, *room_id);

        // Leave immediately to keep room empty for next test
        service
            .leave_room(*room_id, join_response.player_id)
            .expect("Failed to leave after rejoining");
    }
}

#[tokio::test]
async fn test_rapid_leave_rejoin() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_rapid.db");
    let db_path_str = db_path.to_str().unwrap();

    let service = MultiplayerGameService::new_with_db_path(db_path_str);

    let create_response = service
        .create_room(
            "Rapid Test Room".to_string(),
            "RapidHost".to_string(),
            Some(4),
        )
        .expect("Failed to create room");

    let room_id = create_response.room_id;
    let host_id = create_response.host_player_id;

    // Rapid leave/rejoin cycles
    for i in 0..3 {
        // Join a player
        let join_response = service
            .join_room(
                room_id,
                format!("RapidPlayer{}", i),
                Some("LAX".to_string()),
            )
            .expect("Failed to join rapidly");

        // Immediately leave
        service
            .leave_room(room_id, join_response.player_id)
            .expect("Failed to leave rapidly");

        // Verify room state is consistent
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].current_players, 1); // Only host remains
    }

    // Host leaves and rejoins
    service
        .leave_room(room_id, host_id)
        .expect("Host failed to leave");

    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms[0].current_players, 0);

    // Rejoin as different player
    let rejoin_response = service
        .join_room(
            room_id,
            "RejoinerPlayer".to_string(),
            Some("MIA".to_string()),
        )
        .expect("Failed to rejoin empty room");

    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms[0].current_players, 1);

    // Verify room state is still valid
    let final_state = service
        .get_room_state(room_id, rejoin_response.player_id)
        .expect("Failed to get final state");

    assert_eq!(final_state.players.len(), 1);
    assert_eq!(final_state.room_info.name, "Rapid Test Room");
}
