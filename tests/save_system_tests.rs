#[cfg(test)]
mod save_system_tests {
    use kzrk::systems::{GameState, SaveSystem};
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_game() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        unsafe {
            env::set_var("HOME", temp_dir.path());
            env::set_var("CARGO_TARGET_TMPDIR", temp_dir.path());
        }

        // Create a game state
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let original_state = GameState::new(airports, cargo_types);

        // Save the game
        let save_result = SaveSystem::save_game(&original_state, Some("test_save".to_string()));
        assert!(save_result.is_ok());

        // Load the game
        let loaded_state = SaveSystem::load_game("test_save");
        assert!(loaded_state.is_ok());

        let loaded = loaded_state.unwrap();

        // Verify key fields match
        assert_eq!(loaded.player.money, original_state.player.money);
        assert_eq!(
            loaded.player.current_airport,
            original_state.player.current_airport
        );
        assert_eq!(loaded.turn_number, original_state.turn_number);
        assert_eq!(
            loaded.win_condition_money,
            original_state.win_condition_money
        );
    }

    #[test]
    fn test_list_saves() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            env::set_var("HOME", temp_dir.path());
            env::set_var("CARGO_TARGET_TMPDIR", temp_dir.path());
        }

        // Create multiple saves
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let game_state = GameState::new(airports, cargo_types);

        SaveSystem::save_game(&game_state, Some("save1".to_string())).unwrap();
        SaveSystem::save_game(&game_state, Some("save2".to_string())).unwrap();

        // List saves
        let saves = SaveSystem::list_saves();
        assert!(saves.is_ok());

        let save_list = saves.unwrap();
        assert_eq!(save_list.len(), 2);
    }

    #[test]
    fn test_autosave() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            env::set_var("HOME", temp_dir.path());
            env::set_var("CARGO_TARGET_TMPDIR", temp_dir.path());
        }

        // Create a game state
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let game_state = GameState::new(airports, cargo_types);

        // Create autosave
        assert!(!SaveSystem::has_autosave());
        let autosave_result = SaveSystem::autosave(&game_state);
        assert!(autosave_result.is_ok());
        assert!(SaveSystem::has_autosave());

        // Load autosave
        let loaded = SaveSystem::load_autosave();
        assert!(loaded.is_ok());
    }

    #[test]
    fn test_save_with_game_progress() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            env::set_var("HOME", temp_dir.path());
            env::set_var("CARGO_TARGET_TMPDIR", temp_dir.path());
        }

        // Create a game state with some progress
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let mut game_state = GameState::new(airports, cargo_types);

        // Modify the game state
        game_state.player.money = 15000;
        game_state.player.current_airport = "LAX".to_string();
        game_state.turn_number = 10;
        game_state.player.fuel = 50;

        // Add some cargo
        game_state
            .player
            .cargo_inventory
            .add_cargo("electronics", 5);

        // Save
        SaveSystem::save_game(&game_state, Some("progress_save".to_string())).unwrap();

        // Load and verify
        let loaded = SaveSystem::load_game("progress_save").unwrap();
        assert_eq!(loaded.player.money, 15000);
        assert_eq!(loaded.player.current_airport, "LAX");
        assert_eq!(loaded.turn_number, 10);
        assert_eq!(loaded.player.fuel, 50);
        assert_eq!(loaded.player.cargo_inventory.get_quantity("electronics"), 5);
    }

    #[test]
    fn test_load_nonexistent_save() {
        let temp_dir = tempdir().unwrap();
        unsafe {
            env::set_var("HOME", temp_dir.path());
            env::set_var("CARGO_TARGET_TMPDIR", temp_dir.path());
        }

        let result = SaveSystem::load_game("nonexistent");
        assert!(result.is_err());
    }
}
