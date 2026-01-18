## Vibespeak Development Guidelines

This document provides guidelines for AI agents and human developers working on the Vibespeak voice automation system.

### Architecture Overview

Vibespeak is a voice-controlled automation platform built with Rust, featuring:

- **Speech Recognition**: Vosk-based offline recognition with grammar optimization
- **Neural TTS**: Piper TTS for natural voice synthesis
- **Browser Automation**: Chromium-based web control
- **Plugin System**: Extensible command and workflow architecture
- **Web Interface**: Configuration and control via browser
- **Script Execution**: Multi-language script support with security controls

### Development Environment

#### Required Tools
- **Rust**: 1.70+ with Cargo
- **Vosk**: Speech recognition models
- **Piper TTS**: Neural voice synthesis
- **System Audio**: ALSA/PulseAudio for audio I/O

#### Project Structure
```
vibespeak/
├── src/
│   ├── domain/           # Business logic & entities
│   ├── application/      # Use cases & application services
│   ├── infrastructure/   # External adapters & persistence
│   ├── presentation/     # Web & CLI interfaces
│   └── shared/           # Common utilities & types
├── web/                  # Frontend assets
├── config/               # Configuration files
├── model/                # Speech recognition models
├── piper/                # TTS binary & voice models
├── docs/                 # Documentation
└── tests/                # Integration tests
```

### Coding Guidelines

#### Principles
- **Clean Code**: Write readable, maintainable code with clear intent
- **DRY (Don't Repeat Yourself)**: Eliminate duplication through abstraction
- **SOLID**: Single responsibility, Open-closed, Liskov substitution, Interface segregation, Dependency inversion
- **YAGNI (You Aren't Gonna Need It)**: Implement only what's necessary
- **KISS (Keep It Simple, Stupid)**: Prefer simple solutions over complex ones
- **Self-Explanatory Code**: Write code that explains itself without excessive comments
- **Balanced Conciseness**: Code should be neither too verbose nor too abbreviated
- **Safety First**: Always write safe code that prevents common errors and vulnerabilities
- **Performance**: Optimize for real-time voice processing
- **Idiomatic Rust**: Follow official Rust conventions and best practices

#### Code Structure
- Limit modules/files to 200-300 lines of code (LOC)
- Exceed this limit only with clear architectural purpose
- Use guard clauses to avoid deeply nested conditions
- Follow existing patterns and conventions in the codebase

### Development Workflow

#### Commands
- **Lint**: `cargo clippy`
- **Typecheck/Build**: `cargo check` / `cargo build`
- **Test**: `cargo test`
- **Format**: `cargo fmt`
- **Run**: `cargo run -- --mode web` or `cargo run -- --mode listen`
- **Setup**: `make setup` (downloads models and dependencies)

#### Testing Strategy
- **Unit Tests**: Individual components and functions
- **Integration Tests**: Full system workflows
- **Audio Tests**: Voice recognition and synthesis
- **Performance Tests**: Real-time processing benchmarks

### Key Components

#### Speech Recognition (VoskAdapter)
- Uses grammar-based recognition for command accuracy
- Supports 16kHz mono audio input
- Automatic fallback from grammar to general recognition

#### Text-to-Speech (TtsAdapter)
- **Exclusive**: Piper neural TTS (high-quality natural voices only)
- Voice options: natural, male, female, fast, slow
- Real-time synthesis with ~30x speed factor
- No fallback TTS engines - Piper required

#### Command Interpreter (FuzzyCommandInterpreter)
- Jaro-Winkler fuzzy matching for natural speech
- Confidence scoring and threshold filtering
- Command categorization and metadata

#### Plugin System
- Command plugins for custom voice commands
- Workflow plugins for multi-step automation
- Integration plugins for external services
- Secure script execution with sandboxing

### Security Considerations

- **Audio Privacy**: All processing local, no cloud transmission
- **Script Sandboxing**: Restricted execution environments
- **Permission System**: Granular access controls
- **Input Validation**: Sanitize all voice commands and parameters
- **Resource Limits**: Prevent resource exhaustion attacks

### Performance Targets

- **Voice Recognition**: <100ms latency for command recognition
- **TTS Synthesis**: <50ms for short phrases
- **Memory Usage**: <200MB base memory footprint
- **CPU Usage**: <10% during idle, <30% during active recognition

### Quality Assurance

#### Pre-commit Checks
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting and style checks
- `cargo test` - Unit and integration tests
- `cargo build --release` - Release build verification

#### Documentation
- Update README.md for user-facing changes
- Update inline documentation for API changes
- Maintain troubleshooting guides
- Document configuration options

### Troubleshooting

#### Common Issues
- **Vosk Model Loading**: Ensure model files are in correct location
- **Piper TTS Missing**: Run `make setup` or download manually
- **Audio Device Issues**: Check permissions and device availability
- **Port Conflicts**: Change web_server_port in config
- **Performance Issues**: Monitor with `htop` and optimize resource usage

#### Debug Mode
```bash
RUST_LOG=debug cargo run -- --mode web
```

### Contributing

1. **Code Standards**: Follow Rust idioms and project conventions
2. **Testing**: Add tests for new functionality
3. **Documentation**: Update docs for user-facing changes
4. **Security**: Review security implications of changes
5. **Performance**: Ensure real-time performance requirements are met
