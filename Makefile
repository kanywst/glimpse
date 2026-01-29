# Glim Makefile (2026 Standard)

.PHONY: all setup dev format lint test security build help

# Default target
all: help

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%%-20s\033[0m %%s\n", $$1, $$2}'

# --- 🚀 Setup & Dev ---

setup: ## Install required tools (cargo-binstall recommended)
	@echo "Installing modern Rust tools..."
	cargo install cargo-binstall
	cargo binstall cargo-nextest cargo-deny cargo-audit typos-cli --no-confirm

dev: ## Run in development mode with debug logs
	RUST_LOG=debug cargo run

# --- 💅 Quality Assurance ---

check: format lint typos security ## Run full quality check

format: ## Format code (rustfmt)
	cargo fmt

lint: ## Run linter (clippy) - Strict mode
	cargo clippy --all-targets --all-features -- -D warnings

typos: ## Check for spelling errors
	typos

# --- 🛡️ Security & Compliance ---

security: audit deny ## Run all security checks

audit: ## Audit dependencies for vulnerabilities
	cargo audit

deny: ## Check license compliance and banned crates
	cargo deny check

# --- 🧪 Testing ---

test: ## Run tests (using nextest)
	cargo nextest run

coverage: ## Generate coverage report (requires tarpaulin)
	cargo tarpaulin --out Html

# --- 📦 Build ---

build: ## Build release binary
	cargo build --release --locked

demo: build ## Generate demo GIF (requires vhs)
	vhs demo.tape

clean: ## Clean artifacts
	cargo clean
