pub mod terminal;

#[cfg(feature = "gui")]
pub mod egui_app;

#[cfg(feature = "gui")]
pub mod game_api_client;

#[cfg(feature = "gui")]
pub mod scenes;

pub use terminal::TerminalUI;
