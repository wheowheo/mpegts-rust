# Development Plan

## Phase 1 - Cargo Workspace Scaffolding
- workspace root Cargo.toml 생성
- rust-toolchain.toml
- crates/ts-core, crates/ts-analyzer, crates/ts-server 디렉토리 및 Cargo.toml 생성
- 각 crate의 빈 lib.rs / main.rs 생성
- `cargo build` 통과 확인

## Phase 2 - ts-core: MPEG-TS Parser
- packet.rs: TS 패킷 파싱 (188 bytes, sync byte, header, adaptation field)
- pid.rs: PID 상수, PidInfo, PidMap
- psi/pat.rs: PAT 파싱
- psi/pmt.rs: PMT 파싱
- psi/sdt.rs: SDT 파싱
- psi/nit.rs: NIT 파싱 (기본)
- pes.rs: PES 패킷 조립/파싱
- timing.rs: PCR, PTS, DTS 추출
- scte35.rs: SCTE-35 splice command 파싱
- descriptors.rs: 디스크립터 파싱
- bitrate.rs: 비트레이트 계산
- lib.rs: 모듈 re-export
- 단위 테스트 작성

## Phase 3 - ts-analyzer: Analysis Engine
- stream_info.rs: 스트림 전체 정보 집계 (PAT/PMT 기반)
- continuity.rs: CC 에러 감지
- pcr_jitter.rs: PCR 지터/드리프트 분석
- bitrate_stats.rs: PID별 비트레이트 시계열
- StreamAnalyzer 통합 구조체
- lib.rs: 모듈 통합

## Phase 4 - ts-server: Axum Web Server
- state.rs: AppState 정의
- api/stream.rs: GET /api/stream
- api/pid.rs: GET /api/pids, GET /api/pids/:pid
- api/analyze.rs: POST /api/analyze (파일 업로드 → 분석)
- ws/realtime.rs: WebSocket 실시간 데이터 푸시
- ingest/file.rs: 파일 입력 처리
- ingest/udp.rs: UDP 멀티캐스트 수신 (기본 구조)
- main.rs: 라우터 조립, 서버 기동
- `cargo run -p ts-server` 동작 확인

## Phase 5 - Dashboard: Svelte 5 SPA
- SvelteKit 프로젝트 초기화 (adapter-static)
- types/index.ts: API 타입 정의
- lib/api.ts: REST 클라이언트
- lib/ws.ts: WebSocket 클라이언트
- stores/stream.svelte.ts: rune 기반 상태 관리
- components: PidMap, BitrateChart, PcrTimeline, PsiViewer, CcErrors, Scte35Log, StreamSummary
- routes: 메인 대시보드, PID 상세, SCTE-35 로그
- vite proxy 설정

## Phase 6 - Integration & Polish
- 프로덕션 빌드 검증
- README.md 작성
- 전체 동작 확인
