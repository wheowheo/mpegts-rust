# MPEG-TS Engine + Dashboard

A professional-grade MPEG-TS stream analyzer built with Rust and Svelte 5. Provides real-time monitoring, frame-level decoding, TR 101 290 compliance checking, and broadcast-instrument-style visualization.

## Features

- **TS Packet Parsing** — Full MPEG-TS packet demux with PAT/PMT/SDT/NIT, PES assembly, PCR/PTS/DTS extraction, SCTE-35 splice commands
- **Stream Analysis** — Continuity counter error detection, PCR jitter/drift analysis, per-PID bitrate statistics
- **Frame-Level Decoding** — H.264/H.265 NAL unit parsing (SPS/PPS/VPS, slice headers), AC-3/E-AC-3/AAC frame parsing, Dolby Atmos/Vision metadata
- **TR 101 290 Compliance** — Full Priority 1/2/3 monitoring (sync loss, PAT/PMT errors, PCR accuracy, PTS errors, NIT/SDT/EIT checks)
- **Packet Muxing & Output** — TS packet generation, PAT/PMT builder, UDP/RTP multicast output with CBR pacing
- **Thumbnail Extraction** — FFmpeg FFI-based I-frame decoding, proxy thumbnail generation
- **Live Ingest** — UDP multicast, RTP, SRT, HLS/DASH polling
- **HEX Editor View** — Color-highlighted hex dump with field tooltips
- **Session History** — SQLite-backed session management, error trends, bitrate history, report generation
- **Broadcast-Grade UI** — 7-segment displays, LED indicators, oscilloscope charts, multi-panel CMA-1820-style layout

## Architecture

```
crates/
  ts-core/       TS parser core (packet, PSI, PES, PCR, SCTE-35, muxer)
  ts-analyzer/   Analysis engine (CC errors, PCR jitter, bitrate stats, TR 101 290)
  ts-decoder/    Frame-level decoding (H.264/H.265 NAL, AC-3/E-AC-3, AAC)
  ts-server/     Axum web server (REST API + WebSocket + UDP/RTP output)
dashboard/       Svelte 5 SPA (real-time monitoring UI)
```

## Requirements

- Rust stable (1.75+)
- Node.js 18+
- FFmpeg shared libraries (for thumbnail decoding)

## Build & Run

```bash
# Build everything
cargo build --release
cd dashboard && npm install && npm run build && cd ..

# Run server (port 3200, serves dashboard as static files)
cargo run -p ts-server --release
```

Development mode:

```bash
# Terminal 1: Rust server
cargo run -p ts-server

# Terminal 2: Svelte dev server (port 5200, proxies API to 3200)
cd dashboard && npm run dev
```

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/stream` | Stream overview |
| GET | `/api/pids` | PID map |
| GET | `/api/pids/:pid` | PID detail |
| GET | `/api/pids/:pid/packets` | Raw packet bytes (hex view) |
| GET | `/api/pids/:pid/frames` | Frame list (type, size, PTS, DTS) |
| GET | `/api/pids/:pid/frames/:idx` | Individual frame detail |
| GET | `/api/pids/:pid/thumbnails` | Thumbnail list |
| GET | `/api/pids/:pid/thumbnail/:idx` | Thumbnail image |
| POST | `/api/analyze` | Upload & analyze TS file (multipart) |
| POST | `/api/ingest/start` | Start live ingest (UDP/RTP/SRT/HLS) |
| POST | `/api/ingest/stop` | Stop live ingest |
| GET | `/api/ingest/status` | Ingest status |
| GET | `/api/tr290` | TR 101 290 error summary |
| GET | `/api/system` | System stats (CPU, memory) |
| POST | `/api/output/start` | Start UDP/RTP output session |
| POST | `/api/output/stop` | Stop output session |
| GET | `/api/history/sessions` | Session history (paginated) |
| GET | `/api/history/sessions/:id` | Session detail |
| WS | `/ws` | Real-time analysis data stream |

## Test

```bash
cargo test
```

## License

MIT
