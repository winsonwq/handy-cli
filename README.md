# handy-cli

AI transcription CLI tool - A standalone transcription engine extracted from [Handy](https://github.com/cjpais/Handy).

[中文文档 (Chinese)](docs/README_zh.md)

---

## Quick Start (For Users)

### Download Pre-built Binary

Download the latest release for your platform from the [GitHub Releases](https://github.com/cjpais/handy-cli/releases) page:

| Platform | Architecture | Download |
| Platform | Architecture | Download |
|:---------|:-------------|:---------|
| macOS | Apple Silicon (M1/M2/M3) | `handy-cli-x.x.x-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `handy-cli-x.x.x-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 | `handy-cli-x.x.x-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 (aarch64) | `handy-cli-x.x.x-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x86_64 | `handy-cli-x.x.x-x86_64-pc-windows-msvc.zip` |
| Windows | ARM64 | `handy-cli-x.x.x-aarch64-pc-windows-msvc.zip` |
### Usage

```bash
# Extract the archive
tar -xzf handy-cli-*.tar.gz  # Linux/macOS
# or unzip handy-cli-*.zip    # Windows

# Start HTTP server
./handy-cli serve --port 8765

# Specify engine and model
./handy-cli serve --engine sensevoice --model sense-voice-int8

# List available models
./handy-cli list-models

# Download a model
./handy-cli download --model sense-voice-int8

# Delete a model
./handy-cli delete --model small

# Check environment
./handy-cli doctor
```

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/models` | List available models |
| GET | `/api/models/downloaded` | List downloaded models |
| POST | `/api/transcribe` | Transcribe audio |
| POST | `/api/transcribe/stream` | Streaming transcription (SSE) |

#### Transcription Request

```json
POST /api/transcribe
{
  "audio": "<base64-encoded-16bit-mono-pcm>",
  "language": "auto",
  "sample_rate": 16000
}
```

#### Response Format

```json
{
  "text": "Transcribed text",
  "language": "zh",
  "duration": 2.5
}
```

### Supported Models

handy-cli supports multiple ASR engines and models:

| Engine | Model | Size | Languages | Features |
|--------|-------|------|-----------|----------|
| **SenseVoice** | sense-voice-int8 | ~152MB | zh, en, ja, ko, yue | Very fast, Chinese optimized |
| **Whisper** | small | ~465MB | 100+ | Multi-language, translation |
| **Whisper** | medium | ~469MB | 100+ | Higher accuracy |
| **Whisper** | turbo | ~1.5GB | 100+ | Best accuracy/speed balance |
| **Whisper** | large | ~1.5GB | 100+ | Highest accuracy |
| **Whisper** | breeze-asr | ~1GB | zh | Taiwanese Mandarin, code-switching |
| **Parakeet** | parakeet-tdt-0.6b-v2 | ~451MB | en | Best for English speakers |
| **Parakeet** | parakeet-tdt-0.6b-v3 | ~456MB | 25 European | European languages |
| **Moonshine** | moonshine-base | ~55MB | en | Very fast, handles accents |
| **Moonshine** | moonshine-tiny-streaming-en | ~31MB | en | Ultra-fast streaming |
| **Moonshine** | moonshine-small-streaming-en | ~99MB | en | Fast streaming |
| **Moonshine** | moonshine-medium-streaming-en | ~192MB | en | High quality streaming |
| **GigaAM** | gigaam-v3-e2e-ctc | ~151MB | ru | Russian speech recognition |
| **Canary** | canary-180m-flash | ~146MB | en, de, es, fr | Fast, supports translation |
| **Canary** | canary-1b-v2 | ~691MB | 25 European | High accuracy, translation |
| **Cohere** | cohere-int8 | ~1.7GB | 100+ | Very high accuracy |

Use `--engine` flag to select engine (e.g., `--engine whisper`, `--engine parakeet`).

---

## Development (For Contributors)

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Build Dependencies

#### macOS

```bash
brew install pkg-config openssl
```

#### Ubuntu / Debian

```bash
sudo apt install libssl-dev pkg-config libasound2-dev
```

#### Windows

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio) with the "Desktop development with C++" workload.

### Build

```bash
cargo build --release
```

The binary will be at `target/release/handy-cli`.

### Run Tests

```bash
cargo test --all-features
```

### Code Quality

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### Project Structure

```
src/
├── main.rs           # Entry point, CLI argument parsing
├── commands/          # CLI command implementations
│   ├── serve.rs      # HTTP server command
│   ├── download.rs   # Model download command
│   ├── list_models.rs
│   ├── delete.rs
│   └── doctor.rs     # Environment check
├── server/           # HTTP API handlers
│   └── handlers/
├── models/           # Model management
├── transcriber/      # ASR engine implementations
├── audio/            # Audio capture
└── vad/              # Voice activity detection
```

### Architecture

```
CLI → HTTP Server (axum) → Audio (cpal) → VAD (vad-rs) → ASR (transcribe-rs)
```

### Configuration

Default config path: `~/.config/handy-cli/config.yaml`

```yaml
server:
  host: "127.0.0.1"
  port: 8765

engine:
  engine_type: sensevoice
  model: sense-voice-int8

vad:
  threshold: 0.5
  min_speech_duration_ms: 250
  min_silence_duration_ms: 500

audio:
  sample_rate: 16000
  input_device: default

models:
  cache_dir: ~/.cache/handy-cli/models
  download_url: https://blob.handy.computer
```

### Commit Guidelines

Use conventional commit format:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `refactor:` - Code refactoring
- `test:` - Adding tests

---

## License

MIT
