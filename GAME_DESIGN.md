# Aviation Trading Game Design Document

## Game Overview

A classic buy-low-sell-high trading game set in the aviation world, inspired by games like Dope Wars. Players take on the role of a pilot who travels between airports, trading various cargo types to maximize profit while managing fuel costs and travel distances.

## Core Game Loop

1. **Start Turn**: View current airport, money, fuel, and cargo
2. **Market Phase**: Buy/sell cargo at current airport
3. **Travel Phase**: Choose destination airport and fly (consuming fuel)
4. **Arrival**: Arrive at new airport with updated market prices
5. **Repeat**: Continue trading and traveling

## Core Game Systems

### 1. Player System
- **Money**: Starting capital and profits from trading
- **Fuel**: Required for travel, purchasable at airports
- **Current Location**: Which airport the player is at
- **Cargo Inventory**: What cargo is being carried (with capacity limits)
- **Plane Stats**: Cargo capacity, fuel efficiency, fuel tank size

### 2. World System
- **Airports**: Various locations with unique characteristics
- **Distance Matrix**: Travel distances between airports
- **Regional Economy**: Different airports specialize in different cargo types

### 3. Market System
- **Dynamic Pricing**: Randomized prices that fluctuate over time
- **Supply & Demand**: Some airports are better for buying certain goods, others for selling
- **Price Ranges**: Each cargo type has min/max price ranges per airport
- **Market Events**: Occasional events that dramatically affect prices

### 4. Trading System
- **Buy/Sell Interface**: Purchase and sell cargo at current airport
- **Inventory Management**: Track what's being carried
- **Capacity Limits**: Players can't carry unlimited cargo

### 5. Travel System
- **Route Selection**: Choose destination from available airports
- **Fuel Consumption**: Based on distance and plane efficiency
- **Fuel Management**: Must have enough fuel to reach destination

## Data Models

### Airport
```rust
pub struct Airport {
    pub id: String,
    pub name: String,
    pub coordinates: (f64, f64),  // For distance calculations
    pub base_fuel_price: u32,     // Base price for fuel
    pub market_profile: MarketProfile, // What cargo types are common here
}
```

### Cargo Type
```rust
pub struct CargoType {
    pub id: String,
    pub name: String,
    pub base_price: u32,          // Base price for price calculations
    pub weight_per_unit: u32,     // For cargo capacity calculations
    pub volatility: f32,          // How much prices fluctuate
}
```

### Player
```rust
pub struct Player {
    pub money: u32,
    pub current_airport: String,
    pub fuel: u32,
    pub max_fuel: u32,
    pub cargo_inventory: HashMap<String, u32>, // cargo_id -> quantity
    pub max_cargo_weight: u32,
    pub fuel_efficiency: f32,     // Distance per fuel unit
}
```

### Market
```rust
pub struct Market {
    pub airport_id: String,
    pub fuel_price: u32,
    pub cargo_prices: HashMap<String, u32>, // cargo_id -> current_price
    pub last_updated: SystemTime,
}
```

### Game State
```rust
pub struct GameState {
    pub player: Player,
    pub airports: HashMap<String, Airport>,
    pub cargo_types: HashMap<String, CargoType>,
    pub markets: HashMap<String, Market>, // airport_id -> market
    pub distance_cache: HashMap<(String, String), f64>,
    pub turn_number: u32,
}
```

## Game Content

### Initial Airports
1. **New York (JFK)** - Financial hub, good for electronics/luxury goods
2. **Los Angeles (LAX)** - Entertainment/tech goods
3. **Miami (MIA)** - South American trade, agricultural products
4. **Chicago (ORD)** - Industrial goods, central location
5. **Denver (DEN)** - Mountain region, outdoor equipment
6. **Seattle (SEA)** - Tech goods, coffee, aircraft parts

### Cargo Types
1. **Electronics** - High value, low weight, volatile prices
2. **Food & Beverages** - Perishable, steady demand
3. **Textiles** - Moderate value, good volume
4. **Industrial Parts** - Heavy, regional demand
5. **Luxury Goods** - Very high value, small quantities
6. **Raw Materials** - Low value per weight, bulk commodity

### Market Dynamics
- Each airport has 2-3 cargo types it "produces" (lower buy prices)
- Each airport has 2-3 cargo types it "consumes" (higher sell prices)
- Prices fluctuate ±20-50% from base prices
- Special events can cause dramatic price spikes/crashes

## Technical Architecture

### Module Structure
```
src/
├── main.rs              # Game entry point and main loop
├── models/              # Data structures
│   ├── mod.rs
│   ├── player.rs        # Player struct and methods
│   ├── airport.rs       # Airport and world data
│   ├── cargo.rs         # Cargo types and inventory
│   └── market.rs        # Market prices and trading
├── systems/             # Game logic systems
│   ├── mod.rs
│   ├── trading.rs       # Buy/sell logic
│   ├── travel.rs        # Movement and fuel consumption
│   ├── market.rs        # Price generation and updates
│   └── game.rs          # Main game state management
├── ui/                  # User interface
│   ├── mod.rs
│   └── terminal.rs      # Terminal-based UI
└── data/                # Game content
    ├── mod.rs
    ├── airports.rs      # Static airport data
    └── cargo_types.rs   # Static cargo type definitions
```

### Key Dependencies
- `rand` - For price randomization and market events
- `serde` - For save/load game state (future feature)
- `crossterm` - For better terminal UI (if we want fancy display)

## Implementation Phases

### Phase 1: Core Data Models
- [x] Define basic structs for Player, Airport, CargoType, Market
- [x] Create game state structure
- [x] Implement basic distance calculations
- [x] Add static data for airports and cargo types

### Phase 2: Market System
- [x] Implement price generation algorithm
- [x] Create market update system
- [x] Add buy/sell functionality
- [x] Handle inventory management

### Phase 3: Travel System
- [x] Implement fuel consumption calculation
- [x] Add travel validation (enough fuel, valid destination)
- [x] Update player location
- [x] Refresh market prices on arrival

### Phase 4: Game Loop & UI
- [x] Create main game loop
- [x] Implement terminal-based UI
- [x] Add input handling and menu system
- [x] Display current status, market prices, and options

### Phase 5: Game Balance & Polish
- [ ] Balance starting conditions and price ranges
- [ ] Add win/lose conditions
- [ ] Implement market events
- [ ] Add game statistics and scoring

### Phase 6: Advanced Features (Future)
- [ ] Save/load game state
- [ ] Multiple plane types with different stats
- [ ] Loan system for borrowing money
- [ ] Random events (weather, market crashes, etc.)
- [ ] Achievement system

## Starting Conditions

- **Money**: $10,000
- **Fuel**: 100 units (full tank)
- **Location**: Chicago (ORD) - central location
- **Cargo**: Empty
- **Objective**: Reach $100,000 to "win" the game

## Success Metrics

A successful implementation should:
1. Provide engaging risk/reward trading decisions
2. Create meaningful fuel management challenges
3. Generate interesting market dynamics
4. Offer multiple viable strategies for success
5. Be replayable with different outcomes

This design provides a solid foundation for a classic trading game with aviation flavor, while keeping the scope manageable for initial development.