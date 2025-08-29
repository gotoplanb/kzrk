mod data;
mod models;
mod systems;
mod ui;

use ui::TerminalUI;

fn main() {
    TerminalUI::run_game_loop();
}