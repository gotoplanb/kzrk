use crate::{
    data::{get_default_airports, get_default_cargo_types},
    systems::game::GameState,
    ui::scenes::{Scene, SceneState, airport::AirportScene},
};

pub struct KzrkEguiApp {
    game_state: GameState,
    scene_state: SceneState,
}

impl Default for KzrkEguiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl KzrkEguiApp {
    pub fn new() -> Self {
        let airports = get_default_airports();
        let cargo_types = get_default_cargo_types();

        Self {
            game_state: GameState::new(airports, cargo_types),
            scene_state: SceneState::new(),
        }
    }
}

impl eframe::App for KzrkEguiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Make sure scene state matches game state
        if let Scene::Airport(ref airport_id) = self.scene_state.current_scene
            && airport_id != &self.game_state.player.current_airport
        {
            self.scene_state
                .travel_to_airport(self.game_state.player.current_airport.clone());
        }

        // Render current scene
        match &self.scene_state.current_scene {
            Scene::MainMenu => {
                // For now, just redirect to airport
                self.scene_state
                    .travel_to_airport(self.game_state.player.current_airport.clone());
            },
            Scene::Airport(_) => {
                AirportScene::render(&mut self.game_state, &mut self.scene_state, ctx);
            },
        }
    }
}
