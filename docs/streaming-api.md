# Streaming Transcription API

## Overview

The streaming transcription API (`POST /api/transcribe/stream`) provides real-time transcription of audio data as it is received. Unlike the one-shot API which waits for the complete audio file, the streaming API:

1. **Accepts audio chunks as they arrive** - no need to wait for the complete file
2. **Triggers transcription incrementally** - partial results are sent as soon as enough audio is accumulated
3. **Returns final result** - when all audio data has been received

## Algorithm

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Streaming Transcription Flow                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Client                        Server                                 │
│    │                            │                                     │
│    │  POST /api/transcribe/stream                                     │
│    ├───────────────────────────>│                                     │
│    │  [audio chunk 1]           │ Receive & accumulate                │
│    ├───────────────────────────>│                                     │
│    │                            │ audio_buffer += chunk1              │
│    │                            │                                     │
│    │  [audio chunk 2]           │ Receive & accumulate                │
│    ├───────────────────────────>│                                     │
│    │                            │ audio_buffer += chunk2              │
│    │                            │                                     │
│    │                            │ ┌─────────────────────────────┐    │
│    │                            │ │ If audio_buffer >= 16K      │    │
│    │                            │ │ samples (1 second):        │    │
│    │                            │ │   trigger transcription()   │    │
│    │                            │ │   send SSE event           │    │
│    │                            │ └─────────────────────────────┘    │
│    │                            │                                     │
│    │<───────────────────────────│ SSE: {"partial": true, "text": "..."}
│    │                            │                                     │
│    │  [more audio chunks]       │ Receive & accumulate                │
│    ├───────────────────────────>│                                     │
│    │                            │ ┌─────────────────────────────┐    │
│    │                            │ │ Trigger transcription()      │    │
│    │                            │ │ send SSE event              │    │
│    │                            │ └─────────────────────────────┘    │
│    │                            │                                     │
│    │<───────────────────────────│ SSE: {"partial": true, "text": "..."}
│    │                            │                                     │
│    │  [EOF / request complete]  │ Final transcription                │
│    ├───────────────────────────>│                                     │
│    │                            │ ┌─────────────────────────────┐    │
│    │                            │ │ Transcribe full audio       │    │
│    │                            │ │ send SSE event (partial=0) │    │
│    │                            │ └─────────────────────────────┘    │
│    │                            │                                     │
│    │<───────────────────────────│ SSE: {"partial": false, "text": "..."}
│    │                            │                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Implementation Details

### 1. Audio Accumulation

The server continuously receives and accumulates all audio data in memory:

```rust
let mut audio_buffer: Vec<f32> = Vec::new();
let mut base64_buffer = Vec::new();

// For each chunk received:
base64_buffer.extend_from_slice(&chunk);
audio_buffer.extend(decode_base64_to_samples(&base64_buffer));
```

### 2. Incremental Transcription Trigger

Every time enough new samples are accumulated (16,000 samples = 1 second at 16kHz), a transcription is triggered:

```rust
const PARTIAL_TRANSCRIBE_SAMPLES: usize = 16000;

if audio_buffer.len() - last_triggered_samples >= PARTIAL_TRANSCRIBE_SAMPLES {
    // Trigger transcription in background task
    let samples = audio_buffer.clone();
    tokio::spawn(async move {
        let result = transcriber.transcribe(&samples);
        sender.send(SseEventData::Transcript {
            text: result.text,
            partial: true,  // More results may come
        });
    });

    last_triggered_samples = audio_buffer.len();
}
```

### 3. SSE Events

The server sends Server-Sent Events (SSE) with the following format:

```json
// Partial result (more may come)
{"partial": true, "text": "转写文本..."}

// Final result (no more results)
{"partial": false, "text": "完整转写文本..."}
```

## API Usage

### Request

```bash
curl -X POST "http://localhost:8765/api/transcribe/stream?sample_rate=16000" \
  -H "Content-Type: text/plain" \
  -d "$(base64 -i audio.wav | tr -d '\n')"
```

**Parameters:**
- `sample_rate` (optional, default: 16000) - Audio sample rate
- `language` (optional) - Language hint (e.g., "zh", "en")
- `translate` (optional) - Whether to translate to English

**Body:** Base64-encoded audio data (16-bit PCM, mono)

### Response

```
event: transcript
data: {"partial":true,"text":"Receiving audio..."}

event: transcript
data: {"partial":true,"text":"[Partial 1.0s] Transcribing..."}

event: transcript
data: {"partial":true,"text":"部分转写结果..."}

event: transcript
data: {"partial":true,"text":"更多转写结果..."}

event: transcript
data: {"partial":false,"text":"完整最终转写结果..."}
```

## Client Implementation Tips

### JavaScript (Browser)

```javascript
const response = await fetch('/api/transcribe/stream', {
  method: 'POST',
  headers: {
    'Content-Type': 'text/plain',
    'Accept': 'text/event-stream'
  },
  body: base64AudioData
});

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;

  const text = decoder.decode(value);
  // Parse SSE events from text
  const lines = text.split('\n');
  for (const line of lines) {
    if (line.startsWith('data:')) {
      const data = JSON.parse(line.slice(5));
      if (data.partial === false) {
        console.log('Final result:', data.text);
      } else {
        console.log('Partial result:', data.text);
      }
    }
  }
}
```

### Python

```python
import requests

response = requests.post(
    'http://localhost:8765/api/transcribe/stream',
    params={'sample_rate': 16000},
    data=audio_base64,
    stream=True,
    headers={'Accept': 'text/event-stream'}
)

for line in response.iter_lines():
    if line.startswith('data:'):
        import json
        data = json.loads(line[5:])
        if data['partial']:
            print(f"Partial: {data['text']}")
        else:
            print(f"Final: {data['text']}")
```

## Performance Considerations

- **Memory**: Audio is accumulated in memory. For very long recordings, consider using a time-limited buffer or implementing a sliding window.
- **Transcription Frequency**: Currently set to 1 second intervals. Adjust `PARTIAL_TRANSCRIBE_SAMPLES` to balance between responsiveness and server load.
- **Background Tasks**: Each partial transcription runs in a Tokio background task. Results are sent via broadcast channel.

## Future Improvements

- [ ] WebSocket support for bidirectional communication
- [ ] Moonshine streaming model for true incremental transcription
- [ ] Sliding window for memory-efficient processing of long recordings
- [ ] Audio format auto-detection (currently expects 16-bit PCM)
