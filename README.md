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

### API Server Mode
```bash
cargo run api               # REST API server on localhost:3000
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
