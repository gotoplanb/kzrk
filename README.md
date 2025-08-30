# kzrk

A classic buy-low-sell-high trading game set in the aviation world, inspired by games like Dope Wars. Players take on the role of a pilot who travels between airports, trading various cargo types to maximize profit while managing fuel costs and travel distances.

## Dependencies

1. Install Rust with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Running

1. `git clone https://github.com/gotoplanb/kzrk.git`
1. `cd kzrk`
1. `export KZRK_CHEAT=1` for unlimited fuel
1. `cargo run` - CLI mode
1. `cargo run api` - REST API server mode

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
