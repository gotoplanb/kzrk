#[cfg(feature = "gui")]
mod gui_tests {
    use std::sync::Arc;
    use tokio::task;

    use kzrk::api::multiplayer_service::MultiplayerGameService;
    use kzrk::ui::game_api_client::{ApiError, GameApiClient};

    async fn start_test_server() -> u16 {
        use tower_http::cors::CorsLayer;

        let service = Arc::new(MultiplayerGameService::new());
        let app = kzrk::api::routes::create_multiplayer_router((*service).clone())
            .layer(CorsLayer::permissive());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        task::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        port
    }

    #[tokio::test]
    async fn test_list_rooms_sync() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        // Initially no rooms
        let rooms = client.list_rooms_sync().unwrap();
        assert!(rooms.is_empty());

        // Create a room through async API
        let create_response = client
            .create_room("Test Room".to_string(), "TestHost".to_string(), Some(4))
            .await
            .unwrap();

        // List rooms synchronously
        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].name, "Test Room");
        assert_eq!(rooms[0].id, create_response.room_id);
        assert_eq!(rooms[0].host_player_name, "TestHost");
        assert_eq!(rooms[0].max_players, 4);
        assert_eq!(rooms[0].current_players, 1);
        assert!(rooms[0].is_joinable);
    }

    #[tokio::test]
    async fn test_create_room_sync() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        let response = client
            .create_room_sync(
                "Sync Created Room".to_string(),
                "SyncHost".to_string(),
                Some(6),
            )
            .unwrap();

        assert_eq!(response.room_name, "Sync Created Room");
        assert_eq!(response.host_player_name, "SyncHost");
        assert_eq!(response.max_players, 6);
        assert_eq!(response.current_players, 1);

        // Verify room appears in list
        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].id, response.room_id);
    }

    #[tokio::test]
    async fn test_join_room_sync() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        // Create room first
        let create_response = client
            .create_room_sync(
                "Join Test Room".to_string(),
                "JoinTestHost".to_string(),
                Some(4),
            )
            .unwrap();

        // Join room synchronously
        let join_response = client
            .join_room_sync(
                create_response.room_id,
                "JoinTestPlayer".to_string(),
                Some("LAX".to_string()),
            )
            .unwrap();

        assert_eq!(join_response.room_id, create_response.room_id);
        assert_eq!(join_response.player_name, "JoinTestPlayer");
        assert!(join_response.success);

        // Verify room now has 2 players
        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].current_players, 2);
    }

    #[tokio::test]
    async fn test_sync_async_api_consistency() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        // Create room with sync API
        let sync_response = client
            .create_room_sync(
                "Consistency Test".to_string(),
                "ConsistencyHost".to_string(),
                Some(4),
            )
            .unwrap();

        // List rooms with async API
        let async_rooms = client.list_rooms().await.unwrap();
        assert_eq!(async_rooms.len(), 1);
        assert_eq!(async_rooms[0].id, sync_response.room_id);

        // Join with async API
        let _async_join = client
            .join_room(
                sync_response.room_id,
                "AsyncJoiner".to_string(),
                Some("JFK".to_string()),
            )
            .await
            .unwrap();

        // Verify with sync API
        let sync_rooms = client.list_rooms_sync().unwrap();
        assert_eq!(sync_rooms.len(), 1);
        assert_eq!(sync_rooms[0].current_players, 2);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = GameApiClient::new("127.0.0.1:9999".to_string()); // Non-existent server

        // Test connection error
        let result = client.list_rooms_sync();
        assert!(result.is_err());

        match result {
            Err(ApiError::NetworkError(msg)) => {
                assert!(msg.contains("curl") || msg.contains("Failed"));
            },
            _ => panic!("Expected NetworkError"),
        }
    }

    #[tokio::test]
    async fn test_invalid_json_response() {
        // This test would need a mock server that returns invalid JSON
        // For now, we test the error handling path indirectly
        let client = GameApiClient::new("httpbin.org/status/500".to_string());

        let result = client.list_rooms_sync();
        // Should get some kind of error (network or parse)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_concurrent_sync_calls() {
        let port = start_test_server().await;
        let client = Arc::new(GameApiClient::new(format!("127.0.0.1:{}", port)));

        // Create a room first
        let create_response = client
            .create_room_sync(
                "Concurrent Test".to_string(),
                "ConcurrentHost".to_string(),
                Some(8),
            )
            .unwrap();

        // Spawn multiple concurrent sync calls
        let mut handles = Vec::new();

        for i in 0..3 {
            let client_clone = client.clone();
            let room_id = create_response.room_id;

            let handle = task::spawn_blocking(move || {
                client_clone.join_room_sync(
                    room_id,
                    format!("Player{}", i),
                    Some("LAX".to_string()),
                )
            });

            handles.push(handle);
        }

        // Wait for all to complete
        let mut successful_joins = 0;
        for handle in handles {
            if let Ok(Ok(_)) = handle.await {
                successful_joins += 1;
            }
        }

        assert!(successful_joins > 0);

        // Verify final state
        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 1);
        assert!(rooms[0].current_players > 1);
    }

    #[tokio::test]
    async fn test_room_creation_edge_cases() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        // Test minimum players
        let response1 = client
            .create_room_sync("Min Players".to_string(), "MinHost".to_string(), Some(1))
            .unwrap();
        assert_eq!(response1.max_players, 1);

        // Test maximum players
        let response2 = client
            .create_room_sync("Max Players".to_string(), "MaxHost".to_string(), Some(8))
            .unwrap();
        assert_eq!(response2.max_players, 8);

        // Test default players (None)
        let response3 = client
            .create_room_sync(
                "Default Players".to_string(),
                "DefaultHost".to_string(),
                None,
            )
            .unwrap();
        assert_eq!(response3.max_players, 4); // Default should be 4

        // Verify all rooms created
        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 3);
    }

    #[tokio::test]
    async fn test_special_characters_in_names() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        // Test special characters in room name
        let response = client
            .create_room_sync(
                "Test Room with Special chars: éñ中文🎮".to_string(),
                "Host with émojis 🎯".to_string(),
                Some(4),
            )
            .unwrap();

        assert_eq!(response.room_name, "Test Room with Special chars: éñ中文🎮");
        assert_eq!(response.host_player_name, "Host with émojis 🎯");

        // Join with special character name
        let join_response = client
            .join_room_sync(
                response.room_id,
                "Játékos 游戏者 🎮".to_string(),
                Some("JFK".to_string()),
            )
            .unwrap();

        assert_eq!(join_response.player_name, "Játékos 游戏者 🎮");

        // Verify in room list
        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].name, "Test Room with Special chars: éñ中文🎮");
    }
}
