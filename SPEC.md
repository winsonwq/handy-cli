# handy-cli — AI 听写核心工具

## 概述

handy-cli 是一个独立运行的 AI 听写 CLI 工具，提取自 [Handy](https://github.com/cjpais/Handy)（Tauri 应用）的核心功能。

**目标：** 提供一个跨平台（macOS/Linux/Windows）的命令行工具，启动 HTTP 服务提供语音转文字功能，无需安装任何运行时。

## 核心原则

1. **零依赖运行** — 打包成单个可执行文件，用户直接下载运行
2. **保留 Handy 优势** — 多引擎支持、模型管理、VAD 等
3. **CLI 可配置** — 通过命令行参数或配置文件调整行为
4. **无 UI** — 纯 CLI + HTTP API，专注后端能力

## 与 Handy 的关系

| 组件 | Handy (Tauri) | Handy (本项目) |
|------|--------------|---------------|
| 前端 | React UI | ❌ 移除 |
| 快捷键 | rdev/enigo | ❌ 移除 |
| 文字注入 | CGEvent/SendInput | ❌ 移除 |
| 菜单栏/Tray | Tauri API | ❌ 移除 |
| 音频采集 | cpal | ✅ 保留 |
| VAD | vad-rs | ✅ 保留 |
| ASR 引擎 | transcribe-rs | ✅ 保留 |
| 模型管理 | 现有逻辑 | ✅ 保留 |
| HTTP 服务 | ❌ | ✅ 新增 |

## 功能列表

### 1. HTTP API 服务

启动本地 HTTP 服务，提供 REST + SSE 接口。

**端点：**

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/models` | 列出可用模型 |
| GET | `/api/models/downloaded` | 列出已下载模型 |
| POST | `/api/models/download` | 下载模型 |
| POST | `/api/transcribe` | 转写音频（JSON body） |
| POST | `/api/transcribe/stream` | 流式转写（SSE） |
| POST | `/api/audio/start` | 开始录音 |
| POST | `/api/audio/stop` | 停止录音 |
| GET | `/api/audio/status` | 录音状态 |

**转写响应格式：**
```json
{
  "text": "转写文字内容",
  "language": "zh",
  "duration": 3.5,
  "language_probability": 0.99
}
```

**SSE 事件：**
```
event: speech_start
data: {"timestamp": 1234567890}

event: speech_end
data: {"timestamp": 1234567890, "duration": 3.5}

event: transcript
data: {"text": "转写中...", "partial": true}

event: transcript
data: {"text": "最终转写结果。", "partial": false}
```

### 2. 音频采集

- 使用 `cpal` 跨平台采集麦克风音频
- 支持采样率配置（默认 16kHz）
- 支持选择输入设备
- 音频格式：float32 PCM

### 3. VAD (Voice Activity Detection)

- 使用 `vad-rs` 进行语音活动检测
- 可配置阈值、静音时长等参数
- 事件：`speech_start`, `speech_end`

### 4. ASR 引擎

支持多种引擎，通过 `transcribe-rs` 实现：

| 引擎 | 模型 | 模型大小 | 特点 |
|------|------|---------|------|
| whisper.cpp | tiny/base/small/medium/large | ~75MB/~150MB/~500MB/~1.5GB/~3GB | 多语言 |
| SenseVoice | sense-voice-int8 | ~230MB | 中文优化、自带标点 |

**模型下载：**
- 从 Handy 官方 CDN 下载（`blob.handy.computer`）
- 或从 HuggingFace 下载

### 5. CLI 命令

```bash
# 启动 HTTP 服务
handy-cli serve [选项]

# 选项：
#   --port <端口>        HTTP 端口（默认 8765）
#   --host <地址>       监听地址（默认 127.0.0.1）
#   --engine <引擎>     whisper / sensevoice（默认 sensevoice）
#   --model <模型>      模型名称（默认 sense-voice-int8）
#   --vad-threshold     VAD 阈值 0.0-1.0（默认 0.5）
#   --language <语言>    语言代码（默认 auto）

# 列出可用模型
handy-cli list-models

# 下载模型
handy-cli download --model <模型名>

# 检查环境
handy-cli doctor

# 版本信息
handy-cli --version
```

### 6. 配置文件

支持 YAML 配置文件（`~/.handy-cli/config.yaml`）：

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

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                    CLI (clap)                           │
│   serve / list-models / download / doctor               │
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
│                    VAD (vad-rs)                        │
│   speech detection                                      │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│              Transcriber (transcribe-rs)                │
│   whisper.cpp / SenseVoice ONNX                         │
└─────────────────────────────────────────────────────────┘
```

## 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 1.75+ |
| Web 框架 | axum | 0.7+ |
| 音频采集 | cpal | 0.16+ |
| VAD | vad-rs | git |
| ASR | transcribe-rs | 0.3+ |
| CLI | clap | 4.x |
| 日志 | tracing | 0.1+ |
| 配置 | serde_yaml | 0.9+ |

## 项目结构

```
handy-cli/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── serve.rs        # serve 命令
│   │   ├── list_models.rs  # list-models 命令
│   │   ├── download.rs     # download 命令
│   │   └── doctor.rs       # doctor 命令
│   ├── server/
│   │   ├── mod.rs
│   │   ├── api.rs          # REST 端点
│   │   ├── sse.rs           # SSE 端点
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── health.rs
│   │       ├── transcribe.rs
│   │       └── models.rs
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── capture.rs       # cpal 采集
│   │   └── device.rs        # 设备管理
│   ├── vad/
│   │   └── mod.rs           # vad-rs 封装
│   ├── transcriber/
│   │   ├── mod.rs
│   │   ├── whisper.rs       # whisper.cpp
│   │   └── sensevoice.rs    # SenseVoice
│   ├── models/
│   │   ├── mod.rs
│   │   ├── manager.rs       # 模型管理/下载
│   │   └── registry.rs      # 模型注册表
│   ├── config/
│   │   └── mod.rs           # 配置文件
│   └── error.rs             # 错误类型
├── config.yaml.example      # 配置文件示例
├── build.rs                 # Tauri build（预留）
└── README.md
```

## 打包输出

| 平台 | 格式 | 文件名 |
|------|------|--------|
| macOS Apple Silicon | tar.gz | handy-macos-aarch64.tar.gz |
| macOS Intel | tar.gz | handy-macos-x86_64.tar.gz |
| Linux | tar.gz | handy-linux-x86_64.tar.gz |
| Windows | zip | handy-windows-x86_64.zip |

## 开发计划

### Phase 1: 核心抽取
- [ ] 创建 Cargo 项目
- [ ] 实现音频采集（cpal）
- [ ] 实现 VAD（vad-rs）
- [ ] 实现 ASR 引擎（transcribe-rs）
- [ ] 实现基础 HTTP 服务

### Phase 2: CLI 功能
- [ ] CLI 命令框架（clap）
- [ ] serve 命令
- [ ] list-models 命令
- [ ] download 命令
- [ ] 配置文件支持

### Phase 3: 完善
- [ ] 健康检查和状态接口
- [ ] 流式转写（SSE）
- [ ] 模型管理增强
- [ ] 文档和示例

### Phase 4: 打包发布
- [ ] GitHub Actions CI/CD
- [ ] 跨平台打包
- [ ] Release 发布

## 许可

MIT License（与 Handy 保持一致）
