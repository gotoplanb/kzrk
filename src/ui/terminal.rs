use crate::systems::{GameState, TradingSystem, TravelSystem};
use std::io::{self, Write};

pub struct TerminalUI;

impl TerminalUI {
    pub fn run_game_loop() {
        println!("=== KZRK Aviation Trading Game ===");
        println!("Welcome, pilot! Build your aviation trading empire.");
        
        // Initialize game
        let airports = crate::data::get_default_airports();
        let cargo_types = crate::data::get_default_cargo_types();
        let mut game_state = GameState::new(airports, cargo_types);
        
        // Show cheat mode status if enabled
        if game_state.cheat_mode {
            println!("🔧 CHEAT MODE ENABLED: Unlimited fuel for travel!");
            println!("   (Set via KZRK_CHEAT environment variable)");
        }
        println!();

        // Main game loop
        loop {
            // Check win/lose conditions
            if game_state.is_game_won() {
                Self::display_victory(&game_state);
                break;
            }

            if !game_state.can_player_continue() {
                Self::display_game_over(&game_state);
                break;
            }

            // Display current status
            Self::display_status(&game_state);

            // Main menu
            match Self::display_main_menu() {
                MainMenuChoice::ViewMarket => {
                    Self::display_market_info(&game_state);
                },
                MainMenuChoice::Trade => {
                    Self::handle_trading(&mut game_state);
                },
                MainMenuChoice::Travel => {
                    Self::handle_travel(&mut game_state);
                },
                MainMenuChoice::Help => {
                    Self::display_help();
                },
                MainMenuChoice::Quit => {
                    println!("Thanks for playing KZRK! Safe travels, pilot.");
                    break;
                },
            }

            println!(); // Add spacing between turns
        }
    }

    fn display_status(game_state: &GameState) {
        println!("=== STATUS ===");
        
        if let Some(current_airport) = game_state.get_current_airport() {
            println!("Location: {} ({})", current_airport.name, current_airport.id);
        }
        
        println!("Turn: {}", game_state.turn_number);
        println!("Money: ${}", game_state.player.money);
        println!("Fuel: {}/{}", game_state.player.fuel, game_state.player.max_fuel);
        
        let current_weight = game_state.player.current_cargo_weight(&game_state.cargo_types);
        println!("Cargo: {}kg / {}kg", current_weight, game_state.player.max_cargo_weight);
        
        // Show carried cargo
        if current_weight > 0 {
            println!("Carrying:");
            for (cargo_id, quantity) in game_state.player.cargo_inventory.get_all_cargo() {
                if *quantity > 0 {
                    if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                        println!("  {} x{}", cargo_type.name, quantity);
                    }
                }
            }
        }
        
        println!();
    }

    fn display_main_menu() -> MainMenuChoice {
        loop {
            println!("=== MAIN MENU ===");
            println!("1. View Market");
            println!("2. Trade");
            println!("3. Travel");
            println!("4. Help");
            println!("5. Quit");
            print!("Choose an option (1-5): ");
            io::stdout().flush().unwrap();

            let choice = Self::get_user_input();
            match choice.trim() {
                "1" => return MainMenuChoice::ViewMarket,
                "2" => return MainMenuChoice::Trade,
                "3" => return MainMenuChoice::Travel,
                "4" => return MainMenuChoice::Help,
                "5" => return MainMenuChoice::Quit,
                _ => {
                    println!("Invalid choice. Please try again.");
                    println!();
                }
            }
        }
    }

    fn display_market_info(game_state: &GameState) {
        println!("=== MARKET PRICES ===");
        
        if let Some(market) = game_state.get_current_market() {
            println!("Fuel: ${}/unit", market.fuel_price);
            println!();
            println!("Cargo Prices:");
            
            let mut cargo_list: Vec<_> = market.get_all_cargo_prices().iter().collect();
            cargo_list.sort_by(|a, b| a.0.cmp(b.0)); // Sort by cargo ID
            
            for (cargo_id, price) in cargo_list {
                if let Some(cargo_type) = game_state.cargo_types.get(cargo_id) {
                    let max_buyable = TradingSystem::get_max_buyable_quantity(
                        &game_state.player,
                        market,
                        &game_state.cargo_types,
                        cargo_id,
                    );
                    
                    println!("  {}: ${}/unit (can buy: {})", 
                        cargo_type.name, price, max_buyable);
                }
            }
        }
        
        Self::press_enter_to_continue();
    }

    fn handle_trading(game_state: &mut GameState) {
        loop {
            println!("=== TRADING ===");
            println!("1. Buy Cargo");
            println!("2. Sell Cargo");
            println!("3. Buy Fuel");
            println!("4. Back to Main Menu");
            print!("Choose an option (1-4): ");
            io::stdout().flush().unwrap();

            let choice = Self::get_user_input();
            match choice.trim() {
                "1" => Self::handle_buy_cargo(game_state),
                "2" => Self::handle_sell_cargo(game_state),
                "3" => Self::handle_buy_fuel(game_state),
                "4" => break,
                _ => println!("Invalid choice. Please try again."),
            }
        }
    }

    fn handle_buy_cargo(game_state: &mut GameState) {
        if let Some(market) = game_state.get_current_market().cloned() {
            println!("=== BUY CARGO ===");
            
            let mut available_cargo: Vec<_> = market.get_all_cargo_prices().iter().collect();
            available_cargo.sort_by(|a, b| a.0.cmp(b.0));
            
            for (i, (cargo_id, price)) in available_cargo.iter().enumerate() {
                if let Some(cargo_type) = game_state.cargo_types.get(*cargo_id) {
                    let max_buyable = TradingSystem::get_max_buyable_quantity(
                        &game_state.player,
                        &market,
                        &game_state.cargo_types,
                        cargo_id,
                    );
                    
                    println!("{}. {} - ${}/unit (max: {})", 
                        i + 1, cargo_type.name, price, max_buyable);
                }
            }
            
            println!("0. Back");
            print!("Choose cargo to buy (0-{}): ", available_cargo.len());
            io::stdout().flush().unwrap();

            let choice = Self::get_user_input();
            if let Ok(index) = choice.trim().parse::<usize>() {
                if index == 0 {
                    return;
                }
                
                if let Some((cargo_id, _)) = available_cargo.get(index - 1) {
                    let max_buyable = TradingSystem::get_max_buyable_quantity(
                        &game_state.player,
                        &market,
                        &game_state.cargo_types,
                        cargo_id,
                    );
                    
                    if max_buyable == 0 {
                        println!("You cannot afford any of this cargo or have no cargo space.");
                        Self::press_enter_to_continue();
                        return;
                    }
                    
                    print!("Enter quantity to buy (max {}): ", max_buyable);
                    io::stdout().flush().unwrap();
                    
                    let quantity_input = Self::get_user_input();
                    if let Ok(quantity) = quantity_input.trim().parse::<u32>() {
                        if quantity > 0 && quantity <= max_buyable {
                            match TradingSystem::buy_cargo(
                                &mut game_state.player,
                                &market,
                                &game_state.cargo_types,
                                cargo_id,
                                quantity,
                            ) {
                                Ok(cost) => {
                                    if let Some(cargo_type) = game_state.cargo_types.get(*cargo_id) {
                                        println!("✓ Bought {} {} for ${}", 
                                            quantity, cargo_type.name, cost);
                                    }
                                },
                                Err(e) => println!("✗ Purchase failed: {:?}", e),
                            }
                        } else {
                            println!("Invalid quantity.");
                        }
                    } else {
                        println!("Invalid input.");
                    }
                }
            }
            
            Self::press_enter_to_continue();
        }
    }

    fn handle_sell_cargo(game_state: &mut GameState) {
        println!("=== SELL CARGO ===");
        
        let carried_cargo = game_state.player.cargo_inventory.get_all_cargo().clone();
        if carried_cargo.is_empty() || carried_cargo.values().all(|&q| q == 0) {
            println!("You have no cargo to sell.");
            Self::press_enter_to_continue();
            return;
        }
        
        let mut sellable_cargo: Vec<_> = carried_cargo.iter()
            .filter(|(_, quantity)| **quantity > 0)
            .collect();
        sellable_cargo.sort_by(|a, b| a.0.cmp(b.0));
        
        let market = match game_state.get_current_market().cloned() {
            Some(m) => m,
            None => {
                println!("No market available.");
                Self::press_enter_to_continue();
                return;
            }
        };
        
        for (i, (cargo_id, quantity)) in sellable_cargo.iter().enumerate() {
            if let Some(cargo_type) = game_state.cargo_types.get(*cargo_id) {
                if let Some(price) = market.get_cargo_price(cargo_id) {
                    println!("{}. {} x{} - ${}/unit (total: ${})", 
                        i + 1, cargo_type.name, quantity, price, price * *quantity);
                }
            }
        }
        
        println!("0. Back");
        print!("Choose cargo to sell (0-{}): ", sellable_cargo.len());
        io::stdout().flush().unwrap();

        let choice = Self::get_user_input();
        if let Ok(index) = choice.trim().parse::<usize>() {
            if index == 0 {
                return;
            }
            
            if let Some((cargo_id, max_quantity)) = sellable_cargo.get(index - 1) {
                print!("Enter quantity to sell (max {}): ", max_quantity);
                io::stdout().flush().unwrap();
                
                let quantity_input = Self::get_user_input();
                if let Ok(quantity) = quantity_input.trim().parse::<u32>() {
                    if quantity > 0 && quantity <= **max_quantity {
                        match TradingSystem::sell_cargo(
                            &mut game_state.player,
                            &market,
                            cargo_id,
                            quantity,
                        ) {
                            Ok(revenue) => {
                                if let Some(cargo_type) = game_state.cargo_types.get(*cargo_id) {
                                    println!("✓ Sold {} {} for ${}", 
                                        quantity, cargo_type.name, revenue);
                                }
                            },
                            Err(e) => println!("✗ Sale failed: {:?}", e),
                        }
                    } else {
                        println!("Invalid quantity.");
                    }
                } else {
                    println!("Invalid input.");
                }
            }
        }
        
        Self::press_enter_to_continue();
    }

    fn handle_buy_fuel(game_state: &mut GameState) {
        if let Some(market) = game_state.get_current_market().cloned() {
            println!("=== BUY FUEL ===");
            
            let max_fuel = TradingSystem::get_max_fuel_buyable(&game_state.player, &market);
            println!("Fuel price: ${}/unit", market.fuel_price);
            println!("Current fuel: {}/{}", game_state.player.fuel, game_state.player.max_fuel);
            println!("Max you can buy: {}", max_fuel);
            
            if max_fuel == 0 {
                println!("Your tank is full or you cannot afford fuel.");
                Self::press_enter_to_continue();
                return;
            }
            
            print!("Enter fuel quantity to buy (0 to cancel): ");
            io::stdout().flush().unwrap();
            
            let input = Self::get_user_input();
            if let Ok(quantity) = input.trim().parse::<u32>() {
                if quantity == 0 {
                    return;
                }
                
                if quantity <= max_fuel {
                    match TradingSystem::buy_fuel(&mut game_state.player, &market, quantity) {
                        Ok(cost) => {
                            println!("✓ Bought {} fuel for ${}", quantity, cost);
                        },
                        Err(e) => println!("✗ Fuel purchase failed: {:?}", e),
                    }
                } else {
                    println!("Cannot buy that much fuel.");
                }
            } else {
                println!("Invalid input.");
            }
            
            Self::press_enter_to_continue();
        }
    }

    fn handle_travel(game_state: &mut GameState) {
        println!("=== TRAVEL ===");
        
        let destinations = TravelSystem::get_reachable_destinations(game_state);
        
        if destinations.is_empty() {
            println!("No destinations available.");
            Self::press_enter_to_continue();
            return;
        }
        
        println!("Available destinations:");
        for (i, dest) in destinations.iter().enumerate() {
            let status = if dest.can_afford { "✓" } else { "✗" };
            let cheat_indicator = if game_state.cheat_mode && !game_state.player.fuel >= dest.fuel_needed {
                " 🔧"
            } else {
                ""
            };
            println!("{}. {} {} - {:.0}km, {} fuel needed ({}){}", 
                i + 1, status, dest.airport_name, dest.distance_km, dest.fuel_needed, dest.airport_id, cheat_indicator);
        }
        
        println!("0. Back");
        print!("Choose destination (0-{}): ", destinations.len());
        io::stdout().flush().unwrap();

        let choice = Self::get_user_input();
        if let Ok(index) = choice.trim().parse::<usize>() {
            if index == 0 {
                return;
            }
            
            if let Some(destination) = destinations.get(index - 1) {
                if !destination.can_afford {
                    println!("You don't have enough fuel for this trip.");
                    println!("You need {} fuel but only have {}.", 
                        destination.fuel_needed, game_state.player.fuel);
                    Self::press_enter_to_continue();
                    return;
                }
                
                print!("Confirm travel to {} (y/n): ", destination.airport_name);
                io::stdout().flush().unwrap();
                
                let confirm = Self::get_user_input();
                if confirm.trim().to_lowercase() == "y" {
                    match TravelSystem::travel_to(game_state, &destination.airport_id) {
                        Ok(travel_info) => {
                            println!("✓ Travel successful!");
                            println!("Route: {} → {}", travel_info.from, travel_info.to);
                            println!("Distance: {:.0}km, Fuel consumed: {}", 
                                travel_info.distance_km, travel_info.fuel_consumed);
                            println!("Arrived at {}! New market prices await.", travel_info.to);
                        },
                        Err(e) => println!("✗ Travel failed: {:?}", e),
                    }
                }
            }
        }
        
        Self::press_enter_to_continue();
    }

    fn display_help() {
        println!("=== HELP ===");
        println!("KZRK is an aviation trading game. Your goal is to reach $100,000.");
        println!();
        println!("Game Mechanics:");
        println!("• Buy cargo cheap at one airport, sell expensive at another");
        println!("• Different airports produce/consume different goods");
        println!("• Fuel is needed for travel - manage it carefully");
        println!("• Market prices change when you travel");
        println!("• Your plane has limited cargo capacity (weight-based)");
        println!();
        println!("Tips:");
        println!("• Look for airports that produce goods (lower prices)");
        println!("• Sell at airports that consume goods (higher prices)");
        println!("• Electronics and luxury goods are valuable but volatile");
        println!("• Industrial goods and materials are stable but lower profit");
        println!("• Plan your routes to minimize fuel costs");
        println!();
        
        Self::press_enter_to_continue();
    }

    fn display_victory(game_state: &GameState) {
        println!("🎉 CONGRATULATIONS! 🎉");
        println!("You've reached $100,000 and won the game!");
        println!("Final stats:");
        println!("  Money: ${}", game_state.player.money);
        println!("  Turns: {}", game_state.turn_number);
        println!("  Final location: {}", game_state.player.current_airport);
        println!();
        println!("You are now a successful aviation trading mogul!");
        println!("Thanks for playing KZRK!");
    }

    fn display_game_over(game_state: &GameState) {
        println!("💸 GAME OVER 💸");
        println!("You've run out of money and fuel. Your trading career has ended.");
        println!("Final stats:");
        println!("  Money: ${}", game_state.player.money);
        println!("  Fuel: {}/{}", game_state.player.fuel, game_state.player.max_fuel);
        println!("  Turns survived: {}", game_state.turn_number);
        println!();
        println!("Better luck next time, pilot!");
    }

    fn get_user_input() -> String {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => input.trim().to_string(),
            Err(_) => "".to_string(),
        }
    }

    fn press_enter_to_continue() {
        print!("Press Enter to continue...");
        io::stdout().flush().unwrap();
        let _ = Self::get_user_input();
    }
}

#[derive(Debug)]
enum MainMenuChoice {
    ViewMarket,
    Trade,
    Travel,
    Help,
    Quit,
}