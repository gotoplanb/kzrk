#[cfg(feature = "gui")]
use reqwest;
use serde_json;
use uuid::Uuid;

use crate::api::models::*;

#[derive(Clone)]
pub struct GameApiClient {
    #[allow(dead_code)]
    client: reqwest::Client,
    #[allow(dead_code)]
    base_url: String,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum ApiError {
    NetworkError(String),
    ParseError(String),
    ServerError(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::ParseError(err.to_string())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

#[allow(dead_code)]
impl GameApiClient {
    pub fn new(server_address: String) -> Self {
        // Support both full URLs and IP:port format
        let base_url =
            if server_address.starts_with("http://") || server_address.starts_with("https://") {
                // Remove trailing slash if present
                server_address.trim_end_matches('/').to_string()
            } else {
                // Legacy format: assume HTTP and IP:port
                format!("http://{}", server_address)
            };

        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn health_check(&self) -> Result<bool, ApiError> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    // Room management
    pub async fn create_room(
        &self,
        name: String,
        host_player_name: String,
        max_players: Option<usize>,
    ) -> Result<CreateRoomResponse, ApiError> {
        let request = CreateRoomRequest {
            name,
            host_player_name,
            max_players,
        };

        let response = self
            .client
            .post(format!("{}/rooms", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: CreateRoomResponse = response.json().await?;
        Ok(result)
    }

    pub async fn list_rooms(&self) -> Result<Vec<RoomInfo>, ApiError> {
        let response = self
            .client
            .get(format!("{}/rooms", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: Vec<RoomInfo> = response.json().await?;
        Ok(result)
    }

    pub async fn join_room(
        &self,
        room_id: Uuid,
        player_name: String,
        starting_airport: Option<String>,
    ) -> Result<JoinRoomResponse, ApiError> {
        let request = JoinRoomRequest {
            player_name,
            starting_airport,
        };

        let response = self
            .client
            .post(format!("{}/rooms/{}/join", self.base_url, room_id))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: JoinRoomResponse = response.json().await?;
        Ok(result)
    }

    pub async fn leave_room(
        &self,
        room_id: Uuid,
        player_id: Uuid,
    ) -> Result<LeaveRoomResponse, ApiError> {
        let response = self
            .client
            .post(format!(
                "{}/rooms/{}/players/{}/leave",
                self.base_url, room_id, player_id
            ))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: LeaveRoomResponse = response.json().await?;
        Ok(result)
    }

    // Game state
    pub async fn get_room_state(
        &self,
        room_id: Uuid,
        player_id: Uuid,
    ) -> Result<MultiplayerGameStateResponse, ApiError> {
        let response = self
            .client
            .get(format!(
                "{}/rooms/{}/players/{}/state",
                self.base_url, room_id, player_id
            ))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: MultiplayerGameStateResponse = response.json().await?;
        Ok(result)
    }

    // Player actions
    pub async fn player_travel(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        destination: String,
    ) -> Result<PlayerTravelResponse, ApiError> {
        let request = TravelRequest { destination };

        let response = self
            .client
            .post(format!(
                "{}/rooms/{}/players/{}/travel",
                self.base_url, room_id, player_id
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: PlayerTravelResponse = response.json().await?;
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn player_trade(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        cargo_type: String,
        quantity: u32,
        action: TradeAction,
    ) -> Result<PlayerTradeResponse, ApiError> {
        let request = TradeRequest {
            cargo_type,
            quantity,
            action,
        };

        let response = self
            .client
            .post(format!(
                "{}/rooms/{}/players/{}/trade",
                self.base_url, room_id, player_id
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: PlayerTradeResponse = response.json().await?;
        Ok(result)
    }

    pub async fn player_buy_fuel(
        &self,
        room_id: Uuid,
        player_id: Uuid,
        quantity: u32,
    ) -> Result<PlayerFuelResponse, ApiError> {
        let request = FuelRequest { quantity };

        let response = self
            .client
            .post(format!(
                "{}/rooms/{}/players/{}/fuel",
                self.base_url, room_id, player_id
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: PlayerFuelResponse = response.json().await?;
        Ok(result)
    }

    // Reference data
    pub async fn get_available_airports(&self) -> Result<serde_json::Value, ApiError> {
        let response = self
            .client
            .get(format!("{}/airports", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    pub async fn get_available_cargo(&self) -> Result<serde_json::Value, ApiError> {
        let response = self
            .client
            .get(format!("{}/cargo", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(ApiError::ServerError(error.message));
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    // Synchronous versions for GUI using curl (when gui feature is enabled)
    #[cfg(feature = "gui")]
    pub fn list_rooms_sync(&self) -> Result<Vec<RoomInfo>, ApiError> {
        let output = std::process::Command::new("curl")
            .arg("-s") // silent
            .arg("-X")
            .arg("GET")
            .arg(format!("{}/rooms", self.base_url))
            .output()
            .map_err(|e| ApiError::NetworkError(format!("Failed to execute curl: {}", e)))?;

        if !output.status.success() {
            return Err(ApiError::NetworkError("Curl command failed".to_string()));
        }

        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| ApiError::ParseError(format!("Invalid UTF-8 response: {}", e)))?;

        let result: Vec<RoomInfo> = serde_json::from_str(&response_text)?;
        Ok(result)
    }

    #[cfg(feature = "gui")]
    pub fn create_room_sync(
        &self,
        name: String,
        host_player_name: String,
        max_players: Option<usize>,
    ) -> Result<CreateRoomResponse, ApiError> {
        let request = CreateRoomRequest {
            name,
            host_player_name,
            max_players,
        };

        let request_json = serde_json::to_string(&request)?;

        let output = std::process::Command::new("curl")
            .arg("-s") // silent
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(request_json)
            .arg(format!("{}/rooms", self.base_url))
            .output()
            .map_err(|e| ApiError::NetworkError(format!("Failed to execute curl: {}", e)))?;

        if !output.status.success() {
            return Err(ApiError::NetworkError("Curl command failed".to_string()));
        }

        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| ApiError::ParseError(format!("Invalid UTF-8 response: {}", e)))?;

        let result: CreateRoomResponse = serde_json::from_str(&response_text)?;
        Ok(result)
    }

    #[cfg(feature = "gui")]
    pub fn join_room_sync(
        &self,
        room_id: Uuid,
        player_name: String,
        starting_airport: Option<String>,
    ) -> Result<JoinRoomResponse, ApiError> {
        let request = JoinRoomRequest {
            player_name,
            starting_airport,
        };

        let request_json = serde_json::to_string(&request)?;

        let output = std::process::Command::new("curl")
            .arg("-s") // silent
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(request_json)
            .arg(format!("{}/rooms/{}/join", self.base_url, room_id))
            .output()
            .map_err(|e| ApiError::NetworkError(format!("Failed to execute curl: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::NetworkError(format!(
                "Curl command failed: {}",
                stderr
            )));
        }

        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| ApiError::ParseError(format!("Invalid UTF-8 response: {}", e)))?;

        // Log the response for debugging
        eprintln!("Join room response: {}", response_text);

        // Try to parse as JoinRoomResponse first
        if let Ok(result) = serde_json::from_str::<JoinRoomResponse>(&response_text) {
            Ok(result)
        } else {
            // If that fails, try to parse as ErrorResponse
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                Err(ApiError::ServerError(error.message))
            } else {
                Err(ApiError::ParseError(format!(
                    "Failed to parse JSON response as either success or error: '{}'",
                    response_text
                )))
            }
        }
    }

    #[cfg(feature = "gui")]
    pub fn post_message_sync(
        &self,
        room_id: uuid::Uuid,
        player_id: uuid::Uuid,
        content: String,
    ) -> Result<PostMessageResponse, ApiError> {
        let request = PostMessageRequest { content };

        let request_json = serde_json::to_string(&request)?;

        let output = std::process::Command::new("curl")
            .arg("-s") // silent
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(request_json)
            .arg(format!("{}/rooms/{}/players/{}/messages", self.base_url, room_id, player_id))
            .output()
            .map_err(|e| ApiError::NetworkError(format!("Failed to execute curl: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::NetworkError(format!(
                "Curl command failed: {}",
                stderr
            )));
        }

        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| ApiError::ParseError(format!("Invalid UTF-8 response: {}", e)))?;

        // Try to parse as PostMessageResponse first
        if let Ok(result) = serde_json::from_str::<PostMessageResponse>(&response_text) {
            Ok(result)
        } else {
            // If that fails, try to parse as ErrorResponse
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                Err(ApiError::ServerError(error.message))
            } else {
                Err(ApiError::ParseError(format!(
                    "Failed to parse JSON response as either success or error: '{}'",
                    response_text
                )))
            }
        }
    }

    #[cfg(feature = "gui")]
    pub fn get_messages_sync(
        &self,
        room_id: uuid::Uuid,
        player_id: uuid::Uuid,
    ) -> Result<GetMessagesResponse, ApiError> {
        let output = std::process::Command::new("curl")
            .arg("-s") // silent
            .arg("-X")
            .arg("GET")
            .arg(format!("{}/rooms/{}/players/{}/messages", self.base_url, room_id, player_id))
            .output()
            .map_err(|e| ApiError::NetworkError(format!("Failed to execute curl: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::NetworkError(format!(
                "Curl command failed: {}",
                stderr
            )));
        }

        let response_text = String::from_utf8(output.stdout)
            .map_err(|e| ApiError::ParseError(format!("Invalid UTF-8 response: {}", e)))?;

        // Try to parse as GetMessagesResponse first
        if let Ok(result) = serde_json::from_str::<GetMessagesResponse>(&response_text) {
            Ok(result)
        } else {
            // If that fails, try to parse as ErrorResponse
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                Err(ApiError::ServerError(error.message))
            } else {
                Err(ApiError::ParseError(format!(
                    "Failed to parse JSON response as either success or error: '{}'",
                    response_text
                )))
            }
        }
    }
}
