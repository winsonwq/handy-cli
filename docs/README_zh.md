# handy-cli

AI 听写命令行工具 - 从 [Handy](https://github.com/cjpais/Handy) 提取的独立转写引擎。

> **English**: 本文档为中文翻译，原版请查看 [../README.md](../README.md)

---

## 快速开始（用户指南）

### 下载预编译版本

从 [GitHub Releases](https://github.com/cjpais/handy-cli/releases) 页面下载适合您平台的最新版本：

| 平台 | 架构 | 下载文件 |
| 平台 | 架构 | 下载文件 |
|:------|:------|:----------|
| macOS | Apple Silicon (M1/M2/M3) | `handy-cli-x.x.x-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `handy-cli-x.x.x-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 | `handy-cli-x.x.x-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 (AArch64) | `handy-cli-x.x.x-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x86_64 | `handy-cli-x.x.x-x86_64-pc-windows-gnu.zip` |
| Windows | ARM64 | `handy-cli-x.x.x-aarch64-pc-windows-gnu.zip` |
### 使用方法

```bash
# 解压压缩包
tar -xzf handy-cli-*.tar.gz  # Linux/macOS
# 或 unzip handy-cli-*.zip    # Windows

# 启动 HTTP 服务
./handy-cli serve --port 8765

# 指定引擎和模型
./handy-cli serve --engine sensevoice --model sense-voice-int8

# 列出可用模型
./handy-cli list-models

# 下载模型
./handy-cli download --model sense-voice-int8

# 删除模型
./handy-cli delete --model small

# 检查环境
./handy-cli doctor
```

### API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/models` | 列出可用模型 |
| GET | `/api/models/downloaded` | 列出已下载模型 |
| POST | `/api/transcribe` | 转写音频 |
| POST | `/api/transcribe/stream` | 流式转写（SSE） |

#### 转写请求格式

```json
POST /api/transcribe
{
  "audio": "<base64编码的16bit单声道PCM>",
  "language": "auto",
  "sample_rate": 16000
}
```

#### 响应格式

```json
{
  "text": "转写文本",
  "language": "zh",
  "duration": 2.5
}
```

### 支持的模型

共支持 **7 个引擎**，16+ 个模型：

| 引擎 | 语言 | 特点 |
|:-----|:-----|:-----|
| **SenseVoice** | 中文/英文/日韩/粤语 | 中文优化，快速识别 |
| **Whisper** | 100+ 语言 | 多语言，支持翻译 |
| **Parakeet** | 英文/欧洲语言 | 英文最佳 |
| **Moonshine** | 英文 | 极速流式识别 |
| **GigaAM** | 俄语 | 俄语识别 |
| **Canary** | 英/德/西/法等 | 支持翻译 |
| **Cohere** | 100+ 语言 | 最高准确度 |

**可用模型列表**：

| 模型 | 大小 | 语言 | 引擎 |
|:-----|:----:|:----:|:-----|
| sense-voice-int8 | ~230MB | 中文/英文 | SenseVoice |
| sense-voice-fp16 | ~450MB | 中文/英文 | SenseVoice |
| tiny | ~75MB | 多语言 | Whisper |
| base | ~150MB | 多语言 | Whisper |
| small | ~465MB | 多语言 | Whisper |
| medium | ~1.5GB | 多语言 | Whisper |
| large | ~3GB | 多语言 | Whisper |
| large-v3-turbo | ~1.7GB | 多语言 | Whisper |
| small.en | ~465MB | 英文 | Whisper |
| medium.en | ~1.5GB | 英文 | Whisper |
| distil-medium.en | ~1.4GB | 英文 | Whisper |
| parakeet-tdt-cv11 | ~330MB | 英文 | Parakeet |
| parakeet-tdt-d温220m | ~31MB | 英文 | Parakeet |
| moonshine-base | ~440MB | 英文 | Moonshine |
| moonshine-tiny | ~110MB | 英文 | Moonshine |
| gigabyte | ~1.3GB | 俄语 | GigaAM |
| canary-1b | ~2GB | 英/德/西/法 | Canary |
| cohere-english-v3.5 | ~2.5GB | 多语言 | Cohere |
| cohere-multilingual-v3.0 | ~2.5GB | 多语言 | Cohere |
---

## 开发指南（贡献者）

### 前置要求

- Rust 1.70+（通过 [rustup](https://rustup.rs/) 安装）

### 构建依赖

#### macOS

```bash
brew install pkg-config openssl
```

#### Ubuntu / Debian

```bash
sudo apt install libssl-dev pkg-config libasound2-dev
```

#### Windows

使用 Visual Studio Build Tools 安装 C++ 工作负载，或使用 MSYS2：

```bash
pacman -S mingw-w64-x86_64-openssl mingw-w64-x86_64-pkg-config
```

### 构建

```bash
cargo build --release
```

二进制文件将位于 `target/release/handy-cli`。

### 运行测试

```bash
cargo test --all-features
```

### 代码质量检查

```bash
# 检查代码格式
cargo fmt --all -- --check

# 运行 clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### 项目结构

```
src/
├── main.rs           # 入口点，CLI 参数解析
├── commands/          # CLI 命令实现
│   ├── serve.rs      # HTTP 服务命令
│   ├── download.rs   # 模型下载命令
│   ├── list_models.rs
│   ├── delete.rs
│   └── doctor.rs     # 环境检查
├── server/           # HTTP API 处理器
│   └── handlers/
├── models/           # 模型管理
├── transcriber/      # ASR 引擎实现
├── audio/            # 音频采集
└── vad/              # 语音活动检测
```

### 架构

```
CLI → HTTP Server (axum) → Audio (cpal) → VAD (vad-rs) → ASR (transcribe-rs)
```

### 配置文件

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

### 提交规范

使用约定式提交格式：

- `feat:` - 新功能
- `fix:` - 错误修复
- `docs:` - 文档更新
- `refactor:` - 代码重构
- `test:` - 添加测试

---

## 许可证

MIT
