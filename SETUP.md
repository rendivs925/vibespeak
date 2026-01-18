# Vibespeak Setup Guide

This guide provides detailed instructions for setting up Vibespeak from scratch.

## Prerequisites

### System Requirements
- **OS**: Linux (Arch, Ubuntu, Fedora), macOS 10.15+, Windows 10+
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 2GB free space (including voice models)
- **CPU**: Multi-core processor with AVX support
- **Network**: Internet connection for downloading models

### Required Software
- **Rust**: 1.70+ with Cargo
- **CMake**: 3.13+
- **Git**: For cloning repositories
- **C++ Compiler**: GCC 8+ or Clang 8+

## Quick Setup (Recommended)

```bash
# 1. Clone the repository
git clone https://github.com/rendivs925/vibespeak.git
cd vibespeak

# 2. Run automated setup
make setup

# 3. Start the application
make dev
```

## Detailed Manual Setup

### Step 1: Install System Dependencies

#### Arch Linux
```bash
sudo pacman -S vosk-api alsa-utils cmake fmt spdlog onnxruntime-cpu espeak-ng git
```

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install libvosk-dev alsa-utils cmake libfmt-dev libspdlog-dev onnxruntime libespeak-ng-dev git
```

#### macOS
```bash
brew install cmake fmt spdlog espeak-ng git
# ONNX Runtime: Download from https://github.com/microsoft/onnxruntime/releases
```

#### Windows
- Install MSVC build tools (Visual Studio)
- Install Git for Windows
- Download ONNX Runtime from https://github.com/microsoft/onnxruntime/releases

### Step 2: Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should show 1.70+
```

### Step 3: Build Piper TTS

Piper TTS must be built from source for optimal compatibility:

```bash
# Clone Piper repository
git clone https://github.com/rhasspy/piper.git
cd piper

# Create build directory
mkdir build && cd build

# Configure build
cmake ..

# Build Piper (use multiple cores for faster compilation)
make -j$(nproc)

# Install locally in Vibespeak project
cd ../..  # Return to vibespeak directory
cp piper/build/piper piper/
cp -r piper/build/pi/lib/* piper/lib/
cp -r piper/build/pi/share/* piper/share/

# Clean up (optional)
rm -rf piper/
```

### Step 4: Download Voice Models

```bash
# Create models directory
mkdir -p models

# Download the primary voice model (en_US-amy-medium)
cd models
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/amy/medium/en_US-amy-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/amy/medium/en_US-amy-medium.onnx.json

# Verify downloads
ls -la *.onnx*
```

### Step 5: Download Speech Recognition Model

```bash
# Create model directory for Vosk
mkdir -p model

# Download recommended English model (balanced size/accuracy)
cd model
wget https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip
unzip vosk-model-en-us-0.22-lgraph.zip

# The extracted directory should be: vosk-model-en-us-0.22-lgraph/
ls -la
```

### Step 6: Configure Vibespeak

The default configuration should work, but you can customize it:

```bash
# Check configuration
cat config/system.json

# Optional: Edit configuration
nano config/system.json
```

### Step 7: Build and Test

```bash
# Build the application
cargo build

# Test TTS directly
echo "Hello, this is a test of Piper TTS." | ./piper/piper --model models/en_US-amy-medium.onnx --output_file test.wav

# Test full application
cargo run -- --mode web
```

## Troubleshooting Setup Issues

### Common Build Errors

#### CMake Not Found
```
CMake Error: Could not find CMAKE_ROOT
```
**Solution**: Install CMake:
```bash
sudo pacman -S cmake  # Arch
sudo apt install cmake  # Ubuntu
```

#### Missing ONNX Runtime
```
Could NOT find ONNXRuntime
```
**Solution**: Install ONNX Runtime:
```bash
sudo pacman -S onnxruntime-cpu  # Arch
# Ubuntu: Download from https://github.com/microsoft/onnxruntime/releases
```

#### Missing C++ Compiler
```
No CMAKE_CXX_COMPILER could be found
```
**Solution**: Install build tools:
```bash
sudo pacman -S gcc  # Arch
sudo apt install build-essential  # Ubuntu
```

### Model Download Issues

#### Slow Downloads
If downloads are slow, try alternative mirrors or use a download manager.

#### Corrupted Downloads
```bash
# Check file integrity
ls -la models/
file models/en_US-amy-medium.onnx

# Re-download if corrupted
rm models/en_US-amy-medium.onnx
wget [URL again]
```

### Runtime Issues

#### "Piper TTS not found"
- Ensure Piper was built correctly: `ls -la piper/piper`
- Check permissions: `chmod +x piper/piper`
- Verify library path: `ls -la piper/lib/`

#### "Model file doesn't exist"
- Check model location: `ls -la models/en_US-amy-medium.onnx`
- Update path if needed in TTS adapter

#### Audio Device Issues
```bash
# List audio devices
aplay -l

# Test audio playback
aplay test.wav
```

## Alternative Setup Methods

### Docker Setup
```bash
# Build Docker image
make docker

# Run in container
make docker-run
```

### Development Setup
```bash
# Install additional development tools
cargo install cargo-watch  # For auto-rebuilding
cargo install cargo-tarpaulin  # For test coverage

# Web development (optional)
npm install  # In web/ directory
```

## Post-Setup Configuration

### Audio Configuration
```json
{
  "settings": {
    "audio_device": null,  // Auto-detect
    "sample_rate": 22050,  // Piper default
    "noise_reduction": false
  }
}
```

### Network Configuration
```json
{
  "settings": {
    "web_server_port": 8080,
    "web_server_bind": "127.0.0.1:8080"
  }
}
```

### Voice Configuration
Vibespeak uses only the `en_US-amy-medium` model for optimal quality.

## Verification Checklist

- [ ] Rust 1.70+ installed
- [ ] System dependencies installed
- [ ] Piper TTS built successfully
- [ ] Voice model downloaded
- [ ] Vosk model downloaded
- [ ] Application builds without errors
- [ ] TTS generates audio correctly
- [ ] Web interface loads
- [ ] Voice recognition works

## Getting Help

If you encounter issues:
1. Check the troubleshooting section in README.md
2. Review the logs: `tail -f /tmp/vibespeak.log`
3. Open an issue on GitHub with your setup details