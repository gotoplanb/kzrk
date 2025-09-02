use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::systems::GameState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGame {
    pub game_state: GameState,
    pub save_name: String,
    pub timestamp: DateTime<Local>,
    pub version: String,
}

#[derive(Debug)]
pub enum SaveError {
    IoError(String),
    SerializationError(String),
    #[allow(dead_code)]
    InvalidSaveFile,
    SaveNotFound,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::IoError(e) => write!(f, "IO error: {}", e),
            SaveError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            SaveError::InvalidSaveFile => write!(f, "Invalid save file format"),
            SaveError::SaveNotFound => write!(f, "Save file not found"),
        }
    }
}

pub struct SaveSystem;

impl SaveSystem {
    /// Get the default save directory path
    pub fn get_save_directory() -> Result<PathBuf, SaveError> {
        // Check if we're in a test environment
        if cfg!(test)
            && let Ok(test_dir) = std::env::var("CARGO_TARGET_TMPDIR")
        {
            let save_dir = PathBuf::from(test_dir).join("kzrk_saves");
            fs::create_dir_all(&save_dir).map_err(|e| {
                SaveError::IoError(format!("Failed to create test save directory: {}", e))
            })?;
            return Ok(save_dir);
        }

        let save_dir = if cfg!(target_os = "windows") {
            dirs::data_dir()
                .map(|d| d.join("KZRK").join("saves"))
                .ok_or_else(|| {
                    SaveError::IoError("Could not determine data directory".to_string())
                })?
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|d| {
                    d.join("Library")
                        .join("Application Support")
                        .join("KZRK")
                        .join("saves")
                })
                .ok_or_else(|| {
                    SaveError::IoError("Could not determine home directory".to_string())
                })?
        } else {
            // Linux and other Unix-like systems
            dirs::config_dir()
                .map(|d| d.join("kzrk").join("saves"))
                .ok_or_else(|| {
                    SaveError::IoError("Could not determine config directory".to_string())
                })?
        };

        // Create directory if it doesn't exist
        fs::create_dir_all(&save_dir)
            .map_err(|e| SaveError::IoError(format!("Failed to create save directory: {}", e)))?;

        Ok(save_dir)
    }

    /// Save the game state to a file
    pub fn save_game(
        game_state: &GameState,
        save_name: Option<String>,
    ) -> Result<PathBuf, SaveError> {
        let save_dir = Self::get_save_directory()?;

        // Generate save name if not provided
        let save_name =
            save_name.unwrap_or_else(|| format!("save_{}", Local::now().format("%Y%m%d_%H%M%S")));

        let save_file = SaveGame {
            game_state: game_state.clone(),
            save_name: save_name.clone(),
            timestamp: Local::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let file_path = save_dir.join(format!("{}.json", save_name));

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&save_file)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        // Write to file
        fs::write(&file_path, json)
            .map_err(|e| SaveError::IoError(format!("Failed to write save file: {}", e)))?;

        Ok(file_path)
    }

    /// Load a game state from a file
    pub fn load_game(save_name: &str) -> Result<GameState, SaveError> {
        let save_dir = Self::get_save_directory()?;
        let file_path = save_dir.join(format!("{}.json", save_name));

        if !file_path.exists() {
            return Err(SaveError::SaveNotFound);
        }

        // Read file
        let json = fs::read_to_string(&file_path)
            .map_err(|e| SaveError::IoError(format!("Failed to read save file: {}", e)))?;

        // Deserialize from JSON
        let save_file: SaveGame = serde_json::from_str(&json)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        Ok(save_file.game_state)
    }

    /// Load a game from a specific path
    #[allow(dead_code)]
    pub fn load_game_from_path(path: &Path) -> Result<GameState, SaveError> {
        if !path.exists() {
            return Err(SaveError::SaveNotFound);
        }

        // Read file
        let json = fs::read_to_string(path)
            .map_err(|e| SaveError::IoError(format!("Failed to read save file: {}", e)))?;

        // Deserialize from JSON
        let save_file: SaveGame = serde_json::from_str(&json)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        Ok(save_file.game_state)
    }

    /// List all available save files
    pub fn list_saves() -> Result<Vec<SaveInfo>, SaveError> {
        let save_dir = Self::get_save_directory()?;
        let mut saves = Vec::new();

        let entries = fs::read_dir(&save_dir)
            .map_err(|e| SaveError::IoError(format!("Failed to read save directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Try to read save info
                if let Ok(json) = fs::read_to_string(&path)
                    && let Ok(save_file) = serde_json::from_str::<SaveGame>(&json)
                {
                    saves.push(SaveInfo {
                        name: save_file.save_name,
                        timestamp: save_file.timestamp,
                        turn: save_file.game_state.turn_number,
                        money: save_file.game_state.player.money,
                        location: save_file.game_state.player.current_airport.clone(),
                        file_name: path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    });
                }
            }
        }

        // Sort by timestamp, newest first
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    /// Delete a save file
    #[allow(dead_code)]
    pub fn delete_save(save_name: &str) -> Result<(), SaveError> {
        let save_dir = Self::get_save_directory()?;
        let file_path = save_dir.join(format!("{}.json", save_name));

        if !file_path.exists() {
            return Err(SaveError::SaveNotFound);
        }

        fs::remove_file(&file_path)
            .map_err(|e| SaveError::IoError(format!("Failed to delete save file: {}", e)))?;

        Ok(())
    }

    /// Create an autosave
    pub fn autosave(game_state: &GameState) -> Result<PathBuf, SaveError> {
        Self::save_game(game_state, Some("autosave".to_string()))
    }

    /// Check if an autosave exists
    pub fn has_autosave() -> bool {
        if let Ok(save_dir) = Self::get_save_directory() {
            save_dir.join("autosave.json").exists()
        } else {
            false
        }
    }

    /// Load the autosave
    pub fn load_autosave() -> Result<GameState, SaveError> {
        Self::load_game("autosave")
    }

    // Test-specific methods that accept custom directories
    #[allow(dead_code)]
    pub fn save_game_to_dir(
        game_state: &GameState,
        save_name: Option<String>,
        save_dir: &Path,
    ) -> Result<PathBuf, SaveError> {
        // Create directory if it doesn't exist
        fs::create_dir_all(save_dir)
            .map_err(|e| SaveError::IoError(format!("Failed to create save directory: {}", e)))?;

        // Generate save name if not provided
        let save_name =
            save_name.unwrap_or_else(|| format!("save_{}", Local::now().format("%Y%m%d_%H%M%S")));

        let save_file = SaveGame {
            game_state: game_state.clone(),
            save_name: save_name.clone(),
            timestamp: Local::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let file_path = save_dir.join(format!("{}.json", save_name));

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&save_file)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        // Write to file
        fs::write(&file_path, json)
            .map_err(|e| SaveError::IoError(format!("Failed to write save file: {}", e)))?;

        Ok(file_path)
    }

    #[allow(dead_code)]
    pub fn load_game_from_dir(save_name: &str, save_dir: &Path) -> Result<GameState, SaveError> {
        let file_path = save_dir.join(format!("{}.json", save_name));

        if !file_path.exists() {
            return Err(SaveError::SaveNotFound);
        }

        // Read file
        let json = fs::read_to_string(&file_path)
            .map_err(|e| SaveError::IoError(format!("Failed to read save file: {}", e)))?;

        // Deserialize from JSON
        let save_file: SaveGame = serde_json::from_str(&json)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        Ok(save_file.game_state)
    }

    #[allow(dead_code)]
    pub fn list_saves_in_dir(save_dir: &Path) -> Result<Vec<SaveInfo>, SaveError> {
        let mut saves = Vec::new();

        let entries = fs::read_dir(save_dir)
            .map_err(|e| SaveError::IoError(format!("Failed to read save directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Try to read save info
                if let Ok(json) = fs::read_to_string(&path)
                    && let Ok(save_file) = serde_json::from_str::<SaveGame>(&json)
                {
                    saves.push(SaveInfo {
                        name: save_file.save_name,
                        timestamp: save_file.timestamp,
                        turn: save_file.game_state.turn_number,
                        money: save_file.game_state.player.money,
                        location: save_file.game_state.player.current_airport.clone(),
                        file_name: path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    });
                }
            }
        }

        // Sort by timestamp, newest first
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInfo {
    pub name: String,
    pub timestamp: DateTime<Local>,
    pub turn: u32,
    pub money: u32,
    pub location: String,
    pub file_name: String,
}
