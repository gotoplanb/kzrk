use crate::ui::{game_api_client::GameApiClient, scenes::Scene};
use eframe::egui;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

pub struct ServerConnectionScene {
    pub server_address: String,
    pub connection_state: ConnectionState,
    pub error_message: Option<String>,
}

impl Default for ServerConnectionScene {
    fn default() -> Self {
        Self {
            server_address: "http://127.0.0.1:3000".to_string(),
            connection_state: ConnectionState::Disconnected,
            error_message: None,
        }
    }
}

impl ServerConnectionScene {
    pub fn render(&mut self, ctx: &egui::Context) -> Option<(Scene, GameApiClient)> {
        let mut transition = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);

                ui.heading("🚁 KZRK Aviation Trading - Multiplayer");
                ui.add_space(20.0);

                ui.label("Connect to a KZRK server to play with others:");
                ui.add_space(10.0);

                // Server address input
                ui.horizontal(|ui| {
                    ui.label("Server URL:");
                    let response = ui.text_edit_singleline(&mut self.server_address);
                    if response.gained_focus() {
                        response.request_focus();
                    }
                });
                ui.label("Examples: http://127.0.0.1:3000, https://zeromission.ngrok.app");

                ui.add_space(10.0);

                // Connection status
                match &self.connection_state {
                    ConnectionState::Disconnected => {
                        if ui.button("Connect").clicked() {
                            self.connection_state = ConnectionState::Connecting;
                            self.error_message = None;

                            // Start connection attempt
                            let client = GameApiClient::new(self.server_address.clone());

                            // For now, we'll do a simple transition. In a real async GUI app,
                            // you'd want to spawn a task for the health check
                            transition = Some((Scene::RoomLobby, client));
                        }
                    },
                    ConnectionState::Connecting => {
                        ui.spinner();
                        ui.label("Connecting to server...");
                    },
                    ConnectionState::Connected => {
                        ui.label("✅ Connected to server");
                        let client = GameApiClient::new(self.server_address.clone());
                        transition = Some((Scene::RoomLobby, client));
                    },
                    ConnectionState::Error(msg) => {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("❌ Connection failed: {}", msg),
                        );
                        if ui.button("Retry").clicked() {
                            self.connection_state = ConnectionState::Connecting;
                            self.error_message = None;
                        }
                    },
                }

                if let Some(error) = &self.error_message {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.add_space(30.0);

                // Instructions
                ui.separator();
                ui.add_space(10.0);
                ui.label("💡 Instructions:");
                ui.label("• For local server: cargo run api (uses http://127.0.0.1:3000)");
                ui.label(
                    "• For ngrok: Use the HTTPS URL from ngrok (e.g., https://your-name.ngrok.app)",
                );
                ui.label("• Supports both HTTP and HTTPS connections");
                ui.label("• Multiple clients can connect to the same server");
                ui.add_space(10.0);
                ui.separator();
            });
        });

        transition
    }
}
