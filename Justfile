# Justfile - Command Runner (Modern Make)

set shell := ["zsh", "-c"]

default: run

# Run the application in release mode
run:
    cargo run --release

# Run development mode with debug logs
dev:
    RUST_LOG=debug cargo run

# Run tests
test:
    cargo test

# Check code quality
check:
    cargo check
    cargo clippy -- -D warnings
    cargo fmt -- --check

# Build release binary
build:
    cargo build --release

# Format code
fmt:
    cargo fmt

# Clean artifacts
clean:
    cargo clean
