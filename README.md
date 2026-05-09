# handy-cli

AI transcription CLI tool - A standalone transcription engine extracted from [Handy](https://github.com/cjpais/Handy).

## Features

- 🌐 **HTTP API** - REST API for transcription
- 🎤 **Audio Capture** - Cross-platform microphone recording (cpal)
- 🎯 **VAD** - Voice Activity Detection (vad-rs)
- 🤖 **Multi-Engine** - Whisper.cpp / SenseVoice ONNX (transcribe-rs)
- 📦 **Model Management** - Download and manage ASR models

## Build Dependencies

### macOS

```bash
brew install pkg-config openssl
```

### Ubuntu / Debian

```bash
sudo apt install libssl-dev pkg-config
```

### Windows (with MSYS2)

```bash
pacman -S mingw-w64-x86_64-openssl mingw-w64-x86_64-pkg-config
```

## Build

```bash
cargo build --release
```

## Usage

```bash
# Start HTTP server
./target/release/handy-cli serve --port 8765

# Specify engine and model
./target/release/handy-cli serve --engine sensevoice --model sense-voice-int8

# List available models
./target/release/handy-cli list-models

# Download model
./target/release/handy-cli download --model sense-voice-int8

# Check environment
./target/release/handy-cli doctor
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/models` | List available models |
| GET | `/api/models/downloaded` | List downloaded models |
| POST | `/api/transcribe` | Transcribe audio |

### Transcription Request

```json
POST /api/transcribe
{
  "audio": "<base64-encoded-16bit-mono-pcm>",
  "language": "auto",
  "sample_rate": 16000
}
```

### Response Format

```json
{
  "text": "Transcribed text",
  "language": "zh",
  "duration": 2.5
}
```

## Engines

| Engine | Model | Size | Features |
|--------|-------|------|----------|
| SenseVoice | sense-voice-int8 | ~230MB | Chinese optimized, built-in punctuation |
| Whisper | tiny/base/small/medium/large | ~75MB/~150MB/~465MB/~1.5GB/~3GB | Multi-language support |

### Whisper Models

Whisper models need to be downloaded from HuggingFace, file naming convention: `ggml-{model}.bin`:

```bash
# Download tiny model (~75MB)
curl -L -o ~/.cache/handy-cli/models/ggml-tiny.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"

# Download base model (~150MB)
curl -L -o ~/.cache/handy-cli/models/ggml-base.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"

# Download small model (~465MB)
curl -L -o ~/.cache/handy-cli/models/ggml-small.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
```

## Architecture

```
CLI → HTTP Server (axum) → Audio (cpal) → VAD (vad-rs) → ASR (transcribe-rs)
```

## Configuration

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

## License

MIT
