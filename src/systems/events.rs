use crate::models::{Airport, CargoType, Market};
use rand::Rng;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub event_type: MarketEventType,
    pub affected_cargo: String,
    pub affected_airport: String,
    pub price_multiplier: f32,
    pub duration_turns: u32,
    pub turns_remaining: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEventType {
    PriceSpike,     // Sudden high demand increases prices
    PriceCrash,     // Market oversupply crashes prices
    Shortage,       // Limited supply reduces availability
    Boom,           // Regional economic boom affects multiple goods
    Recession,      // Economic downturn lowers all prices
    NewsEvent,      // External news affects specific cargo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatistics {
    pub total_revenue: u32,
    pub total_expenses: u32,
    pub net_profit: u32,
    pub cargo_trades: u32,
    pub fuel_purchased: u32,
    pub distances_traveled: f64,
    pub airports_visited: Vec<String>,
    pub best_single_trade: u32,
    pub most_profitable_cargo: String,
    pub efficiency_score: f32,
}

impl Default for GameStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStatistics {
    pub fn new() -> Self {
        Self {
            total_revenue: 0,
            total_expenses: 0,
            net_profit: 0,
            cargo_trades: 0,
            fuel_purchased: 0,
            distances_traveled: 0.0,
            airports_visited: Vec::new(),
            best_single_trade: 0,
            most_profitable_cargo: String::new(),
            efficiency_score: 0.0,
        }
    }
    
    pub fn record_sale(&mut self, cargo_type: &str, revenue: u32) {
        self.total_revenue += revenue;
        self.net_profit = self.total_revenue.saturating_sub(self.total_expenses);
        self.cargo_trades += 1;
        
        if revenue > self.best_single_trade {
            self.best_single_trade = revenue;
            self.most_profitable_cargo = cargo_type.to_string();
        }
    }
    
    pub fn record_purchase(&mut self, expense: u32) {
        self.total_expenses += expense;
        self.net_profit = self.total_revenue.saturating_sub(self.total_expenses);
    }
    
    pub fn record_cargo_purchase(&mut self, expense: u32) {
        self.record_purchase(expense);
        self.cargo_trades += 1;
    }
    
    pub fn record_fuel_purchase(&mut self, fuel_amount: u32, cost: u32) {
        self.fuel_purchased += fuel_amount;
        self.record_purchase(cost);
    }
    
    pub fn record_travel(&mut self, airport: &str, distance: f64) {
        self.distances_traveled += distance;
        if !self.airports_visited.contains(&airport.to_string()) {
            self.airports_visited.push(airport.to_string());
        }
    }
    
    #[allow(dead_code)]
    pub fn calculate_efficiency(&mut self, turns: u32) {
        if turns > 0 {
            self.efficiency_score = self.net_profit as f32 / turns as f32;
        }
    }
}

#[allow(dead_code)]
pub struct EventSystem;

#[allow(dead_code)]
impl EventSystem {
    pub fn generate_random_event(
        airports: &HashMap<String, Airport>,
        cargo_types: &HashMap<String, CargoType>,
        rng: &mut impl Rng,
    ) -> Option<MarketEvent> {
        // 15% chance of generating an event each turn
        if rng.gen_range(0.0..1.0) > 0.15 {
            return None;
        }
        
        let event_types = [
            MarketEventType::PriceSpike,
            MarketEventType::PriceCrash,
            MarketEventType::Shortage,
            MarketEventType::NewsEvent,
        ];
        
        let event_type = event_types[rng.gen_range(0..event_types.len())].clone();
        
        // Pick random cargo and airport
        let cargo_ids: Vec<_> = cargo_types.keys().collect();
        let airport_ids: Vec<_> = airports.keys().collect();
        
        let affected_cargo = cargo_ids[rng.gen_range(0..cargo_ids.len())].clone();
        let affected_airport = airport_ids[rng.gen_range(0..airport_ids.len())].clone();
        
        let (multiplier, duration, description) = match event_type {
            MarketEventType::PriceSpike => {
                let mult = rng.gen_range(1.5..2.5);
                let desc = Self::generate_spike_description(&affected_cargo, &affected_airport, airports, cargo_types);
                (mult, rng.gen_range(3..8), desc)
            },
            MarketEventType::PriceCrash => {
                let mult = rng.gen_range(0.3..0.7);
                let desc = Self::generate_crash_description(&affected_cargo, &affected_airport, airports, cargo_types);
                (mult, rng.gen_range(4..10), desc)
            },
            MarketEventType::Shortage => {
                let mult = rng.gen_range(1.8..3.0);
                let desc = Self::generate_shortage_description(&affected_cargo, &affected_airport, airports, cargo_types);
                (mult, rng.gen_range(2..6), desc)
            },
            MarketEventType::NewsEvent => {
                let mult = if rng.gen_bool(0.6) { 
                    rng.gen_range(1.3..2.0) // Positive news
                } else { 
                    rng.gen_range(0.5..0.8) // Negative news
                };
                let desc = Self::generate_news_description(&affected_cargo, mult > 1.0);
                (mult, rng.gen_range(5..12), desc)
            },
            _ => return None,
        };
        
        Some(MarketEvent {
            event_type,
            affected_cargo,
            affected_airport,
            price_multiplier: multiplier,
            duration_turns: duration,
            turns_remaining: duration,
            description,
        })
    }
    
    fn generate_spike_description(cargo: &str, airport: &str, airports: &HashMap<String, Airport>, cargo_types: &HashMap<String, CargoType>) -> String {
        let airport_name = airports.get(airport).map(|a| a.name.as_str()).unwrap_or(airport);
        let cargo_name = cargo_types.get(cargo).map(|c| c.name.as_str()).unwrap_or(cargo);
        
        let scenarios = [
            format!("🔥 BREAKING: Factory fire at {} creates urgent demand for {}!", airport_name, cargo_name),
            format!("📈 MARKET ALERT: Supply chain disruption drives up {} prices at {}", cargo_name, airport_name),
            format!("⚡ URGENT: Emergency stockpiling of {} needed at {} due to regulations", cargo_name, airport_name),
            format!("🎯 HOT COMMODITY: Unexpected surge in {} demand at {} - prices soaring!", cargo_name, airport_name),
        ];
        
        scenarios[rand::thread_rng().gen_range(0..scenarios.len())].clone()
    }
    
    fn generate_crash_description(cargo: &str, airport: &str, airports: &HashMap<String, Airport>, cargo_types: &HashMap<String, CargoType>) -> String {
        let airport_name = airports.get(airport).map(|a| a.name.as_str()).unwrap_or(airport);
        let cargo_name = cargo_types.get(cargo).map(|c| c.name.as_str()).unwrap_or(cargo);
        
        let scenarios = [
            format!("📉 MARKET CRASH: Oversupply of {} floods {} market - prices plummet!", cargo_name, airport_name),
            format!("⬇️ PRICE ALERT: Major competitor dumps {} inventory at {} - prices collapse", cargo_name, airport_name),
            format!("💔 DEMAND SHOCK: Economic slowdown kills {} demand at {}", cargo_name, airport_name),
            format!("📊 BEARISH: Analysts downgrade {} outlook for {} region", cargo_name, airport_name),
        ];
        
        scenarios[rand::thread_rng().gen_range(0..scenarios.len())].clone()
    }
    
    fn generate_shortage_description(cargo: &str, airport: &str, airports: &HashMap<String, Airport>, cargo_types: &HashMap<String, CargoType>) -> String {
        let airport_name = airports.get(airport).map(|a| a.name.as_str()).unwrap_or(airport);
        let cargo_name = cargo_types.get(cargo).map(|c| c.name.as_str()).unwrap_or(cargo);
        
        let scenarios = [
            format!("⚠️ SHORTAGE: Critical {} shortage at {} - limited supply available", cargo_name, airport_name),
            format!("🚨 SUPPLY CRISIS: {} production halted - {} faces severe shortage", cargo_name, airport_name),
            format!("⛔ RESTRICTED: Export controls limit {} availability at {}", cargo_name, airport_name),
            format!("🔒 SCARCITY: Strike action disrupts {} supply chain to {}", cargo_name, airport_name),
        ];
        
        scenarios[rand::thread_rng().gen_range(0..scenarios.len())].clone()
    }
    
    fn generate_news_description(cargo: &str, is_positive: bool) -> String {
        let cargo_name = cargo.replace("_", " ");
        
        if is_positive {
            let scenarios = [
                format!("📰 BUSINESS NEWS: New technology breakthrough boosts {} industry outlook", cargo_name),
                format!("🌟 INNOVATION: Revolutionary {} applications drive market optimism", cargo_name),
                format!("📈 ANALYST UPGRADE: {} sector receives strong buy rating from Wall Street", cargo_name),
                format!("🚀 GROWTH STORY: {} demand expected to surge on global expansion", cargo_name),
            ];
            scenarios[rand::thread_rng().gen_range(0..scenarios.len())].clone()
        } else {
            let scenarios = [
                format!("📰 MARKET NEWS: Regulatory concerns weigh on {} sector performance", cargo_name),
                format!("⚠️ INDUSTRY ALERT: Environmental issues impact {} market sentiment", cargo_name),
                format!("📉 ANALYST DOWNGRADE: {} sector faces headwinds from economic uncertainty", cargo_name),
                format!("🌧️ CLOUDY OUTLOOK: Global tensions affect {} international trade", cargo_name),
            ];
            scenarios[rand::thread_rng().gen_range(0..scenarios.len())].clone()
        }
    }
    
    pub fn apply_event_to_market(event: &MarketEvent, market: &mut Market) {
        if let Some(current_price) = market.get_cargo_price(&event.affected_cargo) {
            let new_price = (current_price as f32 * event.price_multiplier) as u32;
            let new_price = new_price.max(1); // Ensure minimum price of $1
            market.set_cargo_price(&event.affected_cargo, new_price);
        }
    }
    
    pub fn update_events(events: &mut Vec<MarketEvent>) -> Vec<String> {
        let mut expired_events = Vec::new();
        
        events.retain_mut(|event| {
            event.turns_remaining = event.turns_remaining.saturating_sub(1);
            if event.turns_remaining == 0 {
                expired_events.push(format!("Market conditions normalize: {} effects have ended", 
                    event.description.split(':').next().unwrap_or("Event")));
                false
            } else {
                true
            }
        });
        
        expired_events
    }
}