# Vibespeak - Voice Automation System

A comprehensive voice-controlled automation platform that transforms your computer into an intelligent voice assistant. Control applications, execute complex workflows, and manage your system entirely through voice commands.

## Features

- **Grammar-Based Voice Recognition** - Advanced Vosk speech recognition with command-specific grammar for superior accuracy
- **Neural Text-to-Speech** - Powered by Piper TTS for incredibly natural, human-like voice synthesis
- **Browser Automation** - Control web browsers programmatically with Chromium
- **Extensible Plugin System** - Add custom commands, workflows, and integrations
- **Web-Based Configuration** - Intuitive browser interface for all settings
- **Remote Access** - Tailscale integration for global access
- **Real-time Processing** - Low-latency voice command execution
- **Privacy-First** - All processing local, no cloud dependencies
- **Script Execution** - Secure multi-language script execution engine

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
sudo pacman -S vosk-api alsa-utils
# Piper TTS is included in the automated setup
```

**Ubuntu/Debian:**

```bash
sudo apt install libvosk-dev alsa-utils
# Piper TTS is included in the automated setup
```

**macOS (using Homebrew):**

```bash
brew install vosk
# Piper TTS is included in the automated setup
```

**Windows:**

- Download Vosk from: https://alphacephei.com/vosk/models
- Download Piper TTS from: https://github.com/rhasspy/piper/releases
- Install both according to their respective instructions

**Note:** Vibespeak now requires Piper TTS exclusively for natural voice synthesis. Espeak/espeak-ng are no longer supported.

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

Vibespeak requires both Vosk language models for speech recognition and Piper voice models for natural speech synthesis:

### Speech Recognition Models (Vosk)

```bash
# Create models directory
mkdir -p model

# Download English model (balanced size and accuracy)
cd model
wget https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip
unzip vosk-model-en-us-0.22-lgraph.zip
# Files will be extracted to vosk-model-en-us-0.22-lgraph/
```

### Text-to-Speech Models (Piper)

The automated setup (`make setup`) will download Piper TTS and voice models automatically. For manual setup:

```bash
# Download Piper TTS binary
wget https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz
tar -xzf piper_amd64.tar.gz

# Download high-quality voice models
# Recommended: Natural female voice
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json

# Optional: Male voice
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/ryan/medium/en_US-ryan-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/ryan/medium/en_US-ryan-medium.onnx.json
```

### Available Models

#### Speech Recognition Models

| Model                          | Size  | Accuracy  | Use Case                          |
| ------------------------------ | ----- | --------- | --------------------------------- |
| `vosk-model-small-en-us-0.15`  | 40MB  | Good      | Development, resource-constrained |
| `vosk-model-en-us-0.22-lgraph` | 120MB | Very Good | Production, balanced              |
| `vosk-model-en-us-0.22`        | 1.8GB | Excellent | Production, high accuracy         |

#### Text-to-Speech Voice Models

| Voice Model              | Quality | Size | Description                     |
| ------------------------ | ------- | ---- | ------------------------------- |
| `en_US-lessac-medium`    | ⭐⭐⭐⭐⭐ | 60MB | Natural female, highly recommended |
| `en_US-ryan-medium`      | ⭐⭐⭐⭐⭐ | 60MB | Natural male voice              |
| `en_GB-alan-medium`      | ⭐⭐⭐⭐⭐ | 60MB | British male voice              |
| `en_US-amy-medium`       | ⭐⭐⭐⭐  | 60MB | Additional female voice         |

**Note**: Piper voices are neural network-based and provide significantly more natural speech than traditional TTS engines.

## Configuration

### Basic Configuration

The main configuration file is `config/system.json`:

```json
{
  "commands": [],
  "workflows": [],
  "scripts": [],
  "settings": {
    "vosk_model_path": "model/vosk-model-en-us-0.22-lgraph",
    "sample_rate": 16000,
    "audio_device": null,
    "web_server_port": 8080,
    "enable_tts": true,
    "enable_webrtc": false,
    "security_level": "trusted",
    "tailscale_enabled": false
  }
}
```

### Voice Configuration

Vibespeak now uses Piper TTS with multiple high-quality voice options:

```json
{
  "settings": {
    "tts_engine": "piper",
    "tts_voice": "natural",
    "tts_pitch": 1.0,
    "tts_rate": 0.95,
    "tts_volume": 0.8
  }
}
```

**Available Voices:**
- `natural` - High-quality female voice (recommended)
- `male` - Natural male voice
- `female` - Alternative female voice
- `fast` - Faster speech rate
- `slow` - Slower, clearer speech

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

Vibespeak comes with an extensive set of pre-configured voice commands for common tasks:

#### System Control
- **"open browser"** - Opens default web browser
- **"open terminal"** - Opens new terminal window
- **"show menu"** - Opens application menu
- **"lock screen"** - Locks the screen

#### Window Management (i3/Sway)
- **"desk one/two/three/four"** - Switch workspaces
- **"focus left/right/up/down"** - Move focus between windows
- **"close window"** - Close active window
- **"full screen"** - Toggle fullscreen mode

#### Tmux Control
- **"split pane"** - Create vertical split
- **"split horizontal"** - Create horizontal split
- **"pane one/two/three..."** - Switch between panes
- **"new window"** - Create new tmux window

#### Development
- **"start dev"** - Start development server
- **"check code"** - Run code linting/checking
- **"save file"** - Save current file
- **"open editor"** - Open text editor

#### File Operations
- **"list files"** - Show directory contents
- **"open files"** - Open file manager
- **"clear screen"** - Clear terminal

#### Voice Features
- **"type"** - Enter voice typing mode (dictation)
- **Win+T** - Toggle typing mode
- **Esc** - Exit typing mode

All commands support fuzzy matching for natural speech recognition.

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

# Development modes
make dev          # Interactive mode selection (web/listen)
make dev-listen   # Start voice listening mode directly
make dev-web      # Start web interface mode directly

# Alternative: Use environment variables
VIBESPEAK_MODE=listen make dev  # Auto-start voice listening
VIBESPEAK_MODE=web make dev     # Auto-start web interface

# Testing & quality
make test         # Run all tests
make check        # Format, lint, and test
make format       # Format code only
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

# Download and extract the recommended model
cd model
wget https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip
unzip vosk-model-en-us-0.22-lgraph.zip
# Update config to point to the extracted directory
```

#### 2. "Piper TTS not found"

```
Error: Piper TTS not found
```

**Solution:**

Piper TTS is required for Vibespeak to function. The automated setup (`make setup`) should install it automatically, but for manual installation:

```bash
# Download and extract Piper TTS
wget https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_amd64.tar.gz
tar -xzf piper_amd64.tar.gz

# Download high-quality voice models
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json

# Make piper executable and available
chmod +x piper/piper

# Test Piper
echo "Hello world" | ./piper/piper --model en_US-lessac-medium.onnx --output_file test.wav
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

- **Vosk**: Open-source offline speech recognition
- **Piper TTS**: High-quality neural text-to-speech synthesis
- **Tokio**: Async runtime for Rust
- **Warp**: Fast web framework
- **Tailscale**: Secure remote access networking
- **Chromium**: Browser automation engine

---

**Built with love for privacy-focused voice automation**

