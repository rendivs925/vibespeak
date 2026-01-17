# Vibespeak - Voice Automation System Makefile
# =================================================
#
# This Makefile provides convenient commands for development, building,
# testing, and deployment of the Vibespeak voice automation system.
#
# Quick Start:
#   make setup    # Initial setup and dependency installation
#   make run      # Build and run the application
#   make dev      # Run in development mode with auto-restart
#

.PHONY: help setup build run dev test clean install docs format lint check-deps web-deps

# Default target
help:
	@echo "Vibespeak - Voice Automation System"
	@echo "===================================="
	@echo ""
	@echo "Available commands:"
	@echo "  setup     - Initial project setup and dependency installation"
	@echo "  build     - Build the application in release mode"
	@echo "  run       - Build and run the application"
	@echo "  dev       - Run in development mode (debug build)"
	@echo "  test      - Run all tests"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install the application locally"
	@echo "  docs      - Generate and serve documentation"
	@echo "  format    - Format code with rustfmt"
	@echo "  lint      - Run clippy linter"
	@echo "  check     - Run all checks (format, lint, test)"
	@echo "  web-deps  - Install web development dependencies"
	@echo "  web-build - Build web assets for production"
	@echo "  config    - Generate default configuration"
	@echo "  docker    - Build Docker image"
	@echo ""
	@echo "Development workflow:"
	@echo "  make setup && make dev"

# Initial setup and dependency installation
setup: check-deps web-deps config
	@echo "Project setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Install Vosk models: https://alphacephei.com/vosk/models"
	@echo "  2. Set up Tailscale: https://tailscale.com/download"
	@echo "  3. Run 'make dev' to start development"

# Check system dependencies
check-deps:
	@echo "Checking system dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "Cargo not found. Install Rust: https://rustup.rs/"; exit 1; }
	@command -v pacman >/dev/null 2>&1 || { echo "Pacman not found (Arch Linux). Some features may not work."; }
	@echo "Rust/Cargo found"

# Install web development dependencies
web-deps:
	@echo "Installing web development dependencies..."
	@command -v node >/dev/null 2>&1 || { echo "Node.js not found. Install from https://nodejs.org/"; }
	@command -v npm >/dev/null 2>&1 || { echo "npm not found."; }
	@if command -v npm >/dev/null 2>&1; then \
		cd web && npm install; \
		echo "Web dependencies installed"; \
	else \
		echo "Skipping web dependencies (npm not available)"; \
	fi

# Build the application in release mode
build:
	@echo "Building Vibespeak (release mode)..."
	cargo build --release
	@echo "Build complete: target/release/vibespeak"

# Build and run the application
run: build
	@echo "Starting Vibespeak..."
	./target/release/vibespeak

# Run in development mode (debug build, auto-restart available)
dev:
	@echo "Starting Vibespeak in development mode..."
	@echo "Press Ctrl+C to stop"
	cargo run

# Run all tests
test:
	@echo "Running tests..."
	cargo test -- --nocapture

# Run tests with coverage (requires cargo-tarpaulin)
test-coverage:
	@echo "Running tests with coverage..."
	cargo tarpaulin --ignore-tests

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean complete"

# Install the application locally
install: build
	@echo "Installing Vibespeak locally..."
	cargo install --path .
	@echo "Installation complete"
	@echo "Run 'vibespeak' from anywhere!"

# Generate and serve documentation
docs:
	@echo "Generating documentation..."
	cargo doc --open --no-deps

# Format code with rustfmt
format:
	@echo "Formatting code..."
	cargo fmt
	@echo "Code formatted"

# Run clippy linter
lint:
	@echo "Running clippy linter..."
	cargo clippy -- -D warnings
	@echo "Lint check complete"

# Run all checks (format, lint, test)
check: format lint test
	@echo "All checks passed!"

# Build web assets for production
web-build:
	@echo "Building web assets..."
	@if [ -d "web" ] && [ -f "web/package.json" ]; then \
		cd web && npm run build; \
		echo "Web assets built"; \
	else \
		echo "Web directory not found or not set up"; \
	fi

# Generate default configuration
config:
	@echo "Generating default configuration..."
	@if [ ! -f "config/system.json" ]; then \
		mkdir -p config; \
		echo '{"commands":[],"workflows":[],"scripts":[],"settings":{"vosk_model_path":"model/vosk-model-small-en-us-0.15","sample_rate":16000,"audio_device":null,"web_server_port":8080,"enable_tts":true,"enable_webrtc":false,"security_level":"trusted"}}' > config/system.json; \
		echo "Default configuration created: config/system.json"; \
	else \
		echo "Configuration already exists"; \
	fi

# Build Docker image
docker:
	@echo "Building Docker image..."
	docker build -t vibespeak .
	@echo "Docker image built: vibespeak"

# Run in Docker container
docker-run:
	@echo "Running Vibespeak in Docker..."
	docker run --rm -p 8080:8080 vibespeak

# Development dependencies (for contributors)
dev-deps:
	@echo "Installing development dependencies..."
	cargo install cargo-tarpaulin  # Test coverage
	cargo install cargo-audit      # Security audit
	cargo install cargo-outdated   # Dependency updates
	@echo "Development dependencies installed"

# Security audit
audit:
	@echo "Running security audit..."
	cargo audit
	@echo "Security audit complete"

# Update dependencies
update-deps:
	@echo "Updating dependencies..."
	cargo update
	@echo "Dependencies updated"

# Check for outdated dependencies
outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated
	@echo "Outdated check complete"

# Performance profiling (requires cargo-flamegraph)
profile:
	@echo "Running performance profiling..."
	cargo flamegraph --dev -- --bench
	@echo "Profiling complete"

# Create release archive
release: build
	@echo "Creating release archive..."
	@VERSION=$$(cargo pkgid | cut -d# -f2 | cut -d: -f2); \
	mkdir -p releases; \
	tar -czf releases/vibespeak-$${VERSION}.tar.gz -C target/release vibespeak; \
	echo "Release archive created: releases/vibespeak-$${VERSION}.tar.gz"

# Quick development loop (build, test, run)
dev-loop:
	@echo "Starting development loop..."
	@while true; do \
		echo "Building and testing..."; \
		make check && make run; \
		echo "Press Enter to rebuild, Ctrl+C to exit"; \
		read -r || exit 1; \
	done

# Show project information
info:
	@echo "Vibespeak Project Information"
	@echo "================================"
	@echo "Version: $$(cargo pkgid | cut -d# -f2 | cut -d: -f2)"
	@echo "Rust Version: $$(rustc --version)"
	@echo "Cargo Version: $$(cargo --version)"
	@echo "Target: $$(rustc -Vv | grep host | cut -d' ' -f2)"
	@echo "Features: Voice recognition, TTS, WebRTC, Plugin system"
	@echo "Web UI: http://localhost:8080"
	@echo "Config: config/system.json"

# Help for specific targets
help-%:
	@echo "Help for target '$*':"
	@grep -A5 -B1 "^$*:" Makefile | grep -v "^--"

# Default target reminder
.DEFAULT_GOAL := help