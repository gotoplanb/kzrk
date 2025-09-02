#[cfg(feature = "gui")]
mod gui_tests {
    use kzrk::ui::game_api_client::GameApiClient;

    // Basic tests that don't require running servers (safer for CI)
    #[tokio::test]
    async fn test_client_creation() {
        let _client = GameApiClient::new("127.0.0.1:3000".to_string());
        // Just verify the client can be created without panicking
        // This test passes if no panic occurs during client creation
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = GameApiClient::new("127.0.0.1:9999".to_string()); // Non-existent server

        // Test connection error
        let result = client.list_rooms_sync();
        assert!(result.is_err());
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

    // Note: Server-based tests are commented out as they can hang in CI environments
    // These tests would need a more robust server setup with proper timeouts and error handling
    // to work reliably in CI environments where networking might be restricted

    /*
    // Example of what full integration tests would look like:

    async fn start_test_server() -> u16 {
        use std::sync::Arc;
        use tokio::task;
        use tower_http::cors::CorsLayer;
        use kzrk::api::multiplayer_service::MultiplayerGameService;

        // Use in-memory database for tests to avoid persistence conflicts
        let service = Arc::new(MultiplayerGameService::new_in_memory());
        let app = kzrk::api::routes::create_multiplayer_router((*service).clone())
            .layer(CorsLayer::permissive());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        task::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        port
    }

    #[tokio::test]
    async fn test_list_rooms_sync() {
        let port = start_test_server().await;
        let client = GameApiClient::new(format!("127.0.0.1:{}", port));

        let rooms = client.list_rooms_sync().unwrap();
        assert!(rooms.is_empty());

        let create_response = client
            .create_room("Test Room".to_string(), "TestHost".to_string(), Some(4))
            .await
            .unwrap();

        let rooms = client.list_rooms_sync().unwrap();
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].name, "Test Room");
        assert_eq!(rooms[0].id, create_response.room_id);
    }
    */
}
