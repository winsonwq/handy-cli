# Agent Guidelines

This file provides guidance for AI agents working on this codebase.

## Documentation Standards

### Language Policy

- **Primary documentation language**: English
- All source code comments, README files, and documentation should be in English
- A Chinese translation of the main README is available at `docs/README_zh.md` and is linked from the root `README.md`

### Documentation Structure

```
handy-cli/
├── README.md           # English (primary)
├── SPEC.md            # English (primary)
├── AGENTS.md          # This file
└── docs/
    ├── README_zh.md   # Chinese translation
    └── ...            # Additional docs (indexed below)
```

### Docs Index

| Document | Language | Description |
|----------|----------|-------------|
| [README.md](../README.md) | EN | Main project documentation |
| [SPEC.md](../SPEC.md) | EN | Technical specification |
| [docs/README_zh.md](docs/README_zh.md) | 中文 | Chinese translation of README |

## Development Workflow

### Before Making Changes

1. Read relevant documentation files
2. Check existing tests and examples
3. Verify current implementation by reading source code

### Code Style

- Use Rust idioms and patterns
- Add inline comments for non-obvious logic
- Document public APIs with doc comments (`///`)
- Keep functions focused and single-purpose

### Testing

- Run `cargo test` before submitting changes
- Run `cargo check` for type checking
- Verify compilation with `cargo build --release`

### Commit Guidelines

- Use conventional commit format: `feat:`, `fix:`, `docs:`, `refactor:`, etc.
- Keep commits focused and atomic
- Write clear commit messages describing what and why

## Project Context

### What This Project Is

handy-cli is a standalone AI transcription CLI tool extracted from [Handy](https://github.com/cjpais/Handy). It provides:
- HTTP API for transcription
- Multi-engine support (Whisper, SenseVoice, etc.)
- Model management (download, delete, list)
- Voice Activity Detection (VAD)

### Key Technologies

- **Language**: Rust
- **HTTP Server**: axum
- **Audio**: cpal
- **VAD**: vad-rs
- **ASR**: transcribe-rs
- **CLI**: clap

### Important Paths

| Path | Purpose |
|------|---------|
| `src/commands/` | CLI command implementations |
| `src/server/` | HTTP API handlers |
| `src/models/` | Model management |
| `src/transcriber/` | ASR engine implementations |
| `src/audio/` | Audio capture |
| `src/vad/` | Voice activity detection |
