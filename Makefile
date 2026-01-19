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

.PHONY: help setup build run dev dev-web dev-listen dev-frontend frontend-build frontend-deploy test clean install docs format lint check-deps web-deps frontend-deps

# Default target
help:
	@echo "Vibespeak - Voice Automation System"
	@echo "===================================="
	@echo ""
	@echo "Available commands:"
	@echo "  setup     - Initial project setup and dependency installation"
	@echo "  build     - Build the application in release mode"
	@echo "  run       - Build and run the application"
	@echo "  dev       - Run in development mode (interactive mode selection)"
	@echo "  dev-web   - Run development mode with web interface"
	@echo "  dev-fullstack - Run full-stack application (Leptos frontend + backend API)"
	@echo "  dev-listen - Run development mode with voice listening"
	@echo "  test      - Run all tests"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install the application locally"
	@echo "  docs      - Generate and serve documentation"
	@echo "  format    - Format code with rustfmt"
	@echo "  lint      - Run clippy linter"
	@echo "  check     - Run all checks (format, lint, test)"
	@echo "  web-deps     - Install web development dependencies"
	@echo "  web-build    - Build web assets for production"
	@echo "  frontend-deps - Install Leptos frontend dependencies (trunk)"
	@echo "  dev-frontend - Run Leptos frontend in development mode"
	@echo "  frontend-build - Build Leptos frontend for production"
	@echo "  frontend-deploy - Build and deploy frontend to web server"
	@echo "  config       - Generate default configuration"
	@echo "  docker    - Build Docker image"
	@echo ""
	@echo "Development workflow:"
	@echo "  make setup && make dev"
	@echo "  make dev-fullstack               # Full-stack web application"
	@echo "  VIBESPEAK_MODE=listen make dev  # Auto-start voice listening"
	@echo "  VIBESPEAK_MODE=web make dev     # Auto-start web interface"

# Initial setup and dependency installation
setup: check-deps web-deps frontend-deps config
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

# Run in development mode (debug build)
dev:
	@echo "Starting Vibespeak in development mode..."
	@if [ "$$VIBESPEAK_MODE" = "listen" ]; then \
		echo "🎤 Voice listening mode (set by VIBESPEAK_MODE=listen)"; \
		cargo run -- --mode listen; \
	elif [ "$$VIBESPEAK_MODE" = "web" ]; then \
		echo "🌐 Web interface mode (set by VIBESPEAK_MODE=web)"; \
		cargo run -- --mode web; \
	else \
		echo "Choose mode:"; \
		echo "  1) Web interface (default)"; \
		echo "  2) Voice listening"; \
		echo -n "Enter choice [1-2]: "; \
		read -r choice; \
		case $$choice in \
			2) echo "Starting voice listening mode..."; cargo run -- --mode listen ;; \
			*) echo "Starting web interface mode..."; cargo run -- --mode web ;; \
		esac; \
	fi

# Run in development mode with web interface
dev-web:
	@echo "Starting Vibespeak web interface (development mode)..."
	@echo "Web UI: http://localhost:8080"
	@echo "Press Ctrl+C to stop"
	cargo run -- --mode web

# Run full-stack application (Leptos frontend + backend API)
dev-fullstack:
	@echo "Starting Vibespeak full-stack application..."
	@echo "Web UI: http://localhost:8080"
	@echo "API endpoints: http://localhost:8080/api/*"
	@echo "Press Ctrl+C to stop"
	cargo run -- --mode web

# Run in development mode with voice listening
dev-listen:
	@echo "Starting Vibespeak voice listening (development mode)..."
	@echo "🎤 Voice listening active - speak commands!"
	@echo "Press Ctrl+C to stop"
	cargo run -- --mode listen

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

 # Install Leptos frontend dependencies (trunk)
 frontend-deps:
	@echo "Installing Leptos frontend dependencies..."
	@command -v trunk >/dev/null 2>&1 || { echo "Installing trunk (WASM bundler)..."; cargo install trunk; }
	@echo "Trunk installed and ready"

 # Run Leptos frontend in development mode
 dev-frontend:
	@echo "Starting Leptos frontend in development mode..."
	@echo "Frontend will be available at http://localhost:3000"
	@echo "Press Ctrl+C to stop"
	@cd frontend && trunk serve --port 3000

 # Build Leptos frontend for production
 frontend-build:
	@echo "Building Leptos frontend for production..."
	@cd frontend && trunk build --release
	@echo "Frontend built successfully in frontend/dist/"

 # Build and deploy frontend to web server
 frontend-deploy: frontend-build
	@echo "Deploying frontend to web server..."
	@mkdir -p web/dist
	@cp -r frontend/dist/* web/dist/
	@echo "Frontend deployed to web/dist/"
	@echo "Run 'make dev-web' to serve the application"

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