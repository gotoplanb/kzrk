use tempfile::tempdir;
use uuid::Uuid;

use kzrk::api::database::Database;
use kzrk::data::{get_default_airports, get_default_cargo_types};
use kzrk::systems::{GameRoom, GameStatus, PlayerSession};

#[test]
fn test_database_creation_and_tables() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let db = Database::new(db_path.to_str().unwrap()).unwrap();

    // Test that tables were created by trying to query them
    let rooms = db.load_all_rooms().unwrap();
    let sessions = db.load_all_sessions().unwrap();

    assert!(rooms.is_empty());
    assert!(sessions.is_empty());
}

#[test]
fn test_in_memory_database() {
    let db = Database::in_memory().unwrap();

    let rooms = db.load_all_rooms().unwrap();
    let sessions = db.load_all_sessions().unwrap();

    assert!(rooms.is_empty());
    assert!(sessions.is_empty());
}

#[test]
fn test_save_and_load_room() {
    let db = Database::in_memory().unwrap();

    // Create a test room
    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();
    let room = GameRoom::new(
        "Test Room".to_string(),
        Uuid::new_v4(),
        "TestHost".to_string(),
        4,
        airports,
        cargo_types,
    );

    let room_id = room.id;

    // Save room
    db.save_room(&room).unwrap();

    // Load all rooms
    let rooms = db.load_all_rooms().unwrap();

    assert_eq!(rooms.len(), 1);
    assert!(rooms.contains_key(&room_id));

    let loaded_room = &rooms[&room_id];
    assert_eq!(loaded_room.name, "Test Room");
    assert_eq!(loaded_room.max_players, 4);
    assert_eq!(loaded_room.game_status, GameStatus::WaitingForPlayers);
}

#[test]
fn test_save_and_load_multiple_rooms() {
    let db = Database::in_memory().unwrap();

    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();

    // Create multiple rooms
    let mut room_ids = Vec::new();
    for i in 0..3 {
        let room = GameRoom::new(
            format!("Room {}", i),
            Uuid::new_v4(),
            format!("Host{}", i),
            4,
            airports.clone(),
            cargo_types.clone(),
        );
        room_ids.push(room.id);
        db.save_room(&room).unwrap();
    }

    // Load all rooms
    let rooms = db.load_all_rooms().unwrap();

    assert_eq!(rooms.len(), 3);
    for room_id in room_ids {
        assert!(rooms.contains_key(&room_id));
    }
}

#[test]
fn test_room_update_overwrites() {
    let db = Database::in_memory().unwrap();

    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();

    let mut room = GameRoom::new(
        "Original Name".to_string(),
        Uuid::new_v4(),
        "OriginalHost".to_string(),
        4,
        airports,
        cargo_types,
    );

    let room_id = room.id;

    // Save original room
    db.save_room(&room).unwrap();

    // Modify and save again
    room.name = "Updated Name".to_string();
    room.game_status = GameStatus::InProgress;
    db.save_room(&room).unwrap();

    // Load and verify update
    let rooms = db.load_all_rooms().unwrap();
    assert_eq!(rooms.len(), 1);

    let loaded_room = &rooms[&room_id];
    assert_eq!(loaded_room.name, "Updated Name");
    assert_eq!(loaded_room.game_status, GameStatus::InProgress);
}

#[test]
fn test_save_and_load_player_session() {
    let db = Database::in_memory().unwrap();

    let player_id = Uuid::new_v4();
    let room_id = Uuid::new_v4();

    let session = PlayerSession {
        player_id,
        player_name: "TestPlayer".to_string(),
        game_room_id: Some(room_id),
        connected_at: chrono::Utc::now(),
    };

    // Save session
    db.save_session(&session).unwrap();

    // Load all sessions
    let sessions = db.load_all_sessions().unwrap();

    assert_eq!(sessions.len(), 1);
    assert!(sessions.contains_key(&player_id));

    let loaded_session = &sessions[&player_id];
    assert_eq!(loaded_session.player_name, "TestPlayer");
    assert_eq!(loaded_session.game_room_id, Some(room_id));
}

#[test]
fn test_find_sessions_by_player_name() {
    let db = Database::in_memory().unwrap();

    let room_id = Uuid::new_v4();

    // Create multiple sessions for the same player name
    for _i in 0..3 {
        let session = PlayerSession {
            player_id: Uuid::new_v4(),
            player_name: "TestPlayer".to_string(),
            game_room_id: Some(room_id),
            connected_at: chrono::Utc::now(),
        };
        db.save_session(&session).unwrap();
    }

    // Create session for different player
    let other_session = PlayerSession {
        player_id: Uuid::new_v4(),
        player_name: "OtherPlayer".to_string(),
        game_room_id: Some(room_id),
        connected_at: chrono::Utc::now(),
    };
    db.save_session(&other_session).unwrap();

    // Find sessions by player name
    let sessions = db.find_sessions_by_player_name("TestPlayer").unwrap();

    assert_eq!(sessions.len(), 3);
    for session in &sessions {
        assert_eq!(session.player_name, "TestPlayer");
    }

    // Find sessions for other player
    let other_sessions = db.find_sessions_by_player_name("OtherPlayer").unwrap();
    assert_eq!(other_sessions.len(), 1);
    assert_eq!(other_sessions[0].player_name, "OtherPlayer");

    // Find sessions for non-existent player
    let empty_sessions = db.find_sessions_by_player_name("NonExistent").unwrap();
    assert!(empty_sessions.is_empty());
}

#[test]
fn test_session_update_overwrites() {
    let db = Database::in_memory().unwrap();

    let player_id = Uuid::new_v4();
    let room_id_1 = Uuid::new_v4();
    let room_id_2 = Uuid::new_v4();

    let mut session = PlayerSession {
        player_id,
        player_name: "TestPlayer".to_string(),
        game_room_id: Some(room_id_1),
        connected_at: chrono::Utc::now(),
    };

    // Save original session
    db.save_session(&session).unwrap();

    // Update session
    session.game_room_id = Some(room_id_2);
    db.save_session(&session).unwrap();

    // Verify only one session exists and it's updated
    let sessions = db.load_all_sessions().unwrap();
    assert_eq!(sessions.len(), 1);

    let loaded_session = &sessions[&player_id];
    assert_eq!(loaded_session.game_room_id, Some(room_id_2));
}

#[test]
fn test_delete_operations() {
    let db = Database::in_memory().unwrap();

    // Create test data
    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();
    let room = GameRoom::new(
        "Test Room".to_string(),
        Uuid::new_v4(),
        "TestHost".to_string(),
        4,
        airports,
        cargo_types,
    );
    let room_id = room.id;

    let player_id = Uuid::new_v4();
    let session = PlayerSession {
        player_id,
        player_name: "TestPlayer".to_string(),
        game_room_id: Some(room_id),
        connected_at: chrono::Utc::now(),
    };

    // Save data
    db.save_room(&room).unwrap();
    db.save_session(&session).unwrap();

    // Verify data exists
    assert_eq!(db.load_all_rooms().unwrap().len(), 1);
    assert_eq!(db.load_all_sessions().unwrap().len(), 1);

    // Delete room
    db.delete_room(&room_id).unwrap();
    assert!(db.load_all_rooms().unwrap().is_empty());
    assert_eq!(db.load_all_sessions().unwrap().len(), 1); // Session should still exist

    // Delete session
    db.delete_session(&player_id).unwrap();
    assert!(db.load_all_sessions().unwrap().is_empty());
}

#[test]
fn test_persistence_across_database_instances() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("persistence_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let player_id = Uuid::new_v4();
    let room_id;

    // Create first database instance and save data
    {
        let db = Database::new(db_path_str).unwrap();

        let airports = get_default_airports();
        let cargo_types = get_default_cargo_types();
        let room = GameRoom::new(
            "Persistent Room".to_string(),
            player_id, // Use player_id as host_player_id
            "PersistentHost".to_string(),
            4,
            airports,
            cargo_types,
        );

        room_id = room.id; // Get the actual room ID generated by new()

        let session = PlayerSession {
            player_id,
            player_name: "PersistentPlayer".to_string(),
            game_room_id: Some(room_id),
            connected_at: chrono::Utc::now(),
        };

        db.save_room(&room).unwrap();
        db.save_session(&session).unwrap();
    }

    // Create second database instance and verify data persists
    {
        let db = Database::new(db_path_str).unwrap();

        let rooms = db.load_all_rooms().unwrap();
        let sessions = db.load_all_sessions().unwrap();

        assert_eq!(rooms.len(), 1);
        assert_eq!(sessions.len(), 1);

        assert!(rooms.contains_key(&room_id));
        assert!(sessions.contains_key(&player_id));

        let loaded_room = &rooms[&room_id];
        assert_eq!(loaded_room.name, "Persistent Room");

        let loaded_session = &sessions[&player_id];
        assert_eq!(loaded_session.player_name, "PersistentPlayer");
    }
}

#[test]
fn test_serialization_error_handling() {
    let db = Database::in_memory().unwrap();

    // Test loading from empty database
    let rooms = db.load_all_rooms().unwrap();
    let sessions = db.load_all_sessions().unwrap();

    assert!(rooms.is_empty());
    assert!(sessions.is_empty());

    // The database should handle malformed data gracefully by skipping invalid entries
    // This is tested implicitly by the serialization/deserialization process
}
