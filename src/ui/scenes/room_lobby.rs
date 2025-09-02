use crate::{
    api::models::{PlayerSessionInfo, RoomInfo},
    ui::{
        game_api_client::{ApiError, GameApiClient},
        scenes::Scene,
    },
};
use eframe::egui;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum LobbyState {
    Loading,
    ShowingRooms,
    CreatingRoom,
    JoiningRoom(Uuid),
    Error(String),
}

pub struct RoomLobbyScene {
    pub lobby_state: LobbyState,
    pub available_rooms: Vec<RoomInfo>,
    pub player_name: String,
    pub previous_player_name: String,
    pub existing_sessions: Vec<PlayerSessionInfo>,
    pub create_room_name: String,
    pub create_room_max_players: usize,
    pub error_message: Option<String>,
    pub last_refresh: std::time::Instant,
}

impl Default for RoomLobbyScene {
    fn default() -> Self {
        Self {
            lobby_state: LobbyState::Loading,
            available_rooms: Vec::new(),
            player_name: "Player".to_string(),
            previous_player_name: String::new(),
            existing_sessions: Vec::new(),
            create_room_name: "My Game Room".to_string(),
            create_room_max_players: 4,
            error_message: None,
            last_refresh: std::time::Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub room_id: Uuid,
    pub player_id: Uuid,
    pub player_name: String,
}

impl RoomLobbyScene {
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        client: &GameApiClient,
    ) -> Option<(Scene, GameSession)> {
        let mut transition = None;

        // Auto-refresh rooms every 5 seconds
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.refresh_rooms(client);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🏢 Game Rooms");
            ui.separator();
            ui.add_space(10.0);

            // Player name input
            ui.horizontal(|ui| {
                ui.label("Your name:");
                let response = ui.text_edit_singleline(&mut self.player_name);

                // Check for existing sessions when player name changes
                if response.changed() && self.player_name != self.previous_player_name {
                    self.previous_player_name = self.player_name.clone();
                    if !self.player_name.trim().is_empty() {
                        self.check_existing_sessions(client);
                    } else {
                        self.existing_sessions.clear();
                    }
                }

                if ui.button("🔄 Refresh Rooms").clicked() {
                    self.refresh_rooms(client);
                }
            });

            // Show existing sessions if any
            if !self.existing_sessions.is_empty() {
                ui.add_space(5.0);
                ui.group(|ui| {
                    ui.strong("🔄 Resume Previous Games:");
                    ui.separator();
                    for session in &self.existing_sessions {
                        ui.horizontal(|ui| {
                            ui.label(format!("📍 {}", session.room_name));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Resume").clicked() {
                                    // Note: This would need proper async handling in a real app
                                    // For now, we'll set the lobby state to indicate joining
                                    self.lobby_state = LobbyState::JoiningRoom(session.room_id);
                                }
                            });
                        });
                    }
                });
            }

            ui.add_space(10.0);

            // Create room section
            ui.collapsing("🆕 Create New Room", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Room Name:");
                    ui.text_edit_singleline(&mut self.create_room_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Max Players:");
                    ui.add(egui::Slider::new(&mut self.create_room_max_players, 1..=8));
                });

                if ui.button("Create Room").clicked() {
                    if !self.player_name.trim().is_empty() && !self.create_room_name.trim().is_empty() {
                        self.lobby_state = LobbyState::CreatingRoom;
                        // In a real async app, you'd spawn a task here
                        // For now, we'll simulate immediate response
                        match self.create_room_sync(client) {
                            Ok(session) => transition = Some((Scene::Airport("JFK".to_string()), session)),
                            Err(e) => {
                                self.error_message = Some(format!("Failed to create room: {:?}", e));
                                self.lobby_state = LobbyState::Error(format!("{:?}", e));
                            }
                        }
                    } else {
                        self.error_message = Some("Please enter your name and room name".to_string());
                    }
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Room list
            ui.label("Available Rooms:");
            ui.add_space(5.0);

            match &self.lobby_state {
                LobbyState::Loading => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading rooms...");
                    });
                },
                LobbyState::ShowingRooms => {
                    if self.available_rooms.is_empty() {
                        ui.label("No rooms available. Create one to start playing!");
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let available_rooms = self.available_rooms.clone();
                            for room in &available_rooms {
                                ui.horizontal(|ui| {
                                    ui.group(|ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.heading(&room.name);
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if room.is_joinable && ui.button("Join").clicked() {
                                                        if !self.player_name.trim().is_empty() {
                                                            self.lobby_state = LobbyState::JoiningRoom(room.id);
                                                            match self.join_room_sync(client, room.id) {
                                                                Ok(session) => transition = Some((Scene::Airport("JFK".to_string()), session)),
                                                                Err(e) => {
                                                                    self.error_message = Some(format!("Failed to join room: {:?}", e));
                                                                    self.lobby_state = LobbyState::Error(format!("{:?}", e));
                                                                }
                                                            }
                                                        } else {
                                                            self.error_message = Some("Please enter your name".to_string());
                                                        }
                                                    }
                                                    if !room.is_joinable {
                                                        ui.label("🚫 Full/In Progress");
                                                    }
                                                });
                                            });

                                            ui.horizontal(|ui| {
                                                ui.label(format!("Host: {}", room.host_player_name));
                                                ui.label(format!("Players: {}/{}", room.current_players, room.max_players));
                                                ui.label(format!("Status: {:?}", room.game_status));
                                            });
                                        });
                                    });
                                });
                                ui.add_space(5.0);
                            }
                        });
                    }
                },
                LobbyState::CreatingRoom => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Creating room...");
                    });
                },
                LobbyState::JoiningRoom(room_id) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("Joining room {}...", room_id));
                    });
                },
                LobbyState::Error(msg) => {
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", msg));
                    if ui.button("Retry").clicked() {
                        self.lobby_state = LobbyState::ShowingRooms;
                        self.error_message = None;
                        self.refresh_rooms(client);
                    }
                },
            }

            if let Some(error) = &self.error_message {
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::RED, error);
                if ui.button("Clear Error").clicked() {
                    self.error_message = None;
                }
            }
        });

        transition
    }

    fn refresh_rooms(&mut self, client: &GameApiClient) {
        self.lobby_state = LobbyState::Loading;
        self.last_refresh = std::time::Instant::now();

        match client.list_rooms_sync() {
            Ok(rooms) => {
                self.available_rooms = rooms;
                self.lobby_state = LobbyState::ShowingRooms;
            },
            Err(err) => {
                self.available_rooms.clear();
                self.lobby_state = LobbyState::Error(format!("Failed to fetch rooms: {}", err));
            },
        }
    }

    fn create_room_sync(&mut self, client: &GameApiClient) -> Result<GameSession, ApiError> {
        let response = client.create_room_sync(
            self.create_room_name.clone(),
            self.player_name.clone(),
            Some(self.create_room_max_players), // Use configured max players
        )?;

        Ok(GameSession {
            room_id: response.room_id,
            player_id: response.host_player_id,
            player_name: response.host_player_name,
        })
    }

    fn join_room_sync(
        &mut self,
        client: &GameApiClient,
        room_id: Uuid,
    ) -> Result<GameSession, ApiError> {
        let response = client.join_room_sync(
            room_id,
            self.player_name.clone(),
            Some("JFK".to_string()), // Default starting airport
        )?;

        Ok(GameSession {
            room_id: response.room_id,
            player_id: response.player_id,
            player_name: response.player_name,
        })
    }

    fn check_existing_sessions(&mut self, _client: &GameApiClient) {
        // In a real async GUI app, you'd use proper async/await to call:
        // client.find_player_sessions(&self.player_name)
        // For now, this is a placeholder that simulates finding sessions

        // Clear existing sessions first
        self.existing_sessions.clear();

        // TODO: Implement actual API call when proper async support is added
        // This would make an HTTP GET request to /players/{player_name}/sessions
    }
}
