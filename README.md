# kzrk

A classic buy-low-sell-high trading game set in the aviation world, inspired by games like Dope Wars. Players take on the role of a pilot who travels between airports, trading various cargo types to maximize profit while managing fuel costs and travel distances.

[![codecov](https://codecov.io/github/gotoplanb/kzrk/branch/main/graph/badge.svg?token=F1KGS5JF3G)](https://codecov.io/github/gotoplanb/kzrk)

## Dependencies

1. Install Rust with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Running

1. `git clone https://github.com/gotoplanb/kzrk.git`
1. `cd kzrk`
1. Choose your preferred mode:

### CLI Mode (Default)
```bash
cargo run                    # Terminal-based interface
export KZRK_CHEAT=1 cargo run  # With cheat mode (unlimited fuel)
```

### GUI Mode (egui-based)
```bash
cargo run --features gui gui    # Sierra Online-style GUI interface
KZRK_CHEAT=1 cargo run --features gui gui  # GUI with cheat mode
```
Features a classic adventure game interface with:
- Scene-based navigation through airport FBO locations
- Visual market boards with price analysis
- Enhanced trading desk with transaction previews
- Interactive flight planning with destination details
- Professional fuel pump interface

### Multiplayer Mode
Play with friends! One person runs a server, others connect with GUI clients:

#### Server (Host)
```bash
cargo run api               # Start multiplayer server on localhost:3000
```

#### Client (Players)
```bash
cargo run --features gui gui    # Launch GUI client
```

**How it works:**
1. **Host starts the server**: The host runs `cargo run api` to start the multiplayer server
2. **Players connect**: Each player runs `cargo run --features gui gui` and enters the server address (default: `127.0.0.1:3000`)
3. **Create or join rooms**: Players can create new game rooms or join existing ones
4. **Play together**: Up to 8 players per room, with shared world state and real-time market updates

**Features:**
- Room-based multiplayer (1-8 players per room)
- Real-time player position tracking
- Shared market economics - your trades affect other players' prices
- Host can be any player - no special privileges required
- Automatic room discovery and joining
- Player name validation and collision handling

**Network Setup:**
- **Local play**: Use default `127.0.0.1:3000` (host and all players on same machine/network)
- **Remote play**: Host needs to share their IP address (e.g., `192.168.1.100:3000`)
- **Firewall**: Make sure port 3000 is open on the host machine

### API Server Mode (Single Player)
```bash
cargo run api               # REST API server on localhost:3000 (for development)
```

## Testing

```bash
cargo test                           # Run all tests
cargo test --test api_integration_tests  # Run API integration tests
cargo test --test property_tests     # Run property-based tests
cargo test --test error_scenario_tests   # Run error scenario tests
```

## Development

### Linting & Formatting
```bash
cargo fmt                           # Format code
cargo fmt --all -- --check         # Check formatting (CI mode)
cargo clippy                        # Run linter
cargo clippy --all-targets --all-features -- -D warnings  # Strict linting
```

### Security & Dependencies
```bash
cargo audit                         # Security audit
cargo deny check                    # License/dependency checks
```

### Benchmarks
```bash
cargo bench                         # Run performance benchmarks
```

### Convenience Commands (via Makefile)
```bash
make test                          # Run all tests
make lint                          # Format + clippy + audit + deny
make bench                         # Run benchmarks
make ci                           # Full CI pipeline locally
```
