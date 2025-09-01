# CLAUDE.md - Context for Claude Code Sessions

## Project Overview

**KZRK** is a classic buy-low-sell-high trading game set in the aviation world, inspired by games like Dope Wars. Players take on the role of a pilot who travels between airports, trading various cargo types to maximize profit while managing fuel costs and travel distances.

## Current Implementation Status

### ✅ Completed Features
- **Core Data Models**: Player, Airport, CargoType, Market, GameState structs implemented
- **Market System**: Dynamic pricing, buy/sell functionality, inventory management
- **Travel System**: Fuel consumption calculation, travel validation, location updates
- **Game Loop & UI**: Terminal-based UI with input handling and menu system
- **REST API**: Complete API server with all game endpoints documented
- **Testing Infrastructure**: Comprehensive test suite including integration, property, and performance tests
- **Development Tooling**: Full CI/CD pipeline with linting, formatting, security audits

### 🚧 In Progress/Future Features
- Game balance and polish (Phase 5 from GAME_DESIGN.md)
- Win/lose conditions
- Market events system
- Save/load game state
- Multiple plane types
- Achievement system

## Project Structure

```
src/
├── main.rs              # Entry point - supports both CLI and API modes
├── lib.rs               # Library exports
├── api/                 # REST API implementation
│   ├── handlers.rs      # Request handlers
│   ├── models.rs        # API data models
│   ├── routes.rs        # Route definitions
│   └── service.rs       # Game service layer
├── data/                # Static game content
│   ├── airports.rs      # Airport definitions (6 airports: JFK, LAX, MIA, ORD, DEN, SEA)
│   └── cargo_types.rs   # Cargo type definitions (6 types: electronics, food, textiles, etc.)
├── models/              # Core game data structures
│   ├── player.rs        # Player state and methods
│   ├── airport.rs       # Airport and world data
│   ├── cargo.rs         # Cargo types and inventory
│   └── market.rs        # Market prices and trading
├── systems/             # Game logic systems
│   ├── trading.rs       # Buy/sell logic
│   ├── travel.rs        # Movement and fuel consumption  
│   ├── market.rs        # Price generation and updates
│   ├── events.rs        # Market events (future feature)
│   └── game.rs          # Main game state management
└── ui/                  # User interface
    ├── terminal.rs      # Terminal-based UI implementation
    └── mod.rs
```

## Key Dependencies

```toml
[dependencies]
rand = "0.8"                    # Price randomization
serde = "1.0"                   # Serialization
axum = "0.7"                    # Web framework
tokio = "1.0"                   # Async runtime
uuid = "1.0"                    # Session IDs
tracing = "0.1"                 # Logging
```

## Running the Game

### CLI Mode
```bash
cargo run                       # Standard game
KZRK_CHEAT=1 cargo run         # Unlimited fuel cheat mode
```

### API Server Mode
```bash
cargo run api                   # Starts REST API on http://127.0.0.1:3000
```

## Development Commands

### Essential Commands
```bash
# Testing
cargo test                      # All tests
make test                       # All tests (via Makefile)

# Linting & Formatting  
make lint                       # Format + clippy + audit + security
cargo fmt                       # Format code
cargo clippy                    # Lint code

# Development cycle
make dev                        # fmt + lint + test
make ci                         # Full CI pipeline locally
```

### Testing Strategy
- **Unit tests**: Core game logic testing
- **Integration tests**: API endpoint testing
- **Property tests**: Randomized testing with proptest
- **Performance tests**: Benchmark critical paths
- **Error scenario tests**: Edge cases and error handling

## Game Mechanics Summary

### Core Loop
1. **Market Phase**: Buy/sell cargo at current airport
2. **Travel Phase**: Choose destination and fly (consumes fuel)
3. **Arrival**: Updated market prices at new location
4. **Repeat**: Continue trading and traveling

### Key Stats
- **Starting Money**: $5,000 (configurable via API)
- **Starting Fuel**: Full tank (varies by plane)
- **Starting Location**: JFK (configurable via API)
- **Win Condition**: Not yet implemented (planned: $100,000)

### Cargo Types
1. **Electronics** - High value, volatile prices
2. **Food & Beverages** - Steady demand, perishable
3. **Textiles** - Moderate value, good volume
4. **Industrial Parts** - Heavy, regional demand
5. **Luxury Goods** - Very high value, small quantities
6. **Raw Materials** - Low value per weight, bulk commodity

### Airports
1. **New York (JFK)** - Financial hub, electronics/luxury goods
2. **Los Angeles (LAX)** - Entertainment/tech goods  
3. **Miami (MIA)** - South American trade, agricultural products
4. **Chicago (ORD)** - Industrial goods, central location
5. **Denver (DEN)** - Mountain region, outdoor equipment
6. **Seattle (SEA)** - Tech goods, coffee, aircraft parts

## API Documentation

Complete REST API with endpoints for:
- **Game Management**: Create game, get state
- **Actions**: Travel, trade cargo, buy fuel
- **Reference**: List airports and cargo types

See `API.md` for full endpoint documentation with examples.

## Development Guidelines

### Code Style
- Follow existing Rust conventions
- Use `cargo fmt` for formatting (configured in `rustfmt.toml`)
- Address all `cargo clippy` warnings (strict mode enabled)
- Maintain comprehensive test coverage

### Security & Dependencies
- Regular security audits via `cargo audit`
- License compliance checking via `cargo deny`
- No secrets or keys in repository
- All external inputs validated

### Git Workflow
- Main branch: `main` (note: currently on `develop` branch)
- Clean commit history
- Descriptive commit messages

## Useful Context for Claude Sessions

### When working on game balance:
- Check `GAME_DESIGN.md` Phase 5 items
- Test with both CLI and API modes
- Consider fuel efficiency vs. cargo capacity tradeoffs
- Market price ranges defined in `data/` modules

### When adding new features:
- Follow existing module structure
- Add comprehensive tests (unit + integration)
- Update API documentation if adding endpoints
- Consider both CLI and API compatibility

### When debugging:
- Use `RUST_BACKTRACE=1` for detailed stack traces
- API server logs available via `tracing` framework
- Cheat mode available: `KZRK_CHEAT=1` for unlimited fuel

### Performance considerations:
- Market price calculations are CPU-intensive
- Distance calculations are cached
- Benchmarks available via `cargo bench`

## Common Tasks

### Adding a new cargo type:
1. Update `src/data/cargo_types.rs`
2. Add to airport market profiles in `src/data/airports.rs`
3. Update tests and documentation

### Adding a new airport:
1. Update `src/data/airports.rs`
2. Add to distance calculation matrix
3. Define market profile for cargo types
4. Update tests and API examples

### Balancing game difficulty:
1. Adjust price ranges in cargo type definitions
2. Modify fuel consumption rates
3. Update starting conditions in game initialization
4. Test via property tests for edge cases

This document should provide sufficient context for future Claude Code sessions to quickly understand and work with the KZRK codebase effectively.