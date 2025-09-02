#[cfg(test)]
mod save_system_tests {
    use kzrk::systems::{GameState, SaveSystem};
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_game() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let save_dir = temp_dir.path().join("saves");

        // Create a game state
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let original_state = GameState::new(airports, cargo_types);

        // Save the game to specific directory
        let save_result =
            SaveSystem::save_game_to_dir(&original_state, Some("test_save".to_string()), &save_dir);
        assert!(save_result.is_ok());

        // Load the game from specific directory
        let loaded_state = SaveSystem::load_game_from_dir("test_save", &save_dir);
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
        let save_dir = temp_dir.path().join("saves");

        // Create multiple saves
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let game_state = GameState::new(airports, cargo_types);

        SaveSystem::save_game_to_dir(&game_state, Some("save1".to_string()), &save_dir).unwrap();
        SaveSystem::save_game_to_dir(&game_state, Some("save2".to_string()), &save_dir).unwrap();

        // List saves from specific directory
        let saves = SaveSystem::list_saves_in_dir(&save_dir);
        assert!(saves.is_ok());

        let save_list = saves.unwrap();
        assert_eq!(save_list.len(), 2);
    }

    #[test]
    fn test_autosave() {
        let temp_dir = tempdir().unwrap();
        let save_dir = temp_dir.path().join("saves");

        // Create a game state
        let airports = kzrk::data::get_default_airports();
        let cargo_types = kzrk::data::get_default_cargo_types();
        let game_state = GameState::new(airports, cargo_types);

        // Create autosave in specific directory
        let autosave_result =
            SaveSystem::save_game_to_dir(&game_state, Some("autosave".to_string()), &save_dir);
        assert!(autosave_result.is_ok());

        // Load autosave from specific directory
        let loaded = SaveSystem::load_game_from_dir("autosave", &save_dir);
        assert!(loaded.is_ok());
    }

    #[test]
    fn test_save_with_game_progress() {
        let temp_dir = tempdir().unwrap();
        let save_dir = temp_dir.path().join("saves");

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

        // Save to specific directory
        SaveSystem::save_game_to_dir(&game_state, Some("progress_save".to_string()), &save_dir)
            .unwrap();

        // Load and verify from specific directory
        let loaded = SaveSystem::load_game_from_dir("progress_save", &save_dir).unwrap();
        assert_eq!(loaded.player.money, 15000);
        assert_eq!(loaded.player.current_airport, "LAX");
        assert_eq!(loaded.turn_number, 10);
        assert_eq!(loaded.player.fuel, 50);
        assert_eq!(loaded.player.cargo_inventory.get_quantity("electronics"), 5);
    }

    #[test]
    fn test_load_nonexistent_save() {
        let temp_dir = tempdir().unwrap();
        let save_dir = temp_dir.path().join("saves");

        let result = SaveSystem::load_game_from_dir("nonexistent", &save_dir);
        assert!(result.is_err());
    }
}
