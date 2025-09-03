# Development tooling makefile

.PHONY: all test lint fmt audit check clean bench install-tools

# Default target
all: fmt lint test

# Install development tools
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-audit
	cargo install cargo-deny
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	rustup component add clippy rustfmt

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Check formatting without changing files
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

# Run clippy lints and check formatting
lint: fmt-check
	@echo "Running clippy lints..."
	cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
	@echo "Running all tests..."
	cargo test --all-features --lib --bins --test api_integration_tests --test error_scenario_tests --test game_integration_tests --test performance_tests --test property_tests --test multiplayer_persistence_tests
	cargo test --all-features --test save_system_tests -- --test-threads=1

# Run integration tests only
test-integration:
	@echo "Running integration tests..."
	cargo test --test api_integration_tests
	cargo test --test property_tests

# Run unit tests only
test-unit:
	@echo "Running unit tests..."
	cargo test --lib --bins

# Security audit
audit:
	@echo "Running security audit..."
	cargo audit

# License and dependency checking
deny:
	@echo "Checking dependencies and licenses..."
	cargo deny check

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	cargo bench

# Full check (format, lint, test, audit)
check: fmt-check lint test audit

# Generate test coverage report
coverage:
	@echo "Generating test coverage report..."
	cargo tarpaulin --all-features --out html --output-dir coverage/

# Watch for changes and run tests
watch:
	@echo "Watching for changes..."
	cargo watch -x test

# Watch for changes and run checks
watch-check:
	@echo "Watching for changes and running checks..."
	cargo watch -x "fmt" -x "clippy" -x "test"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf coverage/

# Run the API server
run-api:
	@echo "Starting API server..."
	cargo run api

# Run the CLI game
run-cli:
	@echo "Starting CLI game..."
	cargo run

# Build release version
build-release:
	@echo "Building release version..."
	cargo build --release

# Quick development cycle
dev: fmt lint test
	@echo "Development cycle complete!"

# CI/CD pipeline simulation
ci: fmt-check lint test audit
	@echo "CI pipeline complete!"