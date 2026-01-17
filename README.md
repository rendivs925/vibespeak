# Vibespeak - Voice Automation System

A comprehensive voice-controlled automation platform that transforms your computer into an intelligent voice assistant. Control applications, execute complex workflows, and manage your system entirely through voice commands.

## Features

- **Advanced Voice Recognition** - Powered by Vosk for offline, privacy-focused speech recognition
- **Text-to-Speech** - Natural voice synthesis with multiple voice options
- **Browser Automation** - Control web browsers programmatically
- **Extensible Plugin System** - Add custom commands, workflows, and integrations
- **Web-Based Configuration** - Intuitive browser interface for all settings
- **Remote Access** - Tailscale integration for global access
- **Real-time Processing** - Low-latency voice command execution
- **Privacy-First** - All processing local, no cloud dependencies

## Table of Contents

- [Quick Start](#quick-start)
- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/rendivs925/vibespeak.git
cd vibespeak

# Run setup (installs dependencies and creates config)
make setup

# Start development server
make dev
```

Open http://localhost:8080 in your browser to configure and use Vibespeak.

## System Requirements

### Minimum Requirements

- **OS**: Linux (Arch, Ubuntu, Fedora), macOS 10.15+, Windows 10+
- **RAM**: 2GB
- **Disk**: 500MB free space
- **Microphone**: Any standard audio input device

### Recommended Requirements

- **OS**: Linux (Arch/Ubuntu)
- **RAM**: 4GB+
- **Disk**: 2GB free space (including voice models)
- **CPU**: Multi-core processor with AVX support
- **Microphone**: High-quality USB microphone

### Dependencies

#### Required System Packages

**Arch Linux:**

```bash
sudo pacman -S vosk-api speech-dispatcher alsa-utils
```

**Ubuntu/Debian:**

```bash
sudo apt install libvosk-dev speech-dispatcher alsa-utils
```

**macOS (using Homebrew):**

```bash
brew install vosk speech-dispatcher
```

**Windows:**

- Download Vosk from: https://alphacephei.com/vosk/models
- Install from releases page

#### Rust Toolchain

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be 1.70+
cargo --version  # Should be 1.70+
```

#### Optional: Web Development Tools

```bash
# Node.js for web interface development
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Or using nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

## Installation

### Option 1: Automated Setup (Recommended)

```bash
# Clone repository
git clone https://github.com/yourusername/vibespeak.git
cd vibespeak

# Run automated setup
make setup
```

This will:

- Check system dependencies
- Install web development tools (if available)
- Generate default configuration
- Download required assets

### Option 2: Manual Setup

```bash
# Clone repository
git clone https://github.com/yourusername/vibespeak.git
cd vibespeak

# Install Rust dependencies
cargo build

# Install web dependencies (optional)
make web-deps

# Generate configuration
make config
```

### Option 3: Docker Setup

```bash
# Build Docker image
make docker

# Run in container
make docker-run
```

## Voice Model Setup

Vibespeak requires Vosk language models for speech recognition:

### Download Models

```bash
# Create models directory
mkdir -p model

# Download English model (small, recommended for development)
cd model
wget https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip
unzip vosk-model-small-en-us-0.15.zip
mv vosk-model-small-en-us-0.15/* .
rmdir vosk-model-small-en-us-0.15

# Alternative: Download larger model for better accuracy
wget https://alphacephei.com/vosk/models/vosk-model-en-us-0.22.zip
unzip vosk-model-en-us-0.22.zip
```

### Available Models

| Model                          | Size  | Accuracy  | Use Case                          |
| ------------------------------ | ----- | --------- | --------------------------------- |
| `vosk-model-small-en-us-0.15`  | 40MB  | Good      | Development, resource-constrained |
| `vosk-model-en-us-0.22`        | 1.8GB | Excellent | Production, high accuracy         |
| `vosk-model-en-us-0.22-lgraph` | 120MB | Very Good | Production, balanced              |

## Configuration

### Basic Configuration

The main configuration file is `config/system.json`:

```json
{
  "commands": [],
  "workflows": [],
  "scripts": [],
  "settings": {
    "vosk_model_path": "model/vosk-model-small-en-us-0.15",
    "sample_rate": 16000,
    "audio_device": null,
    "web_server_port": 8080,
    "enable_tts": true,
    "enable_webrtc": false,
    "security_level": "trusted"
  }
}
```

### Web-Based Configuration

1. Start Vibespeak: `make dev`
2. Open http://localhost:8080
3. Configure voice commands, workflows, and scripts through the web interface

### Advanced Configuration

#### Audio Settings

```json
{
  "settings": {
    "sample_rate": 44100,
    "audio_device": "hw:1,0",
    "noise_reduction": true,
    "echo_cancellation": true
  }
}
```

#### Security Settings

```json
{
  "settings": {
    "security_level": "trusted",
    "allowed_paths": ["/home/user", "/tmp"],
    "blocked_commands": ["rm -rf", "sudo"]
  }
}
```

#### Network Settings

```json
{
  "settings": {
    "web_server_port": 8080,
    "tailscale_enabled": true,
    "tailscale_interface": "tailscale0",
    "cors_origins": ["http://localhost:8080"]
  }
}
```

## Usage

### Starting Vibespeak

```bash
# Development mode
make dev

# Production mode
make run

# Background service
make build
./target/release/vibespeak &
```

### Basic Voice Commands

Default commands are configured through the web interface:

- **"open browser"** - Opens default web browser
- **"new terminal"** - Opens new terminal window
- **"take screenshot"** - Captures screen image
- **"increase volume"** - Audio volume up

### Creating Custom Commands

1. Open http://localhost:8080
2. Go to "Voice Commands" tab
3. Click "Add Command"
4. Enter voice phrase and corresponding action
5. Test recognition and save

### Workflows

Create multi-step automation sequences:

```json
{
  "name": "Code Review",
  "trigger": "start code review",
  "steps": [
    {
      "type": "execute",
      "command": "git fetch origin main"
    },
    {
      "type": "script",
      "language": "bash",
      "content": "cargo check"
    },
    {
      "type": "user_prompt",
      "message": "Code review complete. Any issues?"
    }
  ]
}
```

### Scripts

Execute custom scripts via voice:

**Bash Script Example:**

```bash
# Save as deploy.sh
#!/bin/bash
echo "Starting deployment..."
npm run build
docker build -t myapp .
docker run -d myapp
```

**Voice Command:** "deploy application"

### Browser Automation

Control web browsers programmatically:

```json
{
  "action": "browser_navigate",
  "url": "https://github.com/myrepo"
}
```

## Development

### Development Workflow

```bash
# Initial setup
make setup

# Start development with auto-reload
make dev

# Run tests
make test

# Code quality checks
make check

# Format code
make format
```

### Project Structure

```
vibespeak/
├── src/
│   ├── domain/           # Business logic
│   ├── application/      # Use cases & services
│   ├── infrastructure/   # External interfaces
│   ├── presentation/     # Web & CLI interfaces
│   └── shared/           # Common utilities
├── web/                  # Web interface assets
├── config/               # Configuration files
├── model/                # Voice recognition models
├── docs/                 # Documentation
├── tests/                # Integration tests
└── Makefile             # Build automation
```

### Adding New Features

#### 1. Domain Logic

Add business rules to `src/domain/`

#### 2. Application Services

Implement use cases in `src/application/`

#### 3. Infrastructure

Add external integrations in `src/infrastructure/`

#### 4. Web Interface

Update `web/index.html` and API endpoints

#### 5. Plugins

Implement in `src/domain/services/plugin.rs`

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With coverage (requires tarpaulin)
make test-coverage
```

### Building for Production

```bash
# Optimized release build
make build

# Create release archive
make release

# Docker deployment
make docker
```

## Remote Access Setup

### Tailscale Configuration

1. Install Tailscale: https://tailscale.com/download
2. Authenticate: `sudo tailscale up`
3. Configure Vibespeak to bind to Tailscale interface

```json
{
  "settings": {
    "tailscale_enabled": true,
    "web_server_bind": "100.64.0.1:8080"
  }
}
```

### Alternative Remote Access

#### SSH Tunneling

```bash
# Local access
ssh -L 8080:localhost:8080 user@remote-server

# Then access http://localhost:8080
```

#### VPN Setup

```bash
# WireGuard or OpenVPN configuration
# Bind Vibespeak to VPN interface
```

## Troubleshooting

### Common Issues

#### 1. "Vosk model not found"

```
Error: Failed to load Vosk model
```

**Solution:**

```bash
# Verify model exists
ls -la model/

# Download correct model
cd model
wget https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip
unzip vosk-model-small-en-us-0.15.zip
```

#### 2. "Audio device not found"

```
Error: No audio input device available
```

**Solution:**

```bash
# List available devices
arecord -l

# Configure specific device in config.json
{
  "settings": {
    "audio_device": "hw:1,0"
  }
}
```

#### 3. "Port already in use"

```
Error: Address already in use (os error 98)
```

**Solution:**

```bash
# Kill process using port 8080
sudo lsof -ti:8080 | xargs kill -9

# Or change port in config
{
  "settings": {
    "web_server_port": 8081
  }
}
```

#### 4. "Permission denied"

```
Error: Permission denied (os error 13)
```

**Solution:**

```bash
# Run with appropriate permissions
sudo ./target/release/vibespeak

# Or configure user permissions for audio devices
sudo usermod -a -G audio $USER
```

#### 5. Web interface not loading

**Check:**

```bash
# Verify server is running
curl http://localhost:8080/api/config

# Check firewall settings
sudo ufw status
sudo ufw allow 8080
```

### Performance Issues

#### High CPU Usage

- Reduce model size (use smaller Vosk model)
- Disable TTS if not needed
- Lower audio sample rate

#### High Memory Usage

- Use smaller voice models
- Disable unused plugins
- Monitor with `htop` or `top`

### Audio Quality Issues

#### Poor Recognition Accuracy

- Use larger Vosk model
- Improve microphone quality
- Reduce background noise
- Speak clearly and closer to microphone

#### Audio Stuttering

- Check CPU usage during recognition
- Reduce concurrent processes
- Use wired microphone instead of Bluetooth

### Logs and Debugging

```bash
# Enable debug logging
RUST_LOG=debug make dev

# View logs
tail -f /tmp/vibespeak.log

# Verbose build
cargo build --verbose
```

## Architecture

### Clean Architecture Overview

```
┌─────────────────────────────────────┐
│         Presentation Layer          │
│  - Web Interface                    │
│  - REST API                         │
│  - WebSocket                        │
└─────────────────────────────────────┘
                    │
┌─────────────────────────────────────┐
│       Application Layer             │
│  - Use Cases                        │
│  - Application Services             │
│  - DTOs                             │
└─────────────────────────────────────┘
                    │
┌─────────────────────────────────────┐
│         Domain Layer                │
│  - Entities                         │
│  - Value Objects                    │
│  - Domain Services                  │
│  - Business Rules                   │
└─────────────────────────────────────┘
                    │
┌─────────────────────────────────────┐
│     Infrastructure Layer            │
│  - Vosk Adapter                     │
│  - TTS Adapter                      │
│  - File System                      │
│  - WebRTC                           │
└─────────────────────────────────────┘
```

### Plugin System

Extensible architecture supporting:

- **Command Plugins**: Custom voice commands
- **Workflow Plugins**: Complex automation sequences
- **Integration Plugins**: External service connections
- **Script Plugins**: Custom script execution engines

### Security Model

- **Sandboxed Execution**: Restricted script environments
- **Trusted Execution**: Full system access for approved scripts
- **Isolated Execution**: Container-based execution for untrusted code
- **Permission System**: Granular access controls

## API Reference

### REST Endpoints

```
GET  /api/config          # Get current configuration
POST /api/config          # Update configuration
POST /api/voice/test      # Test voice recognition
GET  /api/status          # System status
GET  /api/logs            # System logs
```

### WebSocket Events

```javascript
// Voice recognition
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === "recognition_result") {
    console.log("Recognized:", data.text);
  }
};
```

## Contributing

### Development Setup

```bash
# Fork and clone
git clone https://github.com/yourusername/vibespeak.git
cd vibespeak

# Set up development environment
make setup
make dev-deps

# Create feature branch
git checkout -b feature/your-feature
```

### Code Standards

- **Rust**: Follow official Rust guidelines
- **Documentation**: Document all public APIs
- **Testing**: 80%+ code coverage required
- **Security**: No unsafe code without security review

### Pull Request Process

1. Create feature branch
2. Write tests for new functionality
3. Update documentation
4. Run `make check` to ensure quality
5. Submit PR with detailed description

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: https://github.com/yourusername/vibespeak/issues
- **Discussions**: https://github.com/yourusername/vibespeak/discussions
- **Documentation**: https://vibespeak.dev/docs

## Acknowledgments

- **Vosk**: Open-source speech recognition
- **TTS-RS**: Rust text-to-speech library
- **Tokio**: Async runtime
- **Warp**: Web framework
- **Tailscale**: Secure remote access

---

**Built with love for privacy-focused voice automation**

