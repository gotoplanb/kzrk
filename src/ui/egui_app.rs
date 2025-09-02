use crate::{
    api::models::MultiplayerGameStateResponse,
    ui::{
        game_api_client::GameApiClient,
        scenes::{
            Scene, SceneState,
            room_lobby::{GameSession, RoomLobbyScene},
            server_connection::ServerConnectionScene,
        },
    },
};
use eframe::egui;

#[derive(Debug, Clone)]
pub enum AppState {
    ServerConnection,
    RoomLobby,
    InGame(GameSession),
}

pub struct KzrkEguiApp {
    app_state: AppState,
    scene_state: SceneState,
    api_client: Option<GameApiClient>,
    game_state: Option<MultiplayerGameStateResponse>,
    converted_game_state: Option<crate::systems::game::GameState>, // Cache converted state
    last_local_action: Option<std::time::Instant>,                 // Track recent local actions
    server_connection_scene: ServerConnectionScene,
    room_lobby_scene: RoomLobbyScene,
    last_state_refresh: std::time::Instant,
}

impl Default for KzrkEguiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl KzrkEguiApp {
    pub fn new() -> Self {
        Self {
            app_state: AppState::ServerConnection,
            scene_state: SceneState::new(),
            api_client: None,
            game_state: None,
            converted_game_state: None,
            last_local_action: None,
            server_connection_scene: ServerConnectionScene::default(),
            room_lobby_scene: RoomLobbyScene::default(),
            last_state_refresh: std::time::Instant::now(),
        }
    }
}

impl eframe::App for KzrkEguiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Debug: Show current app state in title bar
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "KZRK - State: {:?}",
            self.app_state
        )));

        match &self.app_state.clone() {
            AppState::ServerConnection => {
                if let Some((scene, client)) = self.server_connection_scene.render(ctx) {
                    self.api_client = Some(client);
                    if scene == Scene::RoomLobby {
                        self.app_state = AppState::RoomLobby
                    }
                }
            },
            AppState::RoomLobby => {
                if let Some(client) = &self.api_client
                    && let Some((scene, session)) = self.room_lobby_scene.render(ctx, client)
                {
                    self.app_state = AppState::InGame(session);
                    if let Scene::Airport(airport_id) = scene {
                        self.scene_state.travel_to_airport(airport_id);
                    }
                }
            },
            AppState::InGame(session) => {
                // Refresh game state periodically
                if self.last_state_refresh.elapsed().as_secs() >= 2 {
                    self.refresh_game_state(session);
                }

                // Render airport scene with multiplayer data
                match &self.scene_state.current_scene {
                    Scene::Airport(_airport) => {
                        if let Some(multiplayer_state) = &self.game_state {
                            // Only convert/update the cached state if needed
                            if self.converted_game_state.is_none() {
                                self.converted_game_state = self
                                    .convert_multiplayer_to_game_state(multiplayer_state, session);
                            }

                            // Clone the multiplayer state and session for the update
                            let multiplayer_state_clone = multiplayer_state.clone();
                            let session_clone = session.clone();

                            // Use the cached converted state
                            if let Some(converted_state) = &mut self.converted_game_state {
                                // Update only the player data from multiplayer state (dynamic data)
                                // Skip update if there was a recent local action (within 5 seconds)
                                let skip_update = if let Some(last_action) = self.last_local_action
                                {
                                    last_action.elapsed().as_secs() < 5
                                } else {
                                    false
                                };

                                if !skip_update {
                                    Self::update_converted_state_player_static(
                                        converted_state,
                                        &multiplayer_state_clone,
                                        &session_clone,
                                    );
                                }

                                // Use a custom multiplayer-aware render that handles API calls
                                if let Some(action_time) =
                                    Self::render_multiplayer_airport_scene_static(
                                        converted_state,
                                        &mut self.scene_state,
                                        &session_clone,
                                        ctx,
                                    )
                                {
                                    self.last_local_action = Some(action_time);
                                }
                            }
                        } else {
                            // Loading state
                            egui::CentralPanel::default().show(ctx, |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label("Loading game state...");
                                    });
                                });
                            });
                        }
                    },
                    _other_scene => {
                        // Default to airport if we're in an unexpected scene
                        self.scene_state.travel_to_airport("JFK".to_string());
                    },
                }
            },
        }
    }
}

impl KzrkEguiApp {
    fn refresh_game_state(&mut self, session: &GameSession) {
        self.last_state_refresh = std::time::Instant::now();

        // TODO: Implement proper async state fetching
        // For now, we'll create a mock state to avoid the tokio runtime crash
        // In a production app, you'd use proper async channels or polling_promise

        // Always refresh state, but preserve local changes during action protection window
        // Create/update mock state for testing
        use crate::api::models::*;
        use crate::systems::GameStatus;
        use chrono::Utc;

        // Get the current location from converted state if available (to preserve travel)
        let current_location = if let Some(converted_state) = &self.converted_game_state {
            converted_state.player.current_airport.clone()
        } else {
            "JFK".to_string()
        };

        // Get current player state to preserve changes during local action window
        let (current_money, current_fuel, current_cargo) =
            if let Some(converted_state) = &self.converted_game_state {
                (
                    converted_state.player.money,
                    converted_state.player.fuel,
                    converted_state.player.cargo_inventory.clone(),
                )
            } else {
                use crate::models::cargo::CargoInventory;
                (5000, 200, CargoInventory::new())
            };

        let mock_state = MultiplayerGameStateResponse {
            room_info: RoomInfo {
                id: session.room_id,
                name: "Test Room".to_string(),
                host_player_name: session.player_name.clone(),
                current_players: 1,
                max_players: 4,
                created_at: Utc::now(),
                game_status: GameStatus::WaitingForPlayers,
                is_joinable: true,
            },
            my_player_id: session.player_id,
            players: vec![PlayerInfo {
                id: Some(session.player_id),
                name: session.player_name.clone(),
                money: current_money,
                current_airport: current_location.clone(),
                fuel: current_fuel,
                max_fuel: 200,
                cargo_inventory: current_cargo
                    .get_all_cargo()
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect(),
                cargo_weight: current_cargo.get_all_cargo().values().sum::<u32>(),
                max_cargo_weight: 1000,
                fuel_efficiency: 15.0,
                is_online: Some(true),
                last_seen: Some(Utc::now()),
                is_host: Some(true),
            }],
            current_market: MarketInfo {
                airport_id: current_location.clone(),
                airport_name: match current_location.as_str() {
                    "JFK" => "New York JFK".to_string(),
                    "LAX" => "Los Angeles LAX".to_string(),
                    "MIA" => "Miami MIA".to_string(),
                    "ORD" => "Chicago O'Hare".to_string(),
                    "DEN" => "Denver DEN".to_string(),
                    "SEA" => "Seattle SEA".to_string(),
                    _ => "Unknown Airport".to_string(),
                },
                fuel_price: 50,
                cargo_prices: {
                    let mut prices = std::collections::HashMap::new();
                    prices.insert("electronics".to_string(), 500);
                    prices.insert("food".to_string(), 100);
                    prices.insert("textiles".to_string(), 200);
                    prices.insert("industrial".to_string(), 300);
                    prices.insert("luxury".to_string(), 1000);
                    prices.insert("materials".to_string(), 50);
                    prices
                },
                last_updated: std::time::SystemTime::now(),
            },
            available_destinations: vec![
                DestinationInfo {
                    airport_id: "LAX".to_string(),
                    airport_name: "Los Angeles LAX".to_string(),
                    distance: 3974.0,
                    fuel_required: 150,
                    can_travel: true,
                    fuel_price: 45,
                },
                DestinationInfo {
                    airport_id: "MIA".to_string(),
                    airport_name: "Miami MIA".to_string(),
                    distance: 1757.0,
                    fuel_required: 80,
                    can_travel: true,
                    fuel_price: 55,
                },
                DestinationInfo {
                    airport_id: "ORD".to_string(),
                    airport_name: "Chicago O'Hare".to_string(),
                    distance: 1188.0,
                    fuel_required: 60,
                    can_travel: true,
                    fuel_price: 50,
                },
            ],
            statistics: StatisticsInfo {
                total_revenue: 0,
                total_expenses: 0,
                net_profit: 0,
                cargo_trades: 0,
                fuel_purchased: 0,
                distances_traveled: 0.0,
                airports_visited: vec![],
                best_single_trade: 0,
                most_profitable_cargo: "".to_string(),
                efficiency_score: 0.0,
            },
            turn_number: 1,
            world_time: Utc::now(),
        };

        self.game_state = Some(mock_state);
        // Only clear cache if we don't have one yet - keep it stable for UI consistency
        // Cache will be updated in place through the update_converted_state_player_static method
    }

    fn convert_multiplayer_to_game_state(
        &self,
        multiplayer_state: &MultiplayerGameStateResponse,
        session: &GameSession,
    ) -> Option<crate::systems::game::GameState> {
        use crate::models::{Player, cargo::CargoInventory};
        use crate::systems::game::GameState;
        use std::collections::HashMap;

        // Find the current player
        let my_player = multiplayer_state
            .players
            .iter()
            .find(|p| p.id == Some(session.player_id))?;

        // Create CargoInventory from HashMap
        let mut cargo_inventory = CargoInventory::new();
        for (cargo_type, quantity) in &my_player.cargo_inventory {
            cargo_inventory.add_cargo(cargo_type, *quantity);
        }

        // Create Player from multiplayer data
        let player = Player {
            money: my_player.money,
            current_airport: my_player.current_airport.clone(),
            fuel: my_player.fuel,
            max_fuel: my_player.max_fuel,
            cargo_inventory,
            max_cargo_weight: my_player.max_cargo_weight,
            fuel_efficiency: my_player.fuel_efficiency,
        };

        // Load the default airports and cargo types (same as single-player)
        let airports = crate::data::airports::get_default_airports();
        let cargo_types = crate::data::cargo_types::get_default_cargo_types();

        // Create markets for each airport with current multiplayer prices
        let mut markets = HashMap::new();
        for airport_id in airports.keys() {
            let mut market = crate::models::Market::new(airport_id, 50); // Default fuel price

            // Set cargo prices from multiplayer data if available
            if airport_id == &multiplayer_state.current_market.airport_id {
                // Use the current market prices from multiplayer state
                market.update_fuel_price(multiplayer_state.current_market.fuel_price);
                for (cargo_type, price) in &multiplayer_state.current_market.cargo_prices {
                    market.set_cargo_price(cargo_type, *price);
                }
            } else {
                // Set default prices for other airports
                for (cargo_type_id, cargo_type) in &cargo_types {
                    market.set_cargo_price(cargo_type_id, cargo_type.base_price);
                }
            }
            markets.insert(airport_id.clone(), market);
        }

        // Create distance cache with pre-calculated distances between all airports
        let mut distance_cache = HashMap::new();
        for (from_id, from_airport) in &airports {
            for (to_id, to_airport) in &airports {
                if from_id != to_id {
                    // Calculate distance using the airport coordinates
                    let distance =
                        Self::calculate_distance(from_airport.coordinates, to_airport.coordinates);
                    let key = format!("{}-{}", from_id, to_id);
                    distance_cache.insert(key, distance);
                }
            }
        }

        Some(GameState {
            player,
            airports,
            cargo_types,
            markets,
            distance_cache,
            turn_number: multiplayer_state.turn_number,
            cheat_mode: false,
            stats: crate::models::GameStats::new(5000), // Default starting money
            win_condition_money: 100000,                // Default win condition
            active_events: Vec::new(),
        })
    }

    fn update_converted_state_player_static(
        converted_state: &mut crate::systems::game::GameState,
        multiplayer_state: &MultiplayerGameStateResponse,
        session: &GameSession,
    ) {
        // Find the current player in multiplayer state
        if let Some(my_player) = multiplayer_state
            .players
            .iter()
            .find(|p| p.id == Some(session.player_id))
        {
            // Only update values that have actually changed to avoid unnecessary UI refreshes
            if converted_state.player.money != my_player.money {
                converted_state.player.money = my_player.money;
            }
            if converted_state.player.current_airport != my_player.current_airport {
                converted_state.player.current_airport = my_player.current_airport.clone();
            }
            if converted_state.player.fuel != my_player.fuel {
                converted_state.player.fuel = my_player.fuel;
            }

            // Check if cargo inventory actually changed before recreating it
            let current_cargo: std::collections::HashMap<String, u32> = converted_state
                .player
                .cargo_inventory
                .get_all_cargo()
                .clone();
            if current_cargo != my_player.cargo_inventory {
                converted_state.player.cargo_inventory =
                    crate::models::cargo::CargoInventory::new();
                for (cargo_type, quantity) in &my_player.cargo_inventory {
                    converted_state
                        .player
                        .cargo_inventory
                        .add_cargo(cargo_type, *quantity);
                }
            }

            // Only update turn number if it changed
            if converted_state.turn_number != multiplayer_state.turn_number {
                converted_state.turn_number = multiplayer_state.turn_number;
            }
        }
    }

    fn calculate_distance(coord1: (f64, f64), coord2: (f64, f64)) -> f64 {
        // Simple Euclidean distance calculation (in a real app, you'd use great circle distance)
        let dx = coord1.0 - coord2.0;
        let dy = coord1.1 - coord2.1;
        // Convert to approximate kilometers (rough approximation)
        let distance_deg = (dx * dx + dy * dy).sqrt();
        distance_deg * 111.0 // Rough conversion from degrees to km
    }

    fn render_multiplayer_airport_scene_static(
        converted_state: &mut crate::systems::game::GameState,
        scene_state: &mut SceneState,
        _session: &GameSession,
        ctx: &egui::Context,
    ) -> Option<std::time::Instant> {
        // Store the original state to detect changes
        let original_money = converted_state.player.money;
        let original_fuel = converted_state.player.fuel;
        let original_location = converted_state.player.current_airport.clone();
        let original_cargo = converted_state.player.cargo_inventory.clone();

        // Render the original scene
        crate::ui::scenes::airport::AirportScene::render(converted_state, scene_state, ctx);

        // Track changes for action detection (API calls would go here in full implementation)

        // Return timestamp if any action occurred
        let action_occurred = converted_state.player.money != original_money
            || converted_state.player.fuel != original_fuel
            || converted_state.player.current_airport != original_location
            || converted_state.player.cargo_inventory != original_cargo;

        if action_occurred {
            // Action detected - would make appropriate API calls in full implementation
            Some(std::time::Instant::now())
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn render_multiplayer_airport(
        &mut self,
        ctx: &egui::Context,
        session: &GameSession,
        game_state: &MultiplayerGameStateResponse,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("✈️ KZRK Aviation Trading Game");
            ui.heading(format!("🏢 Room: {}", game_state.room_info.name));
            ui.label(format!(
                "📍 Current Location: {}",
                game_state.current_market.airport_name
            ));
            ui.separator();

            // Player list
            ui.collapsing("👥 Players in Room", |ui| {
                for player in &game_state.players {
                    ui.horizontal(|ui| {
                        if player.id == Some(session.player_id) {
                            ui.label("➤");
                        } else {
                            ui.label("  ");
                        }
                        ui.label(&player.name);
                        ui.label(format!("@{}", player.current_airport));
                        ui.label(format!("${}", player.money));
                        if player.is_host == Some(true) {
                            ui.label("👑 Host");
                        }
                        if player.is_online == Some(false) {
                            ui.colored_label(egui::Color32::GRAY, "Offline");
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Current player info
            if let Some(my_player) = game_state
                .players
                .iter()
                .find(|p| p.id == Some(session.player_id))
            {
                ui.horizontal(|ui| {
                    ui.label(format!("💰 Money: ${}", my_player.money));
                    ui.label(format!(
                        "⛽ Fuel: {}/{}",
                        my_player.fuel, my_player.max_fuel
                    ));
                    ui.label(format!(
                        "📦 Cargo: {}/{} kg",
                        my_player.cargo_weight, my_player.max_cargo_weight
                    ));
                });
                ui.separator();
                ui.add_space(10.0);
            }

            // Market info
            ui.heading(format!(
                "🏪 {} Market",
                game_state.current_market.airport_name
            ));
            ui.label(format!(
                "⛽ Fuel Price: ${}/unit",
                game_state.current_market.fuel_price
            ));

            ui.collapsing("📈 Cargo Trading", |ui| {
                for (cargo_type, price) in &game_state.current_market.cargo_prices {
                    ui.horizontal(|ui| {
                        ui.label(cargo_type);
                        ui.label(format!("${}/unit", price));

                        // Buy cargo
                        if ui.button("📈 Buy 1").clicked() {
                            // TODO: Implement buy cargo via API
                            println!("Buy 1 unit of {} for ${}", cargo_type, price);
                        }

                        // Sell cargo (if player has some)
                        if let Some(my_player) = game_state
                            .players
                            .iter()
                            .find(|p| p.id == Some(session.player_id))
                            && let Some(&quantity) = my_player.cargo_inventory.get(cargo_type)
                            && quantity > 0
                        {
                            ui.label(format!("Have: {}", quantity));
                            if ui.button("📉 Sell 1").clicked() {
                                // TODO: Implement sell cargo via API
                                println!("Sell 1 unit of {} for ${}", cargo_type, price);
                            }
                        }
                    });
                }

                ui.add_space(5.0);

                // Fuel purchase section
                ui.horizontal(|ui| {
                    ui.label("⛽ Fuel:");
                    ui.label(format!("${}/unit", game_state.current_market.fuel_price));
                    if ui.button("⛽ Buy 10 units").clicked() {
                        // TODO: Implement fuel purchase via API
                        println!(
                            "Buy 10 units of fuel for ${}",
                            game_state.current_market.fuel_price * 10
                        );
                    }
                    if ui.button("⛽ Fill tank").clicked() {
                        // TODO: Implement fill tank via API
                        println!("Fill fuel tank");
                    }
                });
            });

            ui.add_space(10.0);

            // Available destinations
            ui.collapsing("✈️ Available Destinations", |ui| {
                for dest in &game_state.available_destinations {
                    ui.horizontal(|ui| {
                        ui.label(&dest.airport_name);
                        ui.label(format!("{:.0} km", dest.distance));
                        ui.label(format!("⛽ {}", dest.fuel_required));
                        if dest.can_travel {
                            if ui.button("✈️ Fly").clicked() {
                                // TODO: Implement travel action via API
                                println!(
                                    "Flying to {} (fuel cost: {})",
                                    dest.airport_name, dest.fuel_required
                                );
                            }
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Not enough fuel");
                        }
                    });
                }
            });

            ui.add_space(20.0);

            // Disconnect button
            if ui.button("🔌 Disconnect").clicked() {
                // TODO: Leave room via API - for now just disconnect
                println!("Leaving room: {}", game_state.room_info.name);
                self.app_state = AppState::ServerConnection;
                self.api_client = None;
                self.game_state = None;
                self.converted_game_state = None; // Clear cached state
            }
        });
    }
}
