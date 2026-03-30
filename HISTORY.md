# Development History

## Phase 1 - Cargo Workspace Scaffolding
- workspace root Cargo.toml, rust-toolchain.toml 설정
- crates/ts-core, crates/ts-analyzer, crates/ts-server 디렉토리 생성
- 각 crate의 Cargo.toml, lib.rs/main.rs 생성
- `cargo build` 통과 확인

## Phase 2 - ts-core: MPEG-TS Parser
- packet.rs: TS 패킷 파싱 (188B, sync byte, header, adaptation field)
- pid.rs: PID 상수 및 PidInfo/PidMap
- psi/: PAT, PMT, SDT, NIT 파싱
- pes.rs: PES 헤더 파싱
- timing.rs: PCR, PTS, DTS 추출
- scte35.rs: SCTE-35 splice command 파싱
- descriptors.rs: 디스크립터 파싱
- bitrate.rs: 비트레이트 계산
- tests/packet_test.rs: 패킷 파싱 단위 테스트 5건

## Phase 3 - ts-analyzer: Analysis Engine
- stream_info.rs: PAT/PMT 기반 스트림 정보 집계
- continuity.rs: CC 에러 감지
- pcr_jitter.rs: PCR 지터/드리프트 분석
- bitrate_stats.rs: PID별 비트레이트 시계열
- StreamAnalyzer 통합 구조체

## Phase 4 - ts-server: Axum Web Server
- state.rs: AppState (RwLock<StreamAnalyzer> + broadcast channel)
- api/stream.rs: GET /api/stream
- api/pid.rs: GET /api/pids, GET /api/pids/:pid
- api/analyze.rs: POST /api/analyze (multipart 파일 업로드 → 분석)
- ws/realtime.rs: WebSocket 실시간 데이터 푸시
- ingest/file.rs: 파일 입력 처리, analyze_bytes 공통 로직
- ingest/udp.rs: UDP 멀티캐스트 수신 (기본 구조)
- 서버 포트 3200, CORS/Trace 미들웨어, static fallback

## Phase 5 - Dashboard: Svelte 5 SPA
- SvelteKit + adapter-static 프로젝트 구성
- types/index.ts: API 타입 정의
- lib/api.ts: REST 클라이언트, lib/ws.ts: WebSocket 클라이언트
- stores/stream.svelte.ts: Svelte 5 rune 기반 상태 관리
- 컴포넌트: PidMap, BitrateChart, PcrTimeline, PsiViewer, CcErrors, Scte35Log, StreamSummary
- 라우트: 메인 대시보드(/), PID 상세(/pid/[pid]), SCTE-35 로그(/scte35)
- vite proxy 설정 (API → localhost:3200)

## Phase 6 - Integration & Polish
- Rust warning 정리 (analyze.rs 중복 로직 추출, dead_code 처리)
- PMT → PidMap 연동 버그 수정
- README.md 작성
- 프로덕션 빌드 검증

## Phase 7 - ts-core: 패킷 생성 및 먹싱
- crc32.rs: MPEG-2 CRC32 계산
- packet_builder.rs: TS 패킷 조립 (header, adaptation field, PCR, stuffing)
- psi_builder.rs: PAT/PMT 섹션 생성 (CRC32 포함)
- muxer.rs: 다중 PID 먹싱, PCR 삽입, CBR null stuffing
- round-trip 테스트 8건 (빌더 → 파서 검증)

## Phase 8 - ts-server: UDP/RTP 멀티캐스트 송출
- output/udp.rs: UDP 멀티캐스트 송출 (7 pkts/datagram)
- output/rtp.rs: RTP 송출 (RFC 2250, PT=33, SSRC, timestamp)
- output/pacer.rs: CBR 페이싱 (burst 간격 기반)
- output/session.rs: 송출 세션 관리 (start/stop, 파일 루프 재생)
- api/output.rs: POST start/stop, GET status

## Phase 9 - 송출 품질 모니터링 엔진
- output_stats.rs: 송출 비트레이트, 지터, PCR 드리프트 수집
- system_stats.rs: CPU/메모리 모니터링 (macOS/Linux)
- capacity.rs: CPU/네트워크 기반 여유 스트림 수 추정
- api/system.rs: 시스템 통계 + 용량 추정 API

## Phase 10 - Dashboard: 송출 모니터링 UI
- OutputControl.svelte: 송출 시작/정지 폼, 실시간 상태 카드
- TxBitrateChart.svelte: 목표 vs 실제 비트레이트 차트
- SystemLoad.svelte: CPU/메모리 게이지, 여유 스트림 수 표시
- /output 라우트 추가, 네비게이션 연결
