# MPEG-TS Engine + Dashboard

## 프로젝트 아키텍처

```
ts-engine/
├── Cargo.toml                    # workspace root
├── rust-toolchain.toml
├── .cargo/
│   └── config.toml               # 링커 설정 (lld/sold)
│
├── crates/
│   ├── ts-core/                  # MPEG-TS 파서 코어 (순수 Rust, no_std 가능)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── packet.rs         # TS 패킷 (188 bytes) 파싱
│   │       ├── pid.rs            # PID 필터링, PID 맵
│   │       ├── psi/
│   │       │   ├── mod.rs
│   │       │   ├── pat.rs        # Program Association Table
│   │       │   ├── pmt.rs        # Program Map Table
│   │       │   ├── sdt.rs        # Service Description Table
│   │       │   └── nit.rs        # Network Information Table
│   │       ├── pes.rs            # PES 패킷 조립/파싱
│   │       ├── timing.rs         # PCR, PTS, DTS 추출 및 분석
│   │       ├── scte35.rs         # SCTE-35 스플라이스 커맨드
│   │       ├── descriptors.rs    # 디스크립터 파싱
│   │       └── bitrate.rs        # 비트레이트 계산
│   │
│   ├── ts-analyzer/              # 분석 엔진 (ts-core 위에 통계/집계)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── stream_info.rs    # 스트림 전체 정보 집계
│   │       ├── continuity.rs     # CC 에러 감지
│   │       ├── pcr_jitter.rs     # PCR 지터/드리프트 분석
│   │       └── bitrate_stats.rs  # PID별 비트레이트 시계열
│   │
│   ├── ts-server/                # axum 웹서버 + WebSocket
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── api/
│   │       │   ├── mod.rs
│   │       │   ├── stream.rs     # REST: 스트림 정보
│   │       │   ├── pid.rs        # REST: PID 맵/상세
│   │       │   └── analyze.rs    # REST: 파일 분석 시작
│   │       ├── ws/
│   │       │   ├── mod.rs
│   │       │   └── realtime.rs   # WebSocket: 실시간 데이터 푸시
│   │       ├── ingest/
│   │       │   ├── mod.rs
│   │       │   ├── file.rs       # 파일 입력
│   │       │   └── udp.rs        # UDP 멀티캐스트 수신 (라이브)
│   │       └── state.rs          # 앱 상태 (Arc<RwLock<...>>)
│   │
│   └── ts-napi/                  # (선택) Node.js napi-rs 바인딩
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── dashboard/                    # Svelte 5 SPA
│   ├── package.json
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css
│   │   ├── lib/
│   │   │   ├── api.ts            # REST 클라이언트
│   │   │   ├── ws.ts             # WebSocket 클라이언트
│   │   │   ├── stores/
│   │   │   │   ├── stream.svelte.ts    # rune: 스트림 상태
│   │   │   │   ├── pids.svelte.ts      # rune: PID 맵
│   │   │   │   └── realtime.svelte.ts  # rune: 실시간 메트릭
│   │   │   ├── components/
│   │   │   │   ├── PidMap.svelte       # PID 맵 테이블/트리
│   │   │   │   ├── BitrateChart.svelte # 비트레이트 시계열 차트
│   │   │   │   ├── PcrTimeline.svelte  # PCR/PTS/DTS 타임라인
│   │   │   │   ├── PsiViewer.svelte    # PSI 테이블 뷰어
│   │   │   │   ├── CcErrors.svelte     # CC 에러 로그
│   │   │   │   ├── Scte35Log.svelte    # SCTE-35 이벤트
│   │   │   │   └── StreamSummary.svelte# 요약 카드
│   │   │   └── types/
│   │   │       └── index.ts            # API 응답 타입 정의
│   │   └── routes/
│   │       ├── +layout.svelte
│   │       ├── +page.svelte            # 메인 대시보드
│   │       ├── pid/
│   │       │   └── [pid]/+page.svelte  # PID 상세
│   │       └── scte35/
│   │           └── +page.svelte        # SCTE-35 로그
│   └── static/
│
└── README.md
```

---

## Rust 백엔드 상세

### Cargo workspace (루트 Cargo.toml)

```toml
[workspace]
resolver = "2"
members = [
    "crates/ts-core",
    "crates/ts-analyzer",
    "crates/ts-server",
    # "crates/ts-napi",   # 필요 시 활성화
]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
bytes = "1"
```

### ts-core/Cargo.toml

```toml
[package]
name = "ts-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
bytes = { workspace = true }
bitflags = "2"
```

### ts-server/Cargo.toml

```toml
[package]
name = "ts-server"
version = "0.1.0"
edition = "2021"

[dependencies]
ts-core = { path = "../ts-core" }
ts-analyzer = { path = "../ts-analyzer" }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
axum = { version = "0.8", features = ["ws"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }
```

---

## 핵심 코드 스캐폴딩

### ts-core/src/packet.rs

```rust
use serde::Serialize;
use thiserror::Error;

pub const TS_PACKET_SIZE: usize = 188;
pub const SYNC_BYTE: u8 = 0x47;

#[derive(Error, Debug)]
pub enum TsError {
    #[error("sync byte mismatch: expected 0x47, got 0x{0:02X}")]
    SyncError(u8),
    #[error("packet too short: {0} bytes")]
    TooShort(usize),
    #[error("invalid adaptation field")]
    InvalidAdaptation,
}

#[derive(Debug, Clone, Serialize)]
pub struct TsHeader {
    pub sync_byte: u8,
    pub transport_error: bool,
    pub payload_unit_start: bool,
    pub transport_priority: bool,
    pub pid: u16,
    pub scrambling_control: u8,
    pub adaptation_field_control: u8,
    pub continuity_counter: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdaptationField {
    pub length: u8,
    pub discontinuity: bool,
    pub random_access: bool,
    pub pcr: Option<u64>,       // 90kHz 기준 (33bit base + 9bit ext)
    pub opcr: Option<u64>,
    pub splice_countdown: Option<i8>,
}

#[derive(Debug, Clone)]
pub struct TsPacket {
    pub header: TsHeader,
    pub adaptation: Option<AdaptationField>,
    pub payload: Option<Vec<u8>>,
}

impl TsPacket {
    pub fn parse(data: &[u8]) -> Result<Self, TsError> {
        if data.len() < TS_PACKET_SIZE {
            return Err(TsError::TooShort(data.len()));
        }
        if data[0] != SYNC_BYTE {
            return Err(TsError::SyncError(data[0]));
        }

        let header = TsHeader {
            sync_byte: data[0],
            transport_error: (data[1] & 0x80) != 0,
            payload_unit_start: (data[1] & 0x40) != 0,
            transport_priority: (data[1] & 0x20) != 0,
            pid: ((data[1] as u16 & 0x1F) << 8) | data[2] as u16,
            scrambling_control: (data[3] >> 6) & 0x03,
            adaptation_field_control: (data[3] >> 4) & 0x03,
            continuity_counter: data[3] & 0x0F,
        };

        let mut offset = 4usize;
        let mut adaptation = None;

        // adaptation_field_control: 10 or 11 → adaptation field 있음
        if header.adaptation_field_control >= 2 {
            let af_length = data[offset] as usize;
            offset += 1;

            if af_length > 0 && offset + af_length <= TS_PACKET_SIZE {
                let flags = data[offset];
                let mut af = AdaptationField {
                    length: af_length as u8,
                    discontinuity: (flags & 0x80) != 0,
                    random_access: (flags & 0x40) != 0,
                    pcr: None,
                    opcr: None,
                    splice_countdown: None,
                };

                let mut af_offset = offset + 1;

                // PCR (6 bytes)
                if (flags & 0x10) != 0 && af_offset + 6 <= offset + af_length {
                    let base = ((data[af_offset] as u64) << 25)
                        | ((data[af_offset + 1] as u64) << 17)
                        | ((data[af_offset + 2] as u64) << 9)
                        | ((data[af_offset + 3] as u64) << 1)
                        | ((data[af_offset + 4] as u64) >> 7);
                    let ext = (((data[af_offset + 4] & 0x01) as u64) << 8)
                        | data[af_offset + 5] as u64;
                    af.pcr = Some(base * 300 + ext);
                    af_offset += 6;
                }

                // OPCR (6 bytes)
                if (flags & 0x08) != 0 && af_offset + 6 <= offset + af_length {
                    // 같은 구조, 필요 시 파싱
                    af_offset += 6;
                }

                // Splice countdown (1 byte)
                if (flags & 0x04) != 0 && af_offset < offset + af_length {
                    af.splice_countdown = Some(data[af_offset] as i8);
                }

                adaptation = Some(af);
            }
            offset = 4 + 1 + af_length; // header + af_length_byte + af_data
        }

        // payload: adaptation_field_control 01 or 11
        let payload = if header.adaptation_field_control & 0x01 != 0
            && offset < TS_PACKET_SIZE
        {
            Some(data[offset..TS_PACKET_SIZE].to_vec())
        } else {
            None
        };

        Ok(TsPacket {
            header,
            adaptation,
            payload,
        })
    }
}
```

### ts-core/src/pid.rs

```rust
use serde::Serialize;
use std::collections::HashMap;

/// 잘 알려진 PID 상수
pub const PID_PAT: u16 = 0x0000;
pub const PID_CAT: u16 = 0x0001;
pub const PID_TSDT: u16 = 0x0002;
pub const PID_NIT: u16 = 0x0010;
pub const PID_SDT: u16 = 0x0011;
pub const PID_EIT: u16 = 0x0012;
pub const PID_NULL: u16 = 0x1FFF;

#[derive(Debug, Clone, Serialize)]
pub struct PidInfo {
    pub pid: u16,
    pub label: String,
    pub stream_type: Option<u8>,
    pub packet_count: u64,
    pub cc_errors: u64,
    pub last_cc: u8,
    pub bitrate_bps: f64,
    pub has_pcr: bool,
    pub scrambled: bool,
}

#[derive(Debug, Default)]
pub struct PidMap {
    pub pids: HashMap<u16, PidInfo>,
    pub total_packets: u64,
}

impl PidMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, pid: u16, cc: u8, scrambled: bool, has_pcr: bool) {
        self.total_packets += 1;

        let info = self.pids.entry(pid).or_insert_with(|| PidInfo {
            pid,
            label: Self::default_label(pid),
            stream_type: None,
            packet_count: 0,
            cc_errors: 0,
            last_cc: cc.wrapping_sub(1) & 0x0F, // 첫 패킷은 에러 안 냄
            bitrate_bps: 0.0,
            has_pcr: false,
            scrambled: false,
        });

        info.packet_count += 1;
        info.scrambled = scrambled;
        if has_pcr {
            info.has_pcr = true;
        }

        // CC 에러 감지 (같은 페이로드가 없는 경우 제외)
        let expected = (info.last_cc + 1) & 0x0F;
        if cc != expected && info.packet_count > 1 {
            info.cc_errors += 1;
        }
        info.last_cc = cc;
    }

    fn default_label(pid: u16) -> String {
        match pid {
            PID_PAT => "PAT".into(),
            PID_CAT => "CAT".into(),
            PID_TSDT => "TSDT".into(),
            PID_NIT => "NIT/ST".into(),
            PID_SDT => "SDT/BAT/ST".into(),
            PID_EIT => "EIT/ST".into(),
            PID_NULL => "Null".into(),
            0x0003..=0x000F => "Reserved".into(),
            _ => format!("PID 0x{:04X}", pid),
        }
    }
}
```

### ts-server/src/main.rs

```rust
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

mod api;
mod ws;
mod ingest;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let (tx, _) = broadcast::channel::<String>(256);

    let state = Arc::new(AppState {
        analyzer: RwLock::new(ts_analyzer::StreamAnalyzer::new()),
        ws_tx: tx,
    });

    let api_routes = Router::new()
        .route("/stream", get(api::stream::get_stream_info))
        .route("/pids", get(api::pid::get_pid_map))
        .route("/pids/{pid}", get(api::pid::get_pid_detail))
        .route("/analyze", post(api::analyze::start_analysis));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/ws", get(ws::realtime::ws_handler))
        // Svelte 빌드 결과물 서빙 (dashboard/build)
        .fallback_service(ServeDir::new("../dashboard/build"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:3200";
    tracing::info!("server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### ts-server/src/ws/realtime.rs

```rust
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
};
use std::sync::Arc;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();

    // 클라이언트에게 실시간 데이터 푸시
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            break; // 연결 끊김
        }
    }
}
```

### ts-server/src/state.rs

```rust
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub struct AppState {
    pub analyzer: RwLock<ts_analyzer::StreamAnalyzer>,
    pub ws_tx: broadcast::Sender<String>,
}
```

---

## Svelte 5 대시보드 상세

### dashboard/package.json

```json
{
  "name": "ts-dashboard",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev --port 5200",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.5.0",
    "vite": "^6.0.0"
  },
  "dependencies": {
    "chart.js": "^4.4.0"
  }
}
```

### dashboard/vite.config.ts

```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': 'http://localhost:3200',
      '/ws': {
        target: 'ws://localhost:3200',
        ws: true,
      },
    },
  },
});
```

### dashboard/src/lib/ws.ts

```typescript
import type { RealtimeData } from './types';

export function createWsConnection(
  onMessage: (data: RealtimeData) => void,
) {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const ws = new WebSocket(`${protocol}//${location.host}/ws`);

  ws.onmessage = (event) => {
    try {
      const data: RealtimeData = JSON.parse(event.data);
      onMessage(data);
    } catch (e) {
      console.error('ws parse error', e);
    }
  };

  ws.onclose = () => {
    // 재연결
    setTimeout(() => createWsConnection(onMessage), 2000);
  };

  return ws;
}
```

### dashboard/src/lib/stores/stream.svelte.ts

```typescript
// Svelte 5 rune 문법
import type { StreamInfo, PidInfo, RealtimeData } from '../types';

// 스트림 전체 정보
let streamInfo = $state<StreamInfo | null>(null);

// PID 맵
let pids = $state<PidInfo[]>([]);

// 실시간 메트릭 (시계열)
let bitrateHistory = $state<{ ts: number; bps: number }[]>([]);
let pcrJitter = $state<{ ts: number; jitter_ms: number }[]>([]);

// CC 에러 로그
let ccErrors = $state<{ ts: number; pid: number; expected: number; got: number }[]>([]);

export function getStreamStore() {
  return {
    get streamInfo() { return streamInfo; },
    get pids() { return pids; },
    get bitrateHistory() { return bitrateHistory; },
    get pcrJitter() { return pcrJitter; },
    get ccErrors() { return ccErrors; },

    setStreamInfo(info: StreamInfo) {
      streamInfo = info;
    },

    updatePids(newPids: PidInfo[]) {
      pids = newPids;
    },

    pushRealtime(data: RealtimeData) {
      const now = Date.now();

      if (data.total_bitrate_bps !== undefined) {
        bitrateHistory = [
          ...bitrateHistory.slice(-299),  // 최근 300 포인트 유지
          { ts: now, bps: data.total_bitrate_bps },
        ];
      }

      if (data.pcr_jitter_ms !== undefined) {
        pcrJitter = [
          ...pcrJitter.slice(-299),
          { ts: now, jitter_ms: data.pcr_jitter_ms },
        ];
      }

      if (data.cc_error) {
        ccErrors = [
          ...ccErrors.slice(-999),
          { ts: now, ...data.cc_error },
        ];
      }

      if (data.pids) {
        pids = data.pids;
      }
    },
  };
}
```

### dashboard/src/lib/types/index.ts

```typescript
export interface StreamInfo {
  filename: string;
  duration_ms: number | null;
  total_packets: number;
  total_bitrate_bps: number;
  programs: ProgramInfo[];
}

export interface ProgramInfo {
  program_number: number;
  pmt_pid: number;
  streams: ElementaryStreamInfo[];
}

export interface ElementaryStreamInfo {
  pid: number;
  stream_type: number;
  stream_type_name: string;
  codec: string | null;
}

export interface PidInfo {
  pid: number;
  label: string;
  stream_type: number | null;
  packet_count: number;
  cc_errors: number;
  bitrate_bps: number;
  has_pcr: boolean;
  scrambled: boolean;
  percentage: number;  // 전체 대비 비율
}

export interface RealtimeData {
  total_bitrate_bps?: number;
  pcr_jitter_ms?: number;
  cc_error?: {
    pid: number;
    expected: number;
    got: number;
  };
  pids?: PidInfo[];
}

export interface Scte35Event {
  pts: number;
  command_type: string;
  splice_event_id: number;
  duration_ms: number | null;
  out_of_network: boolean;
}
```

---

## API 엔드포인트 설계

| Method | Path | 설명 |
|--------|------|------|
| GET | /api/stream | 스트림 전체 정보 (PAT/PMT 기반) |
| GET | /api/pids | PID 맵 (전체) |
| GET | /api/pids/:pid | PID 상세 (패킷 수, CC, 비트레이트) |
| POST | /api/analyze | 파일 분석 시작 (multipart upload) |
| GET | /api/scte35 | SCTE-35 이벤트 로그 |
| GET | /api/pcr | PCR 타임라인 데이터 |
| WS | /ws | 실시간 데이터 스트림 |

---

## 개발 워크플로우

```bash
# 터미널 1: Rust 서버 (watch 모드)
cd crates/ts-server
cargo watch -x run

# 터미널 2: Svelte dev 서버 (HMR)
cd dashboard
npm run dev

# Svelte dev 서버가 /api, /ws를 Rust 서버로 프록시
# 개발: http://localhost:5200
# 프로덕션: cargo로 빌드된 바이너리가 Svelte 빌드 결과물도 서빙
```

### 프로덕션 빌드

```bash
# 1. 대시보드 빌드
cd dashboard && npm run build

# 2. Rust 서버 빌드 (대시보드 정적 파일 포함)
cd .. && cargo build --release -p ts-server

# 단일 바이너리 + dashboard/build/ 폴더로 배포
```

---

## 향후 확장

- **ts-napi**: 같은 ts-core를 NestJS(ACS)에서 napi-rs로 활용
- **ts-wasm**: ts-core를 wasm32 타겟으로 빌드 → 브라우저 내 파싱
- **ts-swift**: ts-core를 staticlib으로 빌드 → macOS 비디오 플레이어에 FFI 연동
- **UDP 멀티캐스트 수신**: 라이브 TS 스트림 실시간 분석
