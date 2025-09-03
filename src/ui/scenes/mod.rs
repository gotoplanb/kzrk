pub mod airport;
pub mod room_lobby;
pub mod server_connection;

#[derive(Debug, Clone, PartialEq)]
pub enum Scene {
    ServerConnection,
    RoomLobby,
    Airport(String), // airport_id
}

impl Default for Scene {
    fn default() -> Self {
        Self::ServerConnection
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Location {
    MainDesk,       // General info, fuel status, overview
    MarketBoard,    // View current prices
    TradingDesk,    // Buy/sell cargo
    FlightPlanning, // Travel to other airports
    FuelPump,       // Buy fuel
    MessageBoard,   // Read and post messages
                    // Future locations:
                    // Hangar,       // Plane upgrades
                    // WeatherStation, // Weather info
                    // RadioRoom,    // Communications
}

impl Default for Location {
    fn default() -> Self {
        Self::MainDesk
    }
}

#[derive(Debug, Clone, Default)]
pub struct SceneState {
    pub current_scene: Scene,
    pub current_location: Location,

    // UI-only state for trading
    pub selected_cargo: Option<String>,
    pub trade_quantity: u32,

    // UI-only state for travel
    pub selected_destination: Option<String>,

    // UI state for fuel purchase
    pub fuel_quantity: u32,

    // UI state for message board
    pub message_input: String,
    pub show_message_compose: bool,
}

impl SceneState {
    pub fn new() -> Self {
        Self {
            current_scene: Scene::ServerConnection, // Start at server connection
            current_location: Location::MainDesk,
            selected_cargo: None,
            trade_quantity: 1,
            selected_destination: None,
            fuel_quantity: 10,
            message_input: String::new(),
            show_message_compose: false,
        }
    }

    pub fn go_to_location(&mut self, location: Location) {
        self.current_location = location;
        // Reset UI state when changing locations
        self.selected_cargo = None;
        self.selected_destination = None;
        self.message_input.clear();
        self.show_message_compose = false;
    }

    pub fn travel_to_airport(&mut self, airport_id: String) {
        self.current_scene = Scene::Airport(airport_id);
        self.current_location = Location::MainDesk; // Always arrive at main desk
        // Reset UI state
        self.selected_cargo = None;
        self.selected_destination = None;
    }
}
