use tempfile::tempdir;

use kzrk::api::multiplayer_service::MultiplayerGameService;
use kzrk::systems::GameStatus;

#[tokio::test]
async fn test_room_persistence_through_service_restart() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_persistence.db");
    let db_path_str = db_path.to_str().unwrap();

    // Create first service instance
    let room_response = {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        // Create a room
        let response = service
            .create_room(
                "Persistence Test Room".to_string(),
                "TestHost".to_string(),
                Some(4),
            )
            .expect("Failed to create room");

        // Verify room exists
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].name, "Persistence Test Room");

        response
    }; // First service instance is dropped here

    // Create second service instance (simulates server restart)
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        // Verify room persists
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].id, room_response.room_id);
        assert_eq!(rooms[0].name, "Persistence Test Room");
        assert_eq!(rooms[0].host_player_name, "TestHost");
    }
}

#[tokio::test]
async fn test_empty_room_persistence() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_empty.db");
    let db_path_str = db_path.to_str().unwrap();

    let room_id;
    let host_id;
    let player_id;

    // Create room, join players, then have all leave
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        // Create room
        let room_response = service
            .create_room("Empty Room Test".to_string(), "Host".to_string(), Some(4))
            .expect("Failed to create room");
        room_id = room_response.room_id;
        host_id = room_response.host_player_id;

        // Join second player
        let join_response = service
            .join_room(room_id, "Player2".to_string(), Some("LAX".to_string()))
            .expect("Failed to join room");
        player_id = join_response.player_id;

        // Verify room has 2 players
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms[0].current_players, 2);

        // Both players leave
        service
            .leave_room(room_id, host_id)
            .expect("Failed for host to leave");
        service
            .leave_room(room_id, player_id)
            .expect("Failed for player to leave");

        // Verify room still exists but is empty
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].current_players, 0);
        assert_eq!(rooms[0].game_status, GameStatus::WaitingForPlayers);
        assert!(rooms[0].is_joinable);
    } // Service dropped

    // Create new service and verify empty room persists
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].id, room_id);
        assert_eq!(rooms[0].current_players, 0);
        assert!(rooms[0].is_joinable);

        // Verify we can rejoin the empty room
        let rejoin_response = service
            .join_room(
                room_id,
                "RejoiningPlayer".to_string(),
                Some("JFK".to_string()),
            )
            .expect("Failed to rejoin empty room");

        assert_eq!(rejoin_response.room_id, room_id);
        assert_eq!(rejoin_response.player_name, "RejoiningPlayer");

        // Verify room now has 1 player again
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms[0].current_players, 1);
    }
}

#[tokio::test]
async fn test_game_state_persistence() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_game_state.db");
    let db_path_str = db_path.to_str().unwrap();

    let room_id;
    let player_id;

    // Create room, join, and perform game actions
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        let room_response = service
            .create_room(
                "Game State Test".to_string(),
                "GameHost".to_string(),
                Some(4),
            )
            .expect("Failed to create room");
        room_id = room_response.room_id;
        player_id = room_response.host_player_id;

        // Get initial game state without performing actions
        let initial_state = service
            .get_room_state(room_id, player_id)
            .expect("Failed to get room state");

        // Verify player state
        let player = initial_state
            .players
            .iter()
            .find(|p| p.id == Some(player_id))
            .expect("Player not found");
        assert_eq!(player.current_airport, "JFK"); // Default starting airport
    } // Service dropped

    // Create new service and verify game state persists
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        let persisted_state = service
            .get_room_state(room_id, player_id)
            .expect("Failed to get persisted room state");

        // Verify game state persisted
        let player = persisted_state
            .players
            .iter()
            .find(|p| p.id == Some(player_id))
            .expect("Player not found after persistence");

        assert_eq!(player.current_airport, "JFK");
        assert_eq!(player.name, "GameHost");

        // Verify room metadata persisted
        assert_eq!(persisted_state.room_info.name, "Game State Test");
        assert_eq!(persisted_state.room_info.current_players, 1);
    }
}

#[tokio::test]
async fn test_multiple_rooms_persistence() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_multiple.db");
    let db_path_str = db_path.to_str().unwrap();

    let mut room_ids = Vec::new();

    // Create multiple rooms
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        for i in 0..3 {
            let response = service
                .create_room(format!("Room {}", i), format!("Host{}", i), Some(4))
                .expect("Failed to create room");
            room_ids.push(response.room_id);
        }

        // Verify all rooms exist
        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 3);
    } // Service dropped

    // Verify all rooms persist
    {
        let service = MultiplayerGameService::new_with_db_path(db_path_str);

        let rooms = service.list_rooms().expect("Failed to list rooms");
        assert_eq!(rooms.len(), 3);

        // Verify each room persisted correctly
        for (i, &room_id) in room_ids.iter().enumerate() {
            let room = rooms
                .iter()
                .find(|r| r.id == room_id)
                .unwrap_or_else(|| panic!("Room {} not found", i));

            assert_eq!(room.name, format!("Room {}", i));
            assert_eq!(room.host_player_name, format!("Host{}", i));
            assert_eq!(room.max_players, 4);
        }
    }
}

#[tokio::test]
async fn test_in_memory_service() {
    // Test the in-memory service to make sure it works without persistence
    let service = MultiplayerGameService::new_in_memory();

    let _room_response = service
        .create_room(
            "In Memory Room".to_string(),
            "InMemoryHost".to_string(),
            Some(4),
        )
        .expect("Failed to create room");

    let rooms = service.list_rooms().expect("Failed to list rooms");
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].name, "In Memory Room");

    // Drop the service and create a new one - room should not persist
    drop(service);
    let new_service = MultiplayerGameService::new_in_memory();
    let empty_rooms = new_service.list_rooms().expect("Failed to list rooms");
    assert_eq!(empty_rooms.len(), 0); // Should be empty since it's in-memory
}
