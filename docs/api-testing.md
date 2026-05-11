# API Testing Guide

This guide covers how to test the handy-cli HTTP API endpoints.

## Prerequisites

### Convert Audio to Base64

```bash
# Convert WAV to base64 (without newlines for streaming API)
base64 -i audio.wav | tr -d '\n' > audio.b64

# For testing with raw PCM (16kHz, mono, 16-bit)
ffmpeg -i input.mp3 -ar 16000 -ac 1 -acodec pcm_s16le input.pcm -y
base64 -i input.pcm | tr -d '\n' > audio.b64
```

## Endpoints

### Health Check

```bash
curl http://localhost:8765/api/health
```

### List Available Models

```bash
curl http://localhost:8765/api/models
```

### List Downloaded Models

```bash
curl http://localhost:8765/api/models/downloaded
```

### Transcribe (One-shot)

Send complete audio and wait for result.

```bash
curl -X POST http://localhost:8765/api/transcribe \
  -H "Content-Type: application/json" \
  -d "{\"audio\": \"$(cat audio.b64)\", \"sample_rate\": 16000}"
```

### Transcribe (Streaming)

Send audio data incrementally and receive partial results via SSE.

```bash
curl -s -N -X POST "http://localhost:8765/api/transcribe/stream?sample_rate=16000" \
  -H "Content-Type: text/plain" \
  --data-binary @audio.b64
```

## Testing with Chunked Transfer (True Streaming)

To test real streaming behavior with incremental results, use Python with `http.client`:

```python
import http.client
import json
import time

# Read base64 audio
with open('audio.b64', 'r') as f:
    audio_b64 = f.read()

# Split into chunks
num_chunks = 5
chunk_size = len(audio_b64) // num_chunks
chunks = [audio_b64[i*chunk_size:(i+1)*chunk_size] for i in range(num_chunks)]

# Connect with chunked transfer encoding
conn = http.client.HTTPConnection('localhost', 8765, timeout=60)
conn.putrequest('POST', '/api/transcribe/stream?sample_rate=16000')
conn.putheader('Content-Type', 'text/plain')
conn.putheader('Transfer-Encoding', 'chunked')
conn.endheaders()

results = []
for i, chunk in enumerate(chunks):
    chunk_bytes = chunk.encode()
    conn.send(f"{len(chunk_bytes):x}\r\n".encode())
    conn.send(chunk_bytes)
    conn.send(b"\r\n")
    print(f"Sent chunk {i+1}/{num_chunks}")
    time.sleep(0.5)  # Delay between chunks

conn.send(b"0\r\n\r\n")  # End chunked transfer

# Read response
resp = conn.getresponse()
print(f"Status: {resp.status}")
body = resp.read().decode()
print("Events:")
for line in body.split('\n'):
    if line.startswith('data:'):
        data = json.loads(line[5:])
        print(f"  partial={data.get('partial')}: {data.get('text', '')[:50]}...")

conn.close()
```

## Testing with Large Files

```python
import base64
import requests

with open('large_audio.b64', 'r') as f:
    audio_b64 = f.read()

# One-shot
resp = requests.post(
    'http://localhost:8765/api/transcribe',
    json={'audio': audio_b64, 'sample_rate': 16000},
    timeout=300
)
print(resp.json())

# Streaming
resp = requests.post(
    'http://localhost:8765/api/transcribe/stream',
    json={'audio': audio_b64, 'sample_rate': 16000},
    stream=True,
    timeout=300
)
for line in resp.iter_lines():
    if line:
        print(line.decode())
```

## Common Issues

### "length limit exceeded"

Increase the body limit in `src/commands/serve.rs`:

```rust
use axum::extract::DefaultBodyLimit;

let app = router
    .layer(DefaultBodyLimit::max(100 * 1024 * 1024))  // 100MB
    ...
```

### MP3 files not working

Server expects raw PCM audio (16-bit, mono). Convert first:

```bash
ffmpeg -i input.mp3 -ar 16000 -ac 1 -acodec pcm_s16le input.pcm -y
```

### Base64 with newlines fails

Remove newlines from base64:

```bash
base64 -i audio.pcm | tr -d '\n' > audio.b64
```
