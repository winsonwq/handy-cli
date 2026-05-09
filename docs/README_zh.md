# handy-cli

AI 听写命令行工具 - 从 [Handy](https://github.com/cjpais/Handy) 提取的独立转写引擎。

> **English**: 本文档为中文翻译，原版请查看 [../README.md](../README.md)

## 功能特性

- 🌐 **HTTP API** - 提供 REST API 用于转写
- 🎤 **音频采集** - 跨平台麦克风录音 (cpal)
- 🎯 **VAD** - 语音活动检测 (vad-rs)
- 🤖 **多引擎支持** - Whisper.cpp / SenseVoice ONNX (transcribe-rs)
- 📦 **模型管理** - 下载和管理 ASR 模型

## 构建依赖

### macOS

```bash
brew install pkg-config openssl
```

### Ubuntu / Debian

```bash
sudo apt install libssl-dev pkg-config
```

### Windows (使用 MSYS2)

```bash
pacman -S mingw-w64-x86_64-openssl mingw-w64-x86_64-pkg-config
```

## 构建

```bash
cargo build --release
```

## 使用方法

```bash
# 启动 HTTP 服务
./target/release/handy-cli serve --port 8765

# 指定引擎和模型
./target/release/handy-cli serve --engine sensevoice --model sense-voice-int8

# 列出可用模型
./target/release/handy-cli list-models

# 下载模型
./target/release/handy-cli download --model sense-voice-int8

# 删除模型
./target/release/handy-cli delete --model small

# 检查环境
./target/release/handy-cli doctor
```

## API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/models` | 列出可用模型 |
| GET | `/api/models/downloaded` | 列出已下载模型 |
| POST | `/api/transcribe` | 转写音频 |

### 转写请求格式

```json
POST /api/transcribe
{
  "audio": "<base64编码的16bit单声道PCM>",
  "language": "auto",
  "sample_rate": 16000
}
```

### 响应格式

```json
{
  "text": "转写文本",
  "language": "zh",
  "duration": 2.5
}
```

## 支持的引擎

| 引擎 | 模型 | 大小 | 特点 |
|------|------|------|------|
| SenseVoice | sense-voice-int8 | ~230MB | 中文优化、自带标点 |
| Whisper | tiny/base/small/medium/large | ~75MB/~150MB/~465MB/~1.5GB/~3GB | 多语言支持 |

### Whisper 模型

Whisper 模型需要从 HuggingFace 下载：

```bash
# 下载 tiny 模型 (~75MB)
curl -L -o ~/.cache/handy-cli/models/ggml-tiny.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
```

## 架构

```
CLI → HTTP Server (axum) → Audio (cpal) → VAD (vad-rs) → ASR (transcribe-rs)
```

## 配置文件

默认配置路径: `~/.config/handy-cli/config.yaml`

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

## 许可证

MIT
