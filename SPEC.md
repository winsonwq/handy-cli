# handy-cli — AI Transcription Core Tool

## Overview

handy-cli is a standalone AI transcription CLI tool, extracting core functionality from [Handy](https://github.com/cjpais/Handy) (Tauri application).

**Goal:** Provide a cross-platform (macOS/Linux/Windows) command-line tool that starts an HTTP server for speech-to-text functionality, without requiring any runtime installation.

## Core Principles

1. **Zero-dependency runtime** — Packaged as a single executable, users download and run directly
2. **Preserve Handy advantages** — Multi-engine support, model management, VAD, etc.
3. **CLI-configurable** — Adjust behavior via command-line arguments or config file
4. **No UI** — Pure CLI + HTTP API, focused on backend capabilities

## Relationship with Handy

| Component | Handy (Tauri) | Handy-cli (This Project) |
|-----------|---------------|--------------------------|
| Frontend | React UI | ❌ Removed |
| Hotkeys | rdev/enigo | ❌ Removed |
| Text injection | CGEvent/SendInput | ❌ Removed |
| Menu bar/Tray | Tauri API | ❌ Removed |
| Audio capture | cpal | ✅ Preserved |
| VAD | vad-rs | ✅ Preserved |
| ASR Engine | transcribe-rs | ✅ Preserved |
| Model management | Existing logic | ✅ Preserved |
| HTTP service | ❌ | ✅ New |

## Feature List

### 1. HTTP API Service

Start a local HTTP service providing REST + SSE interfaces.

**Endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/models` | List available models |
| GET | `/api/models/downloaded` | List downloaded models |
| POST | `/api/models/download` | Download model |
| POST | `/api/transcribe` | Transcribe audio (JSON body) |
| POST | `/api/transcribe/stream` | Streaming transcription (SSE) |
| POST | `/api/audio/start` | Start audio recording |
| POST | `/api/audio/stop` | Stop audio recording |
| GET | `/api/audio/status` | Recording status |

**Transcription Response Format:**
```json
{
  "text": "Transcribed text content",
  "language": "zh",
  "duration": 3.5,
  "language_probability": 0.99,
  "segments": [
    {
      "text": "First segment",
      "start": 0.0,
      "end": 1.5
    }
  ]
}
```

**SSE Events:**
```
event: speech_start
data: {"timestamp": 1234567890}

event: speech_end
data: {"timestamp": 1234567890, "duration": 3.5}

event: transcript
data: {"text": "Transcribing...", "partial": true}

event: transcript
data: {"text": "Final transcription result.", "partial": false}
```

### 2. Audio Capture

- Cross-platform microphone capture using `cpal`
- Configurable sample rate (default 16kHz)
- Input device selection
- Audio format: float32 PCM

### 3. VAD (Voice Activity Detection)

- Voice activity detection using `vad-rs`
- Configurable threshold, silence duration, etc.
- Events: `speech_start`, `speech_end`

### 4. ASR Engines

Multiple engine support via `transcribe-rs`:

| Engine | Models | Size | Format | Features |
|--------|--------|------|--------|----------|
| Whisper.cpp | tiny/base/small/medium/large | ~75MB/~150MB/~465MB/~1.5GB/~3GB | Single file `ggml-{model}.bin` | Multi-language |
| SenseVoice | sense-voice-int8 | ~230MB | Directory (contains `model.int8.onnx` and `tokens.txt`) | Chinese optimized, built-in punctuation |
| Parakeet | TDTv2, TDTv3 | ~300MB | Directory (`.onnx` files) | Fast English/European languages |
| Moonshine | Base, Tiny, Small, Medium | ~1GB/~250MB/~500MB/~1.2GB | Directory (`.onnx` files) | Ultra-fast streaming, English only |
| GigaAM | v3 | ~400MB | Directory (`.onnx` files) | Russian speech recognition |
| Canary | 180M Flash, 1B v2 | ~200MB/~2GB | Directory (`.onnx` files) | Supports translation via target_language |
| Cohere | int8 | ~1GB | Directory (`.onnx` files) | Large multilingual model |

**Model Downloads:**
- Whisper models: Download from HuggingFace (`ggerganov/whisper.cpp`)
- SenseVoice models: Download from Handy official CDN (`blob.handy.computer`)

**Whisper Model Download Commands:**
```bash
curl -L -o ~/.cache/handy-cli/models/ggml-tiny.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
```

### 5. CLI Commands

```bash
# Start HTTP server
handy-cli serve [OPTIONS]

# Options:
#   --port <PORT>        HTTP port (default 8765)
#   --host <HOST>        Listen address (default 127.0.0.1)
#   --engine <ENGINE>    whisper / sensevoice (default sensevoice)
#   --model <MODEL>      Model name (default sense-voice-int8)
#   --vad-threshold      VAD threshold 0.0-1.0 (default 0.5)
#   --language <LANG>   Language code (default auto)

# List available models
handy-cli list-models [--engine <ENGINE>]

# Download model
handy-cli download --model <MODEL>

# Delete model
handy-cli delete --model <MODEL>

# Check environment
handy-cli doctor

# Version info
handy-cli --version
```

### 6. Configuration File

YAML configuration file support (`~/.handy-cli/config.yaml`):

```yaml
server:
  host: "127.0.0.1"
  port: 8765

engine:
  type: sensevoice  # whisper / sensevoice
  model: sense-voice-int8

vad:
  threshold: 0.5
  min_speech_duration_ms: 250
  min_silence_duration_ms: 500

audio:
  sample_rate: 16000
  input_device: default  # or device name/index

models:
  cache_dir: ~/.handy-cli/models
  download_url: https://blob.handy.computer
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    CLI (clap)                           │
│   serve / list-models / download / delete / doctor      │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                  HTTP Server (axum)                      │
│   /api/* endpoints + SSE                                │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│               Audio Manager (cpal)                      │
│   capture / device selection                            │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                    VAD (vad-rs)                         │
│   speech detection                                     │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│              Transcriber (transcribe-rs)                │
│   whisper.cpp / SenseVoice ONNX                         │
└─────────────────────────────────────────────────────────┘
```

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.75+ |
| Web Framework | axum | 0.7+ |
| Audio Capture | cpal | 0.16+ |
| VAD | vad-rs | git |
| ASR | transcribe-rs | 0.3+ |
| CLI | clap | 4.x |
| Logging | tracing | 0.1+ |
| Config | serde_yaml | 0.9+ |

## Project Structure

```
handy-cli/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── serve.rs        # serve command
│   │   ├── list_models.rs  # list-models command
│   │   ├── download.rs     # download command
│   │   ├── delete.rs       # delete command
│   │   └── doctor.rs       # doctor command
│   ├── server/
│   │   ├── mod.rs
│   │   ├── api.rs          # REST endpoints
│   │   ├── sse.rs          # SSE endpoints
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── health.rs
│   │       ├── transcribe.rs
│   │       └── models.rs
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── capture.rs      # cpal capture
│   │   └── device.rs       # device management
│   ├── vad/
│   │   └── mod.rs          # vad-rs wrapper
│   ├── transcriber/
│   │   ├── mod.rs
│   │   ├── whisper.rs      # whisper.cpp (ggml-*.bin)
│   │   └── sensevoice.rs   # SenseVoice (ONNX)
│   ├── models/
│   │   ├── mod.rs
│   │   ├── manager.rs      # model management/download
│   │   └── registry.rs     # model registry
│   ├── config/
│   │   └── mod.rs          # config file
│   └── error.rs            # error types
├── config.yaml.example     # config file example
├── build.rs                # Tauri build (reserved)
└── README.md
```

## Build Outputs

| Platform | Format | Filename |
|----------|--------|----------|
| macOS Apple Silicon | tar.gz | handy-macos-aarch64.tar.gz |
| macOS Intel | tar.gz | handy-macos-x86_64.tar.gz |
| Linux | tar.gz | handy-linux-x86_64.tar.gz |
| Windows | zip | handy-windows-x86_64.zip |

## Feature Comparison with Handy

### ✅ Migrated Features

| Category | Feature | Status |
|----------|---------|--------|
| **Core** | Audio Capture (cpal) | ✅ |
| **Core** | VAD (vad-rs) | ✅ |
| **Core** | Whisper Engine | ✅ |
| **Core** | SenseVoice Engine | ✅ |
| **Core** | HTTP Server (REST + SSE) | ✅ |
| **Core** | Model Management | ✅ |
| **CLI** | serve, list-models, download, delete, doctor | ✅ |

### ❌ Not Migrated Features

Features from Handy that are not yet implemented in handy-cli:

#### ASR Engines (High Priority)

| Engine | Models | Description | Status |
|--------|--------|-------------|--------|
| Parakeet | V2, V3 | Fast English/European language models | ✅ Migrated |
| Moonshine | Base, Tiny, Small, Medium | Ultra-fast streaming English models | ✅ Migrated |
| GigaAM | v3 | Russian speech recognition | ✅ Migrated |
| Canary | 180M Flash, 1B v2 | Supports translation | ✅ Migrated |
| Cohere | int8 | Large multilingual model | ✅ Migrated |
| Breeze | ASR | Taiwanese Mandarin optimized | ⏳ Pending |

#### Input & Output

| Feature | Description | Migratable |
|---------|-------------|------------|
| **Custom Words** | Word correction dictionary | ✅ Yes |
| **Audio Feedback** | Sound notifications for recording | ⚠️ Limited |
| **Clipboard Handling** | Copy transcription to clipboard | ✅ Yes |

#### Audio Settings

| Feature | Description | Migratable |
|---------|-------------|------------|
| **Extra Recording Buffer** | Additional audio buffer (ms) | ✅ Yes |
| **Mute While Recording** | Mute output during recording | ⚠️ Limited |
| **Output Device Selection** | Choose audio output device | ❌ No |

#### Post-Processing & Enhancement

| Feature | Description | Migratable | Status |
|---------|-------------|------------|--------|
| **LLM Post-Processing** | OpenAI/Anthropic text enhancement | ✅ Yes | ⏳ Pending |
| **Translation to English** | Whisper translation mode | ✅ Yes | ✅ Migrated |
| **Append Trailing Space** | Add space after transcription | ✅ Yes | ✅ Migrated |

#### History & Storage

| Feature | Description | Migratable |
|---------|-------------|------------|
| **Transcript History** | Save/view past transcriptions | ✅ Yes |
| **History Limit** | Maximum stored transcripts | ✅ Yes |
| **Recording Retention** | Audio file retention period | ✅ Yes |

#### System Integration (N/A for CLI)

| Feature | Description | Migratable |
|---------|-------------|------------|
| **Global Hotkeys** | Platform-specific keyboard shortcuts | ❌ No |
| **Text Injection** | Auto-paste to active app | ❌ No |
| **System Tray** | Background operation | ❌ No |
| **Auto-start** | OS startup registration | ❌ No |
| **Recording Overlay** | GTK layer shell | ❌ No |

#### Advanced Settings

| Feature | Description | Migratable |
|---------|-------------|------------|
| **Accelerator** | CPU/GPU/CUDA/DirectML selection | ✅ Yes |
| **GPU Device** | Multi-GPU device selection | ✅ Yes |
| **Log Level** | Debug/tracing configuration | ✅ Yes |
| **Debug Mode** | Verbose logging | ✅ Yes |

## Development Plan

### Phase 1: Core Extraction
- [x] Create Cargo project
- [x] Implement audio capture (cpal)
- [x] Implement VAD (vad-rs)
- [x] Implement ASR engines (transcribe-rs)
- [x] Implement basic HTTP service

### Phase 2: CLI Features
- [x] CLI command framework (clap)
- [x] serve command
- [x] list-models command
- [x] download command
- [x] delete command
- [x] Config file support

### Phase 3: Enhancement
- [x] Health check and status endpoints
- [x] Streaming transcription (SSE)
- [x] Model management enhancements
- [x] Documentation and examples

### Phase 4: Additional ASR Engines
- [x] Parakeet V2/V3 support
- [x] Moonshine model support
- [x] GigaAM support
- [x] Canary model support
- [x] Cohere model support

### Phase 5: Advanced Features
- [x] Translation to English mode (via translate parameter)
- [ ] Custom words/dictionary API
- [ ] LLM post-processing integration
- [ ] Transcript history API

### Phase 6: Packaging & Release
- [x] GitHub Actions CI/CD workflows
- [ ] Cross-platform packaging (CI builds)
- [ ] Release publishing (on tag)
- [ ] Pre-built binaries distribution

## License

MIT License (consistent with Handy)
