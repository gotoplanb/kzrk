use crate::{
    systems::{game::GameState, trading::TradingSystem, travel::TravelSystem},
    ui::{
        game_api_client::GameApiClient,
        scenes::{Location, SceneState, room_lobby::GameSession},
    },
};

#[cfg(feature = "gui")]
#[allow(unused_imports)] // Only used in GUI feature
use crate::api::models::MessageInfo;

pub struct AirportScene;

impl AirportScene {
    pub fn render(
        game_state: &mut GameState,
        scene_state: &mut SceneState,
        ctx: &eframe::egui::Context,
        api_client: &GameApiClient,
        session: &GameSession,
    ) {
        // Get current airport info
        let current_airport = game_state
            .airports
            .get(&game_state.player.current_airport)
            .cloned();

        let airport_name = current_airport
            .as_ref()
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown Airport");

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            // Airport header
            ui.horizontal(|ui| {
                ui.heading(format!("🛩️ {} - Fixed Base Operation", airport_name));
                ui.with_layout(
                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                    |ui| {
                        ui.label(format!("Turn: {}", game_state.turn_number));
                    },
                );
            });

            ui.separator();

            // Player status bar - always visible
            Self::render_status_bar(game_state, ui);

            ui.separator();

            // FBO location buttons
            Self::render_location_buttons(scene_state, ui);

            ui.separator();

            // Current location content
            match scene_state.current_location {
                Location::MainDesk => Self::render_main_desk(game_state, scene_state, ui),
                Location::MarketBoard => Self::render_market_board(game_state, ui),
                Location::TradingDesk => Self::render_trading_desk(game_state, scene_state, ui),
                Location::FlightPlanning => {
                    Self::render_flight_planning(game_state, scene_state, ui)
                },
                Location::FuelPump => Self::render_fuel_pump(game_state, scene_state, ui),
                Location::MessageBoard => {
                    Self::render_message_board(game_state, scene_state, ui, api_client, session)
                },
            }
        });
    }

    fn render_status_bar(game_state: &GameState, ui: &mut eframe::egui::Ui) {
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_gray(240))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("💰 ${}", game_state.player.money));
                    ui.separator();
                    ui.label(format!(
                        "⛽ {}/{}",
                        game_state.player.fuel, game_state.player.max_fuel
                    ));
                    ui.separator();

                    let current_weight = game_state
                        .player
                        .current_cargo_weight(&game_state.cargo_types);
                    ui.label(format!(
                        "📦 {}kg/{}kg",
                        current_weight, game_state.player.max_cargo_weight
                    ));
                    ui.separator();

                    ui.label(format!(
                        "📍 {}",
                        game_state
                            .airports
                            .get(&game_state.player.current_airport)
                            .map(|a| &a.name)
                            .unwrap_or(&game_state.player.current_airport)
                    ));
                });
            });
    }

    fn render_location_buttons(scene_state: &mut SceneState, ui: &mut eframe::egui::Ui) {
        ui.heading("🏢 FBO Locations");

        ui.horizontal_wrapped(|ui| {
            let locations = [
                (Location::MainDesk, "🏠 Main Desk"),
                (Location::MarketBoard, "📊 Market Board"),
                (Location::TradingDesk, "💼 Trading Desk"),
                (Location::FlightPlanning, "✈️ Flight Planning"),
                (Location::FuelPump, "⛽ Fuel Pump"),
                (Location::MessageBoard, "💬 Message Board"),
            ];

            for (location, label) in locations {
                let is_current = scene_state.current_location == location;

                let button = if is_current {
                    eframe::egui::Button::new(format!("▶ {}", label))
                        .fill(eframe::egui::Color32::from_rgb(100, 150, 255))
                } else {
                    eframe::egui::Button::new(label)
                };

                if ui.add_sized([120.0, 32.0], button).clicked() && !is_current {
                    scene_state.go_to_location(location);
                }
            }
        });
    }

    fn render_main_desk(
        game_state: &GameState,
        _scene_state: &SceneState,
        ui: &mut eframe::egui::Ui,
    ) {
        ui.heading("🏠 Main Desk - Welcome, Pilot!");

        // Welcome message with airport info
        let current_airport = game_state.airports.get(&game_state.player.current_airport);
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_rgb(245, 250, 255))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                if let Some(airport) = current_airport {
                    ui.label(format!("\"Welcome to {} Fixed Base Operation! Good to see you, pilot. What can we help you with today?\"", airport.name));
                } else {
                    ui.label("\"Good to see you at our Fixed Base Operation. What can we help you with today?\"");
                }
            });

        ui.separator();

        // Game status overview with color coding
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_gray(248))
            .stroke(eframe::egui::Stroke::new(
                1.0,
                eframe::egui::Color32::from_gray(200),
            ))
            .inner_margin(eframe::egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.strong("📊 Flight Status Overview");
                ui.separator();

                eframe::egui::Grid::new("status_overview")
                    .num_columns(2)
                    .spacing([30.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Pilot Status:");
                        let (status_text, status_color) = if game_state.is_game_won() {
                            (
                                "🏆 WINNER! You've made $100,000!",
                                eframe::egui::Color32::from_rgb(255, 215, 0),
                            )
                        } else if game_state.can_player_continue() {
                            (
                                "✅ Active pilot - ready for business",
                                eframe::egui::Color32::from_rgb(50, 150, 50),
                            )
                        } else {
                            (
                                "⚠️ Low on fuel and funds - need assistance",
                                eframe::egui::Color32::from_rgb(220, 50, 50),
                            )
                        };
                        ui.colored_label(status_color, status_text);
                        ui.end_row();

                        ui.label("Current Funds:");
                        let money_color = if game_state.player.money > 50000 {
                            eframe::egui::Color32::from_rgb(50, 150, 50)
                        } else if game_state.player.money > 10000 {
                            eframe::egui::Color32::from_rgb(255, 140, 0)
                        } else if game_state.player.money > 1000 {
                            eframe::egui::Color32::from_rgb(200, 100, 50)
                        } else {
                            eframe::egui::Color32::from_rgb(220, 50, 50)
                        };
                        ui.colored_label(money_color, format!("${}", game_state.player.money));
                        ui.end_row();

                        ui.label("Cargo Manifest:");
                        let cargo_count = game_state.player.cargo_inventory.get_all_cargo().len();
                        let current_weight = game_state
                            .player
                            .current_cargo_weight(&game_state.cargo_types);
                        ui.label(format!(
                            "{} types | {}kg/{} max",
                            cargo_count, current_weight, game_state.player.max_cargo_weight
                        ));
                        ui.end_row();

                        ui.label("Fuel Status:");
                        let fuel_percent = (game_state.player.fuel as f32
                            / game_state.player.max_fuel as f32)
                            * 100.0;
                        let fuel_color = if fuel_percent > 75.0 {
                            eframe::egui::Color32::from_rgb(50, 150, 50)
                        } else if fuel_percent > 25.0 {
                            eframe::egui::Color32::from_rgb(255, 140, 0)
                        } else {
                            eframe::egui::Color32::from_rgb(220, 50, 50)
                        };
                        ui.colored_label(
                            fuel_color,
                            format!(
                                "{:.0}% capacity ({}/{})",
                                fuel_percent, game_state.player.fuel, game_state.player.max_fuel
                            ),
                        );
                        ui.end_row();

                        ui.label("Game Progress:");
                        ui.label(format!("Turn {} | Goal: $100,000", game_state.turn_number));
                        ui.end_row();
                    });
            });

        ui.separator();

        // Current cargo inventory (if any)
        let inventory = game_state.player.cargo_inventory.get_all_cargo();
        if !inventory.is_empty() {
            ui.collapsing("📦 Current Cargo Inventory", |ui| {
                eframe::egui::Grid::new("inventory_display")
                    .num_columns(4)
                    .spacing([20.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Cargo Type");
                        ui.strong("Quantity");
                        ui.strong("Weight");
                        ui.strong("Estimated Value");
                        ui.end_row();

                        for (cargo_id, quantity) in inventory {
                            if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                                let icon = match cargo_type.name.as_str() {
                                    "Electronics" => "💻",
                                    "Food & Beverages" => "🍎",
                                    "Textiles" => "👔",
                                    "Industrial Parts" => "🔧",
                                    "Luxury Goods" => "💎",
                                    "Raw Materials" => "🏗️",
                                    _ => "📦",
                                };
                                ui.label(format!("{} {}", icon, cargo_type.name));
                                ui.label(format!("{}", quantity));
                                ui.label(format!("{}kg", cargo_type.weight_per_unit * quantity));

                                // Estimate value based on base price
                                let est_value = cargo_type.base_price * quantity;
                                ui.label(format!("~${}", est_value));
                                ui.end_row();
                            }
                        }
                    });
            });
            ui.separator();
        }

        // Quick action buttons
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_rgb(250, 255, 250))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🚀 Quick Actions:");
                    ui.separator();
                    if ui.small_button("📊 Check Prices").clicked() {
                        // This would switch to Market Board in a real implementation
                    }
                    if ui.small_button("⛽ Fuel Status").clicked() {
                        // This would switch to Fuel Pump in a real implementation
                    }
                    if ui.small_button("✈️ Plan Flight").clicked() {
                        // This would switch to Flight Planning in a real implementation
                    }
                });
            });

        ui.separator();

        // Sierra-style tips and information
        ui.collapsing("💡 Pilot's Handbook", |ui| {
            ui.label("📈 Trading Tips:");
            ui.label("  • Buy low at production centers, sell high at consumption markets");
            ui.label("  • Check market trends - volatile goods offer higher profits but more risk");
            ui.label("  • Consider cargo weight vs. fuel efficiency for longer routes");
            ui.separator();

            ui.label("✈️ Flight Operations:");
            ui.label("  • Monitor fuel levels - emergency landings are expensive!");
            ui.label("  • Shorter routes are more fuel-efficient for frequent trading");
            ui.label("  • Each airport specializes in different cargo types");
            ui.separator();

            ui.label("🎯 Strategy:");
            ui.label("  • Start with short, profitable routes to build capital");
            ui.label("  • Expand to longer routes as you gain experience and funds");
            ui.label("  • Goal: Reach $100,000 to become a successful aviation trader!");

            if game_state.cheat_mode {
                ui.separator();
                ui.colored_label(
                    eframe::egui::Color32::from_rgb(255, 140, 0),
                    "⚡ Cheat mode is active - unlimited fuel available!",
                );
            }
        });
    }

    fn render_market_board(game_state: &GameState, ui: &mut eframe::egui::Ui) {
        ui.heading("📊 Market Board - Current Prices");

        if let Some(market) = game_state.get_current_market() {
            // Market board header with timestamp
            eframe::egui::Frame::none()
                .fill(eframe::egui::Color32::from_gray(250))
                .inner_margin(eframe::egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("\"Here are today's market prices, updated hourly.\"");
                        ui.with_layout(
                            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                            |ui| {
                                ui.label(format!("🕒 Turn {}", game_state.turn_number));
                            },
                        );
                    });
                });

            ui.separator();

            // Enhanced market grid with more information
            eframe::egui::Grid::new("market_prices_grid")
                .num_columns(6)
                .spacing([25.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header row with better styling
                    ui.strong("Cargo Type");
                    ui.strong("Current Price");
                    ui.strong("Base Price");
                    ui.strong("Change");
                    ui.strong("Weight");
                    ui.strong("Market Trend");
                    ui.end_row();

                    for (cargo_id, price) in &market.cargo_prices {
                        if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                            // Cargo name with icon
                            let cargo_icon = match cargo_type.name.as_str() {
                                "Electronics" => "💻",
                                "Food & Beverages" => "🍎",
                                "Textiles" => "👔",
                                "Industrial Parts" => "🔧",
                                "Luxury Goods" => "💎",
                                "Raw Materials" => "🏗️",
                                _ => "📦",
                            };
                            ui.label(format!("{} {}", cargo_icon, cargo_type.name));

                            // Current price with color coding
                            let price_color = if *price
                                > cargo_type.base_price + (cargo_type.base_price / 4)
                            {
                                eframe::egui::Color32::from_rgb(220, 50, 50) // Red for high prices
                            } else if *price < cargo_type.base_price - (cargo_type.base_price / 4) {
                                eframe::egui::Color32::from_rgb(50, 150, 50) // Green for low prices
                            } else {
                                eframe::egui::Color32::from_gray(120) // Gray for normal
                            };
                            ui.colored_label(price_color, format!("${}", price));

                            // Base price for reference
                            ui.label(format!("${}", cargo_type.base_price));

                            // Price change percentage
                            let change_percent = (((*price as f32)
                                - (cargo_type.base_price as f32))
                                / (cargo_type.base_price as f32))
                                * 100.0;
                            let change_text = if change_percent > 0.0 {
                                format!("+{:.1}%", change_percent)
                            } else {
                                format!("{:.1}%", change_percent)
                            };
                            let change_color = if change_percent > 0.0 {
                                eframe::egui::Color32::from_rgb(220, 50, 50)
                            } else if change_percent < 0.0 {
                                eframe::egui::Color32::from_rgb(50, 150, 50)
                            } else {
                                eframe::egui::Color32::from_gray(120)
                            };
                            ui.colored_label(change_color, change_text);

                            // Weight per unit
                            ui.label(format!("{}kg", cargo_type.weight_per_unit));

                            // Enhanced trend indicator
                            let (trend_text, trend_color) = if cargo_type.volatility > 0.4 {
                                (
                                    "📈 Very Volatile",
                                    eframe::egui::Color32::from_rgb(255, 140, 0),
                                )
                            } else if cargo_type.volatility > 0.3 {
                                ("📊 Volatile", eframe::egui::Color32::from_rgb(255, 165, 0))
                            } else if *price > cargo_type.base_price {
                                (
                                    "📈 Above Average",
                                    eframe::egui::Color32::from_rgb(220, 50, 50),
                                )
                            } else if *price < cargo_type.base_price {
                                (
                                    "📉 Below Average",
                                    eframe::egui::Color32::from_rgb(50, 150, 50),
                                )
                            } else {
                                ("➖ Stable", eframe::egui::Color32::from_gray(120))
                            };
                            ui.colored_label(trend_color, trend_text);
                            ui.end_row();
                        }
                    }
                });

            ui.separator();

            // Enhanced fuel information
            eframe::egui::Frame::none()
                .fill(eframe::egui::Color32::from_rgb(255, 255, 200))
                .inner_margin(eframe::egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("⛽ Fuel Information:");
                        ui.separator();
                        ui.label(format!("Current: ${}/unit", market.fuel_price));
                        ui.separator();
                        ui.label("💡 Typical range: $60-$120");
                        ui.separator();
                        let fuel_trend = if market.fuel_price > 90 {
                            ("Expensive", eframe::egui::Color32::from_rgb(220, 50, 50))
                        } else if market.fuel_price < 70 {
                            ("Cheap", eframe::egui::Color32::from_rgb(50, 150, 50))
                        } else {
                            ("Normal", eframe::egui::Color32::from_gray(120))
                        };
                        ui.colored_label(fuel_trend.1, fuel_trend.0);
                    });
                });

            ui.separator();

            // Market analysis summary
            ui.collapsing("📈 Market Analysis", |ui| {
                let mut high_prices = Vec::new();
                let mut low_prices = Vec::new();

                for (cargo_id, price) in &market.cargo_prices {
                    if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                        let change_percent = (((*price as f32) - (cargo_type.base_price as f32))
                            / (cargo_type.base_price as f32))
                            * 100.0;
                        if change_percent > 20.0 {
                            high_prices.push((cargo_type.name.clone(), change_percent));
                        } else if change_percent < -20.0 {
                            low_prices.push((cargo_type.name.clone(), change_percent));
                        }
                    }
                }

                if !high_prices.is_empty() {
                    ui.label("🔥 High Prices (Good for Selling):");
                    for (name, change) in &high_prices {
                        ui.label(format!("  • {}: +{:.1}%", name, change));
                    }
                }

                if !low_prices.is_empty() {
                    ui.label("💰 Low Prices (Good for Buying):");
                    for (name, change) in &low_prices {
                        ui.label(format!("  • {}: {:.1}%", name, change));
                    }
                }

                if high_prices.is_empty() && low_prices.is_empty() {
                    ui.label("📊 All prices are within normal ranges today.");
                }
            });
        } else {
            ui.label("❌ Market data not available at this location.");
        }
    }

    fn render_trading_desk(
        game_state: &mut GameState,
        scene_state: &mut SceneState,
        ui: &mut eframe::egui::Ui,
    ) {
        ui.heading("💼 Trading Desk - Buy & Sell Cargo");

        // Trading desk header
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_rgb(240, 248, 255))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.label("\"Looking to do some business? We handle all cargo transactions here.\"");
            });

        ui.separator();

        // Enhanced cargo selection with icons and details
        ui.horizontal(|ui| {
            ui.label("📦 Select Cargo:");
            ui.add_space(10.0);

            eframe::egui::ComboBox::from_id_salt("cargo_selection")
                .width(200.0)
                .selected_text(
                    scene_state
                        .selected_cargo
                        .as_ref()
                        .and_then(|id| game_state.cargo_types.get(id))
                        .map(|ct| {
                            let icon = match ct.name.as_str() {
                                "Electronics" => "💻",
                                "Food & Beverages" => "🍎",
                                "Textiles" => "👔",
                                "Industrial Parts" => "🔧",
                                "Luxury Goods" => "💎",
                                "Raw Materials" => "🏗️",
                                _ => "📦",
                            };
                            format!("{} {}", icon, ct.name)
                        })
                        .unwrap_or("Choose cargo type...".to_string()),
                )
                .show_ui(ui, |ui| {
                    if let Some(market) = game_state.get_current_market() {
                        for cargo_id in market.cargo_prices.keys() {
                            if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                                let icon = match cargo_type.name.as_str() {
                                    "Electronics" => "💻",
                                    "Food & Beverages" => "🍎",
                                    "Textiles" => "👔",
                                    "Industrial Parts" => "🔧",
                                    "Luxury Goods" => "💎",
                                    "Raw Materials" => "🏗️",
                                    _ => "📦",
                                };
                                ui.selectable_value(
                                    &mut scene_state.selected_cargo,
                                    Some(cargo_id.clone()),
                                    format!(
                                        "{} {} ({}kg/unit)",
                                        icon, cargo_type.name, cargo_type.weight_per_unit
                                    ),
                                );
                            }
                        }
                    }
                });
        });

        // Enhanced quantity selection with smart suggestions
        ui.horizontal(|ui| {
            ui.label("📊 Quantity:");
            ui.add_space(10.0);

            // Smart max calculation
            let max_quantity = if let Some(selected_cargo_id) = &scene_state.selected_cargo {
                if let (Some(cargo_type), Some(market)) = (
                    game_state.cargo_types.get(selected_cargo_id),
                    game_state.get_current_market(),
                ) {
                    if let Some(price) = market.cargo_prices.get(selected_cargo_id) {
                        let max_by_money = game_state.player.money / price;
                        let current_weight = game_state
                            .player
                            .current_cargo_weight(&game_state.cargo_types);
                        let available_weight = game_state
                            .player
                            .max_cargo_weight
                            .saturating_sub(current_weight);
                        let max_by_weight = available_weight / cargo_type.weight_per_unit;
                        max_by_money.min(max_by_weight).max(1)
                    } else {
                        20
                    }
                } else {
                    20
                }
            } else {
                20
            };

            let current_quantity = scene_state.trade_quantity;
            ui.add(
                eframe::egui::Slider::new(
                    &mut scene_state.trade_quantity,
                    1..=max_quantity.max(current_quantity),
                )
                .text("units"),
            );

            // Quick quantity buttons
            ui.separator();
            if ui.small_button("1").clicked() {
                scene_state.trade_quantity = 1;
            }
            if ui.small_button("5").clicked() {
                scene_state.trade_quantity = 5.min(max_quantity);
            }
            if ui.small_button("Max").clicked() {
                scene_state.trade_quantity = max_quantity;
            }
        });

        ui.separator();

        // Enhanced transaction details for selected cargo
        if let Some(selected_cargo_id) = &scene_state.selected_cargo {
            let cargo_type = game_state.cargo_types.get(selected_cargo_id).cloned();
            let market = game_state.get_current_market().cloned();

            if let (Some(cargo_type), Some(market)) = (cargo_type, market)
                && let Some(current_price) = market.cargo_prices.get(selected_cargo_id)
            {
                // Transaction details frame
                eframe::egui::Frame::none()
                    .fill(eframe::egui::Color32::from_gray(248))
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::from_gray(200),
                    ))
                    .inner_margin(eframe::egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        let icon = match cargo_type.name.as_str() {
                            "Electronics" => "💻",
                            "Food & Beverages" => "🍎",
                            "Textiles" => "👔",
                            "Industrial Parts" => "🔧",
                            "Luxury Goods" => "💎",
                            "Raw Materials" => "🏗️",
                            _ => "📦",
                        };

                        ui.strong(format!("{} {} Transaction Details", icon, cargo_type.name));
                        ui.separator();

                        eframe::egui::Grid::new("transaction_details")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Current Market Price:");
                                ui.label(format!("${}/unit", current_price));
                                ui.end_row();

                                ui.label("You Currently Own:");
                                let owned = game_state
                                    .player
                                    .cargo_inventory
                                    .get_quantity(selected_cargo_id);
                                ui.label(format!("{} units", owned));
                                ui.end_row();

                                ui.label("Weight per Unit:");
                                ui.label(format!("{}kg", cargo_type.weight_per_unit));
                                ui.end_row();

                                ui.label("Transaction Quantity:");
                                ui.label(format!("{} units", scene_state.trade_quantity));
                                ui.end_row();
                            });
                    });

                ui.add_space(8.0);

                let total_cost = current_price * scene_state.trade_quantity;
                let total_weight = cargo_type.weight_per_unit * scene_state.trade_quantity;

                // Buy transaction
                eframe::egui::Frame::none()
                    .fill(eframe::egui::Color32::from_rgb(240, 255, 240))
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::from_rgb(100, 200, 100),
                    ))
                    .inner_margin(eframe::egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let can_buy = game_state.player.can_afford(total_cost)
                                && game_state
                                    .player
                                    .can_carry_more_weight(total_weight, &game_state.cargo_types);

                            ui.add_enabled_ui(can_buy, |ui| {
                                if ui
                                    .button(format!("💰 BUY {} units", scene_state.trade_quantity))
                                    .clicked()
                                {
                                    match TradingSystem::buy_cargo(
                                        &mut game_state.player,
                                        &market,
                                        &game_state.cargo_types,
                                        selected_cargo_id,
                                        scene_state.trade_quantity,
                                    ) {
                                        Ok(_) => {
                                            game_state.advance_turn();
                                        },
                                        Err(_e) => {
                                            // Could show error dialog
                                        },
                                    }
                                }
                            });

                            ui.separator();

                            if can_buy {
                                ui.label(format!(
                                    "Cost: ${} | Weight: {}kg",
                                    total_cost, total_weight
                                ));
                                ui.separator();
                                ui.label(format!(
                                    "After: ${} remaining",
                                    game_state.player.money.saturating_sub(total_cost)
                                ));
                            } else if !game_state.player.can_afford(total_cost) {
                                ui.colored_label(
                                    eframe::egui::Color32::from_rgb(200, 50, 50),
                                    "💸 Not enough money",
                                );
                            } else {
                                ui.colored_label(
                                    eframe::egui::Color32::from_rgb(200, 50, 50),
                                    "📦 Not enough cargo space",
                                );
                            }
                        });
                    });

                ui.add_space(4.0);

                // Sell transaction
                eframe::egui::Frame::none()
                    .fill(eframe::egui::Color32::from_rgb(255, 248, 240))
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::from_rgb(200, 150, 100),
                    ))
                    .inner_margin(eframe::egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let owned_quantity = game_state
                                .player
                                .cargo_inventory
                                .get_quantity(selected_cargo_id);
                            let sell_quantity = scene_state.trade_quantity.min(owned_quantity);
                            let can_sell = owned_quantity > 0;

                            ui.add_enabled_ui(can_sell, |ui| {
                                if ui
                                    .button(format!("💵 SELL {} units", sell_quantity))
                                    .clicked()
                                {
                                    match TradingSystem::sell_cargo(
                                        &mut game_state.player,
                                        &market,
                                        selected_cargo_id,
                                        sell_quantity,
                                    ) {
                                        Ok(_) => {
                                            game_state.advance_turn();
                                        },
                                        Err(_e) => {
                                            // Could show error dialog
                                        },
                                    }
                                }
                            });

                            ui.separator();

                            if can_sell {
                                let sell_value = current_price * sell_quantity;
                                ui.label(format!(
                                    "Revenue: ${} | Units: {}",
                                    sell_value, sell_quantity
                                ));
                                ui.separator();
                                ui.label(format!(
                                    "After: ${} total",
                                    game_state.player.money + sell_value
                                ));
                            } else {
                                ui.colored_label(
                                    eframe::egui::Color32::from_rgb(200, 50, 50),
                                    "❌ No cargo to sell",
                                );
                            }
                        });
                    });
            }
        } else {
            // No cargo selected
            eframe::egui::Frame::none()
                .fill(eframe::egui::Color32::from_gray(250))
                .inner_margin(eframe::egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("👆 Please select a cargo type above to begin trading");
                        ui.add_space(8.0);
                        ui.label("💡 Tip: Check the Market Board first to see the best deals!");
                    });
                });
        }
    }

    fn render_flight_planning(
        game_state: &mut GameState,
        scene_state: &mut SceneState,
        ui: &mut eframe::egui::Ui,
    ) {
        ui.heading("✈️ Flight Planning - Choose Your Destination");

        // Flight planning header
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_rgb(240, 250, 255))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("\"Where would you like to fly today? I'll calculate fuel requirements for you.\"");
                    if game_state.cheat_mode {
                        ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                            ui.colored_label(eframe::egui::Color32::from_rgb(255, 140, 0), "⚡ CHEAT MODE: Unlimited Fuel");
                        });
                    }
                });
            });

        ui.separator();

        // Current flight status
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_gray(248))
            .stroke(eframe::egui::Stroke::new(
                1.0,
                eframe::egui::Color32::from_gray(200),
            ))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🛩️ Aircraft Status:");
                    ui.separator();
                    ui.label(format!(
                        "Fuel: {}/{} units",
                        game_state.player.fuel, game_state.player.max_fuel
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Range: ~{:.0}km",
                        (game_state.player.fuel as f32) * game_state.player.fuel_efficiency
                    ));
                    ui.separator();
                    let fuel_percent =
                        (game_state.player.fuel as f32 / game_state.player.max_fuel as f32) * 100.0;
                    let fuel_color = if fuel_percent > 75.0 {
                        eframe::egui::Color32::from_rgb(50, 150, 50)
                    } else if fuel_percent > 25.0 {
                        eframe::egui::Color32::from_rgb(255, 140, 0)
                    } else {
                        eframe::egui::Color32::from_rgb(220, 50, 50)
                    };
                    ui.colored_label(fuel_color, format!("Fuel: {:.0}%", fuel_percent));
                });
            });

        ui.separator();

        let destinations: Vec<_> = game_state
            .get_available_destinations()
            .into_iter()
            .cloned()
            .collect();

        // Enhanced destinations grid
        eframe::egui::Grid::new("destinations_grid")
            .num_columns(7)
            .spacing([15.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                // Enhanced header row
                ui.strong("🏢 Airport");
                ui.strong("📏 Distance");
                ui.strong("⛽ Fuel Needed");
                ui.strong("💰 Est. Fuel Cost");
                ui.strong("✈️ Can Fly?");
                ui.strong("📊 Market Info");
                ui.strong("🎯 Action");
                ui.end_row();

                for airport in destinations {
                    if let Some(distance) =
                        game_state.get_distance(&game_state.player.current_airport, &airport.id)
                    {
                        let fuel_needed = game_state.player.fuel_needed_for_distance(distance);
                        let can_travel = game_state.player.can_travel_distance(distance)
                            || game_state.cheat_mode;

                        // Airport name with region indicator
                        let region_icon = match airport.id.as_str() {
                            "JFK" => "🗽",
                            "LAX" => "🌴",
                            "MIA" => "🏖️",
                            "ORD" => "🏙️",
                            "DEN" => "🏔️",
                            "SEA" => "🌲",
                            _ => "🏢",
                        };
                        ui.label(format!("{} {}", region_icon, airport.name));

                        // Distance with color coding
                        let distance_color = if distance > 3000.0 {
                            eframe::egui::Color32::from_rgb(220, 50, 50) // Red for long distance
                        } else if distance > 1500.0 {
                            eframe::egui::Color32::from_rgb(255, 140, 0) // Orange for medium
                        } else {
                            eframe::egui::Color32::from_rgb(50, 150, 50) // Green for short
                        };
                        ui.colored_label(distance_color, format!("{:.0} km", distance));

                        // Fuel needed with efficiency indicator
                        ui.label(format!("{} units", fuel_needed));

                        // Estimated fuel cost (assuming current market price)
                        let fuel_cost = if let Some(market) = game_state.get_current_market() {
                            fuel_needed * market.fuel_price
                        } else {
                            fuel_needed * 80 // Default estimate
                        };
                        ui.label(format!("~${}", fuel_cost));

                        // Can travel status with better feedback
                        if can_travel {
                            if game_state.cheat_mode {
                                ui.colored_label(
                                    eframe::egui::Color32::from_rgb(255, 140, 0),
                                    "⚡ Cheat",
                                );
                            } else {
                                ui.colored_label(
                                    eframe::egui::Color32::from_rgb(50, 150, 50),
                                    "✅ Yes",
                                );
                            }
                        } else {
                            let fuel_deficit = fuel_needed.saturating_sub(game_state.player.fuel);
                            ui.colored_label(
                                eframe::egui::Color32::from_rgb(220, 50, 50),
                                format!("❌ Need +{}", fuel_deficit),
                            );
                        }

                        // Market intelligence preview
                        let market_hint = match airport.id.as_str() {
                            "JFK" => "💻 Tech Hub",
                            "LAX" => "🎬 Entertainment",
                            "MIA" => "🍎 Agriculture",
                            "ORD" => "🔧 Industrial",
                            "DEN" => "🏔️ Regional",
                            "SEA" => "☕ Pacific",
                            _ => "📊 Mixed",
                        };
                        ui.label(market_hint);

                        // Enhanced action button
                        ui.add_enabled_ui(can_travel, |ui| {
                            let button_text = if game_state.cheat_mode {
                                "⚡ Instant Fly"
                            } else {
                                "🛫 Fly"
                            };

                            if ui.button(button_text).clicked() {
                                match TravelSystem::travel_to(game_state, &airport.id) {
                                    Ok(_) => {
                                        scene_state.travel_to_airport(airport.id.clone());
                                    },
                                    Err(_e) => {
                                        // Could show error dialog
                                    },
                                }
                            }
                        });

                        ui.end_row();
                    }
                }
            });

        ui.separator();

        // Flight planning tips
        ui.collapsing("💡 Flight Planning Tips", |ui| {
            ui.label("• Short flights (< 1500km) are more fuel efficient for cargo runs");
            ui.label("• Check market prices at destination before flying");
            ui.label("• Keep emergency fuel reserves for unexpected opportunities");
            ui.label("• Consider fuel costs when calculating trade profits");
            if game_state.cheat_mode {
                ui.colored_label(
                    eframe::egui::Color32::from_rgb(255, 140, 0),
                    "• Cheat mode active: Unlimited fuel available",
                );
            }
        });
    }

    fn render_fuel_pump(
        game_state: &mut GameState,
        scene_state: &mut SceneState,
        ui: &mut eframe::egui::Ui,
    ) {
        ui.heading("⛽ Fuel Pump - Fill Up Your Tank");

        // Fuel pump header
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_rgb(255, 248, 220))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.label("\"Need fuel for your next flight? We've got premium aviation fuel ready to pump!\"");
            });

        ui.separator();

        if let Some(market) = game_state.get_current_market() {
            // Fuel status display
            eframe::egui::Frame::none()
                .fill(eframe::egui::Color32::from_gray(245))
                .stroke(eframe::egui::Stroke::new(
                    1.0,
                    eframe::egui::Color32::from_gray(200),
                ))
                .inner_margin(eframe::egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.strong("🛩️ Aircraft Fuel Status");
                    ui.separator();

                    eframe::egui::Grid::new("fuel_status")
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Current Fuel:");
                            let fuel_percent = (game_state.player.fuel as f32
                                / game_state.player.max_fuel as f32)
                                * 100.0;
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{}/{} units",
                                    game_state.player.fuel, game_state.player.max_fuel
                                ));
                                let fuel_color = if fuel_percent > 75.0 {
                                    eframe::egui::Color32::from_rgb(50, 150, 50)
                                } else if fuel_percent > 25.0 {
                                    eframe::egui::Color32::from_rgb(255, 140, 0)
                                } else {
                                    eframe::egui::Color32::from_rgb(220, 50, 50)
                                };
                                ui.colored_label(fuel_color, format!("({:.0}%)", fuel_percent));
                            });
                            ui.end_row();

                            ui.label("Current Range:");
                            let range =
                                (game_state.player.fuel as f32) * game_state.player.fuel_efficiency;
                            ui.label(format!("~{:.0} km", range));
                            ui.end_row();

                            ui.label("Fuel Price Today:");
                            let price_color = if market.fuel_price > 90 {
                                eframe::egui::Color32::from_rgb(220, 50, 50)
                            } else if market.fuel_price < 70 {
                                eframe::egui::Color32::from_rgb(50, 150, 50)
                            } else {
                                eframe::egui::Color32::from_gray(120)
                            };
                            ui.colored_label(price_color, format!("${}/unit", market.fuel_price));
                            ui.end_row();
                        });
                });

            ui.separator();

            // Enhanced fuel quantity selection
            ui.horizontal(|ui| {
                ui.label("⛽ Fuel Quantity:");
                ui.add_space(10.0);

                let max_fuel_can_add = game_state.player.max_fuel - game_state.player.fuel;
                let max_slider = max_fuel_can_add.max(scene_state.fuel_quantity);

                ui.add(
                    eframe::egui::Slider::new(&mut scene_state.fuel_quantity, 1..=max_slider)
                        .text("units"),
                );

                // Quick fuel buttons
                ui.separator();
                if ui.small_button("10").clicked() {
                    scene_state.fuel_quantity = 10.min(max_fuel_can_add);
                }
                if ui.small_button("25").clicked() {
                    scene_state.fuel_quantity = 25.min(max_fuel_can_add);
                }
                if ui.small_button("50").clicked() {
                    scene_state.fuel_quantity = 50.min(max_fuel_can_add);
                }
                if ui.small_button("Fill").clicked() {
                    scene_state.fuel_quantity = max_fuel_can_add;
                }
            });

            let max_fuel_can_add = game_state.player.max_fuel - game_state.player.fuel;
            let actual_fuel_to_add = scene_state.fuel_quantity.min(max_fuel_can_add);
            let total_cost = market.fuel_price * actual_fuel_to_add;

            ui.separator();

            // Transaction preview
            if actual_fuel_to_add > 0 {
                eframe::egui::Frame::none()
                    .fill(eframe::egui::Color32::from_rgb(240, 255, 240))
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::from_rgb(100, 200, 100),
                    ))
                    .inner_margin(eframe::egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.strong("🧾 Fuel Purchase Preview");
                        ui.separator();

                        eframe::egui::Grid::new("fuel_preview")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Fuel to Add:");
                                ui.label(format!("{} units", actual_fuel_to_add));
                                ui.end_row();

                                ui.label("Total Cost:");
                                ui.label(format!("${}", total_cost));
                                ui.end_row();

                                ui.label("After Purchase:");
                                ui.label(format!(
                                    "{}/{} units ({:.0}%)",
                                    game_state.player.fuel + actual_fuel_to_add,
                                    game_state.player.max_fuel,
                                    ((game_state.player.fuel + actual_fuel_to_add) as f32
                                        / game_state.player.max_fuel as f32)
                                        * 100.0
                                ));
                                ui.end_row();

                                ui.label("New Range:");
                                let new_range = (game_state.player.fuel + actual_fuel_to_add)
                                    as f32
                                    * game_state.player.fuel_efficiency;
                                ui.label(format!("~{:.0} km", new_range));
                                ui.end_row();

                                ui.label("Money After:");
                                ui.label(format!(
                                    "${}",
                                    game_state.player.money.saturating_sub(total_cost)
                                ));
                                ui.end_row();
                            });
                    });

                ui.separator();

                let can_buy = game_state.player.can_afford(total_cost) && actual_fuel_to_add > 0;

                // Purchase button
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(can_buy, |ui| {
                        if ui
                            .button(format!(
                                "⛽ PURCHASE {} units for ${}",
                                actual_fuel_to_add, total_cost
                            ))
                            .clicked()
                            && game_state.player.spend_money(total_cost)
                        {
                            game_state.player.add_fuel(actual_fuel_to_add);
                            game_state.advance_turn();
                        }
                    });

                    ui.separator();

                    if !can_buy {
                        if total_cost > game_state.player.money {
                            ui.colored_label(
                                eframe::egui::Color32::from_rgb(220, 50, 50),
                                "💸 Not enough money",
                            );
                        }
                    } else {
                        ui.label("💡 Tip: Fill up before long flights!");
                    }
                });
            } else {
                // Tank is full
                eframe::egui::Frame::none()
                    .fill(eframe::egui::Color32::from_rgb(255, 255, 240))
                    .inner_margin(eframe::egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("⛽ Tank is Already Full!");
                            ui.add_space(8.0);
                            ui.label("🎉 You're ready for any flight with maximum fuel capacity.");
                        });
                    });
            }
        } else {
            ui.label("❌ Fuel pumps are not operational at this time.");
        }
    }

    fn render_message_board(
        game_state: &mut GameState,
        scene_state: &mut SceneState,
        ui: &mut eframe::egui::Ui,
        api_client: &GameApiClient,
        session: &GameSession,
    ) {
        ui.heading("💬 Message Board - Pilot Communications");

        let current_airport = &game_state.player.current_airport;

        // Display recent messages from API
        eframe::egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                match api_client.get_messages_sync(session.room_id, session.player_id) {
                    Ok(response) => {
                        if response.messages.is_empty() {
                            eframe::egui::Frame::none()
                                .fill(eframe::egui::Color32::from_rgb(250, 250, 250))
                                .inner_margin(eframe::egui::Margin::same(16.0))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label("📭 No messages at this airport yet.");
                                        ui.add_space(8.0);
                                        ui.label(
                                            "Be the first to leave a message for other pilots!",
                                        );
                                    });
                                });
                        } else {
                            ui.label(format!("📋 Recent messages at {}:", current_airport));
                            ui.add_space(4.0);

                            for message in &response.messages {
                                eframe::egui::Frame::none()
                                    .fill(eframe::egui::Color32::from_rgb(245, 245, 250))
                                    .inner_margin(eframe::egui::Margin::same(8.0))
                                    .outer_margin(eframe::egui::Margin::symmetric(0.0, 4.0))
                                    .rounding(eframe::egui::Rounding::same(6.0))
                                    .show(ui, |ui| {
                                        ui.horizontal_top(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    eframe::egui::RichText::new(
                                                        &message.author_name,
                                                    )
                                                    .strong()
                                                    .color(eframe::egui::Color32::from_rgb(
                                                        70, 130, 180,
                                                    )),
                                                );

                                                // Format the timestamp
                                                let local_time = message
                                                    .created_at
                                                    .with_timezone(&chrono::Local);
                                                ui.label(
                                                    eframe::egui::RichText::new(
                                                        local_time.format("%H:%M").to_string(),
                                                    )
                                                    .small()
                                                    .color(eframe::egui::Color32::GRAY),
                                                );
                                            });

                                            ui.separator();
                                            ui.label(&message.content);
                                        });
                                    });
                            }
                        }
                    },
                    Err(err) => {
                        ui.colored_label(
                            eframe::egui::Color32::RED,
                            format!("Error loading messages: {}", err),
                        );
                    },
                }
            });

        ui.add_space(8.0);
        ui.separator();

        // Message composition area
        ui.heading("✍️ Post a Message");

        if !scene_state.show_message_compose {
            if ui.button("📝 Write a message").clicked() {
                scene_state.show_message_compose = true;
                scene_state.message_input.clear();
            }
        } else {
            // Text input for message
            ui.label("Message content (max 500 characters):");
            let text_edit = eframe::egui::TextEdit::multiline(&mut scene_state.message_input)
                .desired_width(f32::INFINITY)
                .desired_rows(3);
            ui.add(text_edit);

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Characters: {}/500",
                    scene_state.message_input.len()
                ));
            });

            ui.add_space(4.0);

            // Post/Cancel buttons
            ui.horizontal(|ui| {
                let can_post = !scene_state.message_input.trim().is_empty()
                    && scene_state.message_input.len() <= 500;

                ui.add_enabled_ui(can_post, |ui| {
                    if ui.button("📤 Post Message").clicked() {
                        // Post message to API
                        match api_client.post_message_sync(
                            session.room_id,
                            session.player_id,
                            scene_state.message_input.clone(),
                        ) {
                            Ok(response) => {
                                if response.success {
                                    scene_state.message_input.clear();
                                    scene_state.show_message_compose = false;
                                } else {
                                    eprintln!("Failed to post message: {}", response.message);
                                    scene_state.message_input.clear();
                                    scene_state.show_message_compose = false;
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to post message: {}", e);
                                scene_state.message_input.clear();
                                scene_state.show_message_compose = false;
                            },
                        }
                    }
                });

                if ui.button("❌ Cancel").clicked() {
                    scene_state.show_message_compose = false;
                    scene_state.message_input.clear();
                }

                if !can_post && !scene_state.message_input.trim().is_empty() {
                    ui.colored_label(
                        eframe::egui::Color32::from_rgb(220, 50, 50),
                        "⚠️ Message too long",
                    );
                }
            });
        }

        ui.add_space(8.0);

        // Instructions
        eframe::egui::Frame::none()
            .fill(eframe::egui::Color32::from_rgb(255, 252, 240))
            .inner_margin(eframe::egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.label(
                    "💡 Messages are location-specific - only pilots at this airport can see them.",
                );
                ui.label("📝 Share tips, warnings, or just say hello to fellow aviators!");
            });
    }
}
