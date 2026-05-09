# Feature Comparison Analysis: Handy vs handy-cli

## Project Overview

**Handy** (Tauri App): A full-featured desktop speech-to-text application with UI, global hotkeys, text injection, and extensive customization options.

**handy-cli** (This Project): A standalone CLI tool that extracts core transcription functionality as an HTTP API server.

---

## Feature Comparison Matrix

### Core Transcription (Migrated)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| Audio Capture (cpal) | ✅ | ✅ | Migrated |
| VAD (vad-rs/Silero) | ✅ | ✅ | Migrated |
| Whisper Models | ✅ | ✅ | Migrated |
| SenseVoice | ✅ | ✅ | Migrated |
| Model Management | ✅ | ✅ | Migrated |
| HTTP API Server | ❌ | ✅ | New Feature |
| Streaming Transcription (SSE) | ❌ | ✅ | New Feature |

### ASR Engine Support (Partial)

| Engine | Handy | handy-cli | Status |
|--------|-------|-----------|--------|
| Whisper (tiny/base/small/medium/large) | ✅ | ✅ | Migrated |
| SenseVoice | ✅ | ✅ | Migrated |
| Parakeet V2/V3 | ✅ | ❌ | **Not Migrated** |
| Moonshine (base/tiny/small/medium) | ✅ | ❌ | **Not Migrated** |
| GigaAM v3 | ✅ | ❌ | **Not Migrated** |
| Canary 180M/1B | ✅ | ❌ | **Not Migrated** |
| Cohere | ✅ | ❌ | **Not Migrated** |
| Breeze ASR | ✅ | ❌ | **Not Migrated** |
| Custom Whisper Model Discovery | ✅ | ❌ | **Not Migrated** |

### UI/Frontend (Not Applicable for CLI)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| React Settings UI | ✅ | ❌ | N/A (CLI) |
| Recording Overlay | ✅ | ❌ | N/A |
| Debug Mode (Cmd+Shift+D) | ✅ | ❌ | N/A |
| Onboarding Flow | ✅ | ❌ | N/A |

### Input Control (Not Migrated)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| Global Keyboard Shortcuts | ✅ | ❌ | **Not Migrated** |
| Push-to-Talk Mode | ✅ | ❌ | **Not Migrated** |
| Text Injection | ✅ | ❌ | **Not Migrated** |
| Clipboard Handling | ✅ | ❌ | **Not Migrated** |
| Typing Tool Selection | ✅ | ❌ | **Not Migrated** |
| External Script Execution | ✅ | ❌ | **Not Migrated** |

### Audio Settings (Partial)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| Microphone Selection | ✅ | ✅ | Migrated |
| Audio Feedback Sounds | ✅ | ❌ | **Not Migrated** |
| Sound Themes | ✅ | ❌ | **Not Migrated** |
| Mute While Recording | ✅ | ❌ | **Not Migrated** |
| Clamshell Microphone | ✅ | ❌ | **Not Migrated** |
| Extra Recording Buffer | ✅ | ❌ | **Not Migrated** |

### Post-Processing (Not Migrated)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| LLM Post-Processing | ✅ | ❌ | **Not Migrated** |
| OpenAI Integration | ✅ | ❌ | **Not Migrated** |
| Anthropic Integration | ✅ | ❌ | **Not Migrated** |
| Custom Prompts | ✅ | ❌ | **Not Migrated** |
| Translate to English | ✅ | ❌ | **Not Migrated** |
| Custom Words/Dictionary | ✅ | ❌ | **Not Migrated** |
| Filler Word Filtering | ✅ | ❌ | **Not Migrated** |

### History & Storage (Not Migrated)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| Transcript History | ✅ | ❌ | **Not Migrated** |
| Recording Storage | ✅ | ❌ | **Not Migrated** |
| History Retention Settings | ✅ | ❌ | **Not Migrated** |

### System Integration (Not Migrated)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| System Tray | ✅ | ❌ | **Not Migrated** |
| Auto-start | ✅ | ❌ | **Not Migrated** |
| Portable Mode | ✅ | ❌ | **Not Migrated** |
| Single Instance | ✅ | ❌ | **Not Migrated** |
| Auto Updates | ✅ | ❌ | **Not Migrated** |

### Advanced Settings (Not Migrated)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| Model Unload Timeout | ✅ | ❌ | **Not Migrated** |
| GPU Device Selection | ✅ | ❌ | **Not Migrated** |
| Accelerator Settings (CUDA/Metal/Vulkan) | ✅ | ❌ | **Not Migrated** |
| Paste Delay | ✅ | ❌ | **Not Migrated** |
| Auto-submit Options | ✅ | ❌ | **Not Migrated** |
| App Language/i18n | ✅ | ❌ | **Not Migrated** |

### Debugging & Logging (Partial)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| File Logging | ✅ | ❌ | **Not Migrated** |
| Log Level Control | ✅ | ❌ | **Not Migrated** |
| Doctor Command | ❌ | ✅ | New Feature |

### Raycast Extension (External)
| Feature | Handy | handy-cli | Status |
|---------|-------|-----------|--------|
| Raycast Integration | ✅ | ❌ | External |

---

## Summary: Not Migrated Features

### High Priority (Could be implemented)
1. **Additional ASR Engines**: Parakeet, Moonshine, GigaAM, Canary, Cohere, Breeze ASR
2. **Custom Whisper Model Discovery**: Auto-detect custom GGML models
3. **LLM Post-Processing**: OpenAI/Anthropic/etc. integration for text enhancement
4. **Custom Words/Dictionary**: Word correction and filtering
5. **Transcript History API**: GET /api/history endpoints

### Medium Priority
6. **Audio Feedback**: Sound notifications for recording start/stop
7. **Paste Options**: Configurable paste method, delay, auto-submit
8. **Streaming Enhancement**: Partial transcription results during recording
9. **Recording Storage**: Save raw recordings for later processing

### Low Priority (UI-dependent, difficult for CLI)
10. **Global Hotkeys**: Platform-specific implementation required
11. **Text Injection**: CGEvent/SendInput/enigo for auto-paste
12. **System Tray**: Background operation mode
13. **Auto-start**: OS-specific startup registration
14. **Debug Mode UI**: Development troubleshooting interface

### Not Applicable (CLI limitation)
15. **React Settings UI**: Configuration via config file instead
16. **Recording Overlay**: GTK layer shell not applicable
17. **Onboarding Flow**: First-run experience not needed
18. **Raycast Extension**: External integration
