# MPEG-TS Engine + Dashboard

Rust 기반 MPEG-TS 스트림 분석 엔진과 Svelte 5 실시간 대시보드.

## Architecture

```
crates/
  ts-core/       MPEG-TS 파서 코어 (패킷, PSI, PES, PCR, SCTE-35)
  ts-analyzer/   분석 엔진 (CC 에러, PCR 지터, 비트레이트 통계)
  ts-server/     Axum 웹서버 (REST API + WebSocket)
dashboard/       Svelte 5 SPA (실시간 모니터링 UI)
```

## Requirements

- Rust stable
- Node.js 18+

## Build & Run

```bash
# Rust 서버 빌드
cargo build --release

# Dashboard 빌드
cd dashboard && npm install && npm run build

# 서버 실행 (포트 3200, dashboard를 static으로 서빙)
cargo run -p ts-server --release
```

개발 모드:

```bash
# 터미널 1: Rust 서버
cargo run -p ts-server

# 터미널 2: Svelte dev server (포트 5200, API는 3200으로 프록시)
cd dashboard && npm run dev
```

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/stream` | 스트림 전체 정보 |
| GET | `/api/pids` | PID 맵 |
| GET | `/api/pids/:pid` | PID 상세 정보 |
| POST | `/api/analyze` | TS 파일 업로드 및 분석 (multipart) |
| WS | `/ws` | 실시간 분석 데이터 스트림 |

## Test

```bash
cargo test
```
