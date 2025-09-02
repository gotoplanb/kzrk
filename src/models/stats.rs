use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub total_trades: u32,
    pub successful_trades: u32,
    pub total_profit: i64,
    pub total_loss: i64,
    pub best_trade_profit: i64,
    pub worst_trade_loss: i64,
    pub total_distance_traveled: f64,
    pub total_fuel_consumed: u32,
    pub total_fuel_purchased: u32,
    pub total_cargo_bought: u32,
    pub total_cargo_sold: u32,
    pub airports_visited: Vec<String>,
    pub favorite_cargo: Option<String>,
    pub most_profitable_route: Option<(String, String)>,
    pub peak_money: u32,
    pub lowest_money: u32,
    pub times_went_broke: u32,
}

impl GameStats {
    pub fn new(starting_money: u32) -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            total_profit: 0,
            total_loss: 0,
            best_trade_profit: 0,
            worst_trade_loss: 0,
            total_distance_traveled: 0.0,
            total_fuel_consumed: 0,
            total_fuel_purchased: 0,
            total_cargo_bought: 0,
            total_cargo_sold: 0,
            airports_visited: Vec::new(),
            favorite_cargo: None,
            most_profitable_route: None,
            peak_money: starting_money,
            lowest_money: starting_money,
            times_went_broke: 0,
        }
    }

    pub fn record_trade(&mut self, profit: i64, _cargo_type: String, quantity: u32, is_buy: bool) {
        self.total_trades += 1;

        if is_buy {
            self.total_cargo_bought += quantity;
        } else {
            self.total_cargo_sold += quantity;

            if profit > 0 {
                self.successful_trades += 1;
                self.total_profit += profit;
                if profit > self.best_trade_profit {
                    self.best_trade_profit = profit;
                }
            } else if profit < 0 {
                self.total_loss += profit.abs();
                if profit < self.worst_trade_loss {
                    self.worst_trade_loss = profit;
                }
            }
        }
    }

    pub fn record_travel(&mut self, distance: f64, fuel_consumed: u32, _from: String, to: String) {
        self.total_distance_traveled += distance;
        self.total_fuel_consumed += fuel_consumed;

        if !self.airports_visited.contains(&to) {
            self.airports_visited.push(to.clone());
        }
    }

    pub fn record_fuel_purchase(&mut self, amount: u32) {
        self.total_fuel_purchased += amount;
    }

    pub fn update_money_stats(&mut self, current_money: u32) {
        if current_money > self.peak_money {
            self.peak_money = current_money;
        }
        if current_money < self.lowest_money {
            self.lowest_money = current_money;
        }
        if current_money == 0 {
            self.times_went_broke += 1;
        }
    }

    pub fn get_success_rate(&self) -> f32 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.successful_trades as f32 / self.total_trades as f32) * 100.0
        }
    }

    pub fn get_net_profit(&self) -> i64 {
        self.total_profit - self.total_loss
    }

    #[allow(dead_code)]
    pub fn get_average_profit_per_trade(&self) -> i64 {
        if self.total_trades == 0 {
            0
        } else {
            self.get_net_profit() / self.total_trades as i64
        }
    }

    pub fn get_fuel_efficiency(&self) -> f64 {
        if self.total_fuel_consumed == 0 {
            0.0
        } else {
            self.total_distance_traveled / self.total_fuel_consumed as f64
        }
    }
}
