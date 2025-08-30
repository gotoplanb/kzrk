use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kzrk::api::service::GameService;
use kzrk::api::models::*;
use kzrk::data::{airports::get_default_airports, cargo_types::get_default_cargo_types};
use kzrk::models::Player;

fn bench_distance_calculations(c: &mut Criterion) {
    let airports = get_default_airports();
    let airport_pairs: Vec<_> = airports.iter()
        .flat_map(|(_, a1)| airports.values().map(move |a2| (a1, a2)))
        .collect();

    c.bench_function("distance_calculation", |b| {
        b.iter(|| {
            for (a1, a2) in &airport_pairs {
                black_box(a1.distance_to(a2));
            }
        });
    });

    // Benchmark single distance calculation
    let jfk = &airports["JFK"];
    let lax = &airports["LAX"];
    
    c.bench_function("single_distance_calculation", |b| {
        b.iter(|| {
            black_box(jfk.distance_to(lax))
        });
    });
}

fn bench_player_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("player_operations");
    
    for money_amount in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("player_creation", money_amount),
            money_amount,
            |b, &money| {
                b.iter(|| {
                    black_box(Player::new(money, "JFK", 200, 1000, 15.0))
                });
            }
        );
    }

    let player = Player::new(10000, "JFK", 200, 1000, 15.0);
    
    group.bench_function("spend_money", |b| {
        b.iter(|| {
            let mut p = player.clone();
            black_box(p.spend_money(100));
        });
    });

    group.bench_function("earn_money", |b| {
        b.iter(|| {
            let mut p = player.clone();
            p.earn_money(100);
            black_box(());
        });
    });

    group.bench_function("consume_fuel", |b| {
        b.iter(|| {
            let mut p = player.clone();
            black_box(p.consume_fuel(10));
        });
    });

    group.bench_function("add_fuel", |b| {
        b.iter(|| {
            let mut p = player.clone();
            p.add_fuel(10);
            black_box(());
        });
    });

    let cargo_types = get_default_cargo_types();
    group.bench_function("can_carry_more_weight", |b| {
        b.iter(|| {
            black_box(player.can_carry_more_weight(100, &cargo_types));
        });
    });

    group.bench_function("fuel_needed_for_distance", |b| {
        b.iter(|| {
            black_box(player.fuel_needed_for_distance(1000.0));
        });
    });

    group.finish();
}

fn bench_cargo_inventory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cargo_inventory");

    // Benchmark inventory operations
    group.bench_function("add_cargo", |b| {
        b.iter(|| {
            let mut player = Player::new(10000, "JFK", 200, 1000, 15.0);
            player.cargo_inventory.add_cargo("electronics", 5);
            black_box(());
        });
    });

    group.bench_function("get_quantity", |b| {
        let mut player = Player::new(10000, "JFK", 200, 1000, 15.0);
        player.cargo_inventory.add_cargo("electronics", 10);
        
        b.iter(|| {
            black_box(player.cargo_inventory.get_quantity("electronics"));
        });
    });

    group.bench_function("remove_cargo", |b| {
        b.iter(|| {
            let mut player = Player::new(10000, "JFK", 200, 1000, 15.0);
            player.cargo_inventory.add_cargo("electronics", 10);
            player.cargo_inventory.remove_cargo("electronics", 5);
            black_box(());
        });
    });

    let cargo_types = get_default_cargo_types();
    group.bench_function("total_weight_calculation", |b| {
        let mut player = Player::new(10000, "JFK", 200, 1000, 15.0);
        player.cargo_inventory.add_cargo("electronics", 5);
        player.cargo_inventory.add_cargo("textiles", 3);
        player.cargo_inventory.add_cargo("luxury", 2);
        
        b.iter(|| {
            black_box(player.cargo_inventory.total_weight(&cargo_types));
        });
    });

    group.finish();
}

#[allow(dead_code)]
async fn bench_api_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_operations");
    group.sample_size(50); // Reduce sample size for slower operations

    let service = GameService::new();

    group.bench_function("create_game", |b| {
        b.iter(|| {
            let request = CreateGameRequest {
                player_name: "Benchmark Player".to_string(),
                starting_money: Some(10000),
                starting_airport: Some("JFK".to_string()),
            };
            let _ = service.create_game(request);
        });
    });

    // Create a game for subsequent benchmarks
    let create_request = CreateGameRequest {
        player_name: "Benchmark Player".to_string(),
        starting_money: Some(10000),
        starting_airport: Some("JFK".to_string()),
    };
    let game_response = service.create_game(create_request).unwrap();
    let session_id = game_response.session_id;

    group.bench_function("get_game_state", |b| {
        b.iter(|| {
            let _ = service.get_game_state(session_id);
        });
    });

    group.bench_function("fuel_purchase", |b| {
        b.iter(|| {
            let request = FuelRequest { quantity: 50 };
            // Note: This will eventually fail when player runs out of money,
            // but should give us performance measurements for the successful cases
            let _ = service.buy_fuel(session_id, request);
        });
    });

    group.bench_function("cargo_trade_buy", |b| {
        b.iter(|| {
            let request = TradeRequest {
                cargo_type: "electronics".to_string(),
                quantity: 1,
                action: TradeAction::Buy,
            };
            let _ = service.trade(session_id, request);
        });
    });

    group.finish();
}

fn bench_market_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("market_operations");

    use kzrk::models::Market;
    
    group.bench_function("market_creation", |b| {
        b.iter(|| {
            black_box(Market::new("TEST", 100));
        });
    });

    group.bench_function("set_cargo_price", |b| {
        b.iter(|| {
            let mut market = Market::new("TEST", 100);
            market.set_cargo_price("electronics", 500);
            black_box(());
        });
    });

    group.bench_function("get_cargo_price", |b| {
        let mut market = Market::new("TEST", 100);
        market.set_cargo_price("electronics", 500);
        
        b.iter(|| {
            black_box(market.get_cargo_price("electronics"));
        });
    });

    group.bench_function("update_fuel_price", |b| {
        b.iter(|| {
            let mut market = Market::new("TEST", 100);
            market.update_fuel_price(120);
            black_box(());
        });
    });

    group.finish();
}

fn bench_data_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_loading");

    group.bench_function("load_airports", |b| {
        b.iter(|| {
            black_box(get_default_airports());
        });
    });

    group.bench_function("load_cargo_types", |b| {
        b.iter(|| {
            black_box(get_default_cargo_types());
        });
    });

    group.finish();
}

fn bench_game_state_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_state");
    
    use kzrk::systems::GameState;
    
    group.bench_function("game_state_creation", |b| {
        b.iter(|| {
            let airports = get_default_airports();
            let cargo_types = get_default_cargo_types();
            black_box(GameState::new(airports, cargo_types));
        });
    });

    let airports = get_default_airports();
    let cargo_types = get_default_cargo_types();
    let game_state = GameState::new(airports, cargo_types);

    group.bench_function("get_current_market", |b| {
        b.iter(|| {
            black_box(game_state.get_current_market());
        });
    });

    group.finish();
}

// Benchmark realistic game scenarios
fn bench_realistic_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_scenarios");
    group.sample_size(20);

    // Simulate a complete trading round
    group.bench_function("complete_trading_round", |b| {
        b.iter(|| {
            let service = GameService::new();
            
            // Create game
            let create_request = CreateGameRequest {
                player_name: "Scenario Player".to_string(),
                starting_money: Some(15000),
                starting_airport: Some("JFK".to_string()),
            };
            let game_response = service.create_game(create_request).unwrap();
            let session_id = game_response.session_id;

            // Buy fuel
            let fuel_request = FuelRequest { quantity: 100 };
            let _ = service.buy_fuel(session_id, fuel_request);

            // Buy cargo
            let trade_request = TradeRequest {
                cargo_type: "electronics".to_string(),
                quantity: 2,
                action: TradeAction::Buy,
            };
            let _ = service.trade(session_id, trade_request);

            // Travel
            let travel_request = TravelRequest {
                destination: "ORD".to_string(),
            };
            let _ = service.travel(session_id, travel_request);

            // Sell cargo
            let sell_request = TradeRequest {
                cargo_type: "electronics".to_string(),
                quantity: 2,
                action: TradeAction::Sell,
            };
            let _ = service.trade(session_id, sell_request);

            black_box(session_id);
        });
    });

    group.finish();
}

// Create benchmark groups
criterion_group!(
    benches,
    bench_distance_calculations,
    bench_player_operations,
    bench_cargo_inventory_operations,
    bench_market_operations,
    bench_data_loading,
    bench_game_state_operations,
    bench_realistic_scenarios
);

// For async benchmarks (commented out as criterion doesn't directly support async)
// We'll need to use tokio runtime for the API benchmarks
#[allow(dead_code)]
fn bench_api_sync(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        bench_api_operations(c).await;
    });
}

criterion_main!(benches);