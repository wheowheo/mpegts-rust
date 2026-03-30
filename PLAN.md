# Development Plan

## Phase 1 - Cargo Workspace Scaffolding [done]
- workspace root Cargo.toml 생성
- rust-toolchain.toml
- crates/ts-core, crates/ts-analyzer, crates/ts-server 디렉토리 및 Cargo.toml 생성
- 각 crate의 빈 lib.rs / main.rs 생성
- `cargo build` 통과 확인

## Phase 2 - ts-core: MPEG-TS Parser [done]
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

## Phase 3 - ts-analyzer: Analysis Engine [done]
- stream_info.rs: 스트림 전체 정보 집계 (PAT/PMT 기반)
- continuity.rs: CC 에러 감지
- pcr_jitter.rs: PCR 지터/드리프트 분석
- bitrate_stats.rs: PID별 비트레이트 시계열
- StreamAnalyzer 통합 구조체
- lib.rs: 모듈 통합

## Phase 4 - ts-server: Axum Web Server [done]
- state.rs: AppState 정의
- api/stream.rs: GET /api/stream
- api/pid.rs: GET /api/pids, GET /api/pids/:pid
- api/analyze.rs: POST /api/analyze (파일 업로드 → 분석)
- ws/realtime.rs: WebSocket 실시간 데이터 푸시
- ingest/file.rs: 파일 입력 처리
- ingest/udp.rs: UDP 멀티캐스트 수신 (기본 구조)
- main.rs: 라우터 조립, 서버 기동
- `cargo run -p ts-server` 동작 확인

## Phase 5 - Dashboard: Svelte 5 SPA [done]
- SvelteKit 프로젝트 초기화 (adapter-static)
- types/index.ts: API 타입 정의
- lib/api.ts: REST 클라이언트
- lib/ws.ts: WebSocket 클라이언트
- stores/stream.svelte.ts: rune 기반 상태 관리
- components: PidMap, BitrateChart, PcrTimeline, PsiViewer, CcErrors, Scte35Log, StreamSummary
- routes: 메인 대시보드, PID 상세, SCTE-35 로그
- vite proxy 설정

## Phase 6 - Integration & Polish [done]
- Rust warning 정리
- PMT → PidMap 연동 버그 수정
- README.md 작성
- 프로덕션 빌드 검증

---

## Phase 7 - ts-core: 패킷 생성 및 먹싱
- packet_builder.rs: TS 패킷 생성 (header + adaptation field + payload 조립)
- psi_builder.rs: PAT/PMT 섹션 생성 (CRC32 포함)
- pcr_stamper.rs: PCR 삽입 로직 (일정 간격으로 PCR 패킷 생성)
- null_stuffer.rs: CBR 유지를 위한 null 패킷 삽입
- muxer.rs: 여러 PID 스트림을 하나의 TS 스트림으로 먹싱
- 단위 테스트: 생성한 패킷을 다시 파싱해서 round-trip 검증

## Phase 8 - ts-server: UDP/RTP 멀티캐스트 송출
- output/udp.rs: UDP 멀티캐스트 송출 (TS over UDP, 7 packets/datagram)
- output/rtp.rs: RTP 멀티캐스트 송출 (RFC 2250, RTP 헤더 + TS 페이로드)
- output/pacer.rs: CBR 페이싱 (PCR 기반 타이밍, 버스트 방지)
- output/mod.rs: OutputSink trait (UDP/RTP 공통 인터페이스)
- api/output.rs: 송출 제어 API
  - POST /api/output/start (source, dest_ip, dest_port, protocol, bitrate)
  - POST /api/output/stop
  - GET /api/output/status
- state.rs: 송출 세션 상태 추가 (OutputSession)
- 소스 지원: 파일 루프 재생, UDP 수신 → 릴레이

## Phase 9 - 송출 품질 모니터링 엔진
- output_stats.rs: 송출 측 실시간 통계 수집
  - 실제 송출 비트레이트 vs 목표 비트레이트
  - 패킷 송출 간격 지터 (inter-packet jitter)
  - PCR 정확도 (PCR drift from wall clock)
  - 버퍼 점유율 (송출 큐 사용량)
- system_stats.rs: 시스템 리소스 모니터링
  - CPU 사용률 (프로세스 단위)
  - 네트워크 인터페이스 TX 대역폭, 드롭 카운트
  - 메모리 사용량
- capacity.rs: 여유 용량 추정
  - 현재 부하 대비 추가 가능한 스트림 수 추정
  - CPU/네트워크 병목 판단

## Phase 10 - Dashboard: 송출 제어 및 안정성 모니터링 UI
- components/OutputControl.svelte: 송출 시작/정지 폼 (소스, 목적지, 프로토콜, 비트레이트)
- components/OutputStatus.svelte: 현재 송출 세션 상태 카드
- components/TxBitrateChart.svelte: 송출 비트레이트 실시간 차트 (목표 vs 실제)
- components/JitterGauge.svelte: 패킷 간격 지터 게이지
- components/SystemLoad.svelte: CPU/메모리/네트워크 대역폭 게이지
- components/CapacityMeter.svelte: "추가 N개 스트림 가능" 여유도 표시
- routes/output/+page.svelte: 송출 모니터링 페이지
- stores/output.svelte.ts: 송출 상태 rune store
- WebSocket 확장: 송출 통계도 실시간 푸시

## Phase 11 - 다중 스트림 및 안정화
- 다중 송출 세션 동시 관리 (세션 ID 기반)
- 세션별 독립 모니터링
- 자동 경고: CC 에러 급증, 비트레이트 이탈, 지터 임계값 초과
- 전체 동작 테스트 (파일→UDP, 파일→RTP, UDP→UDP 릴레이)

## Phase 12 - Dashboard: 계측기 수준 UI 리디자인
- 전체 테마: 방송 계측기 스타일 다크 UI (짙은 navy/charcoal 배경, 형광 accent)
- 헤더: 장비 모델명 스타일 로고, 연결 상태 LED 인디케이터, 시계
- StreamSummary: 큰 숫자 계기판 스타일, 비트레이트 LED 바 그래프
- PidMap: 컬러 코딩 강화 (video=cyan, audio=green, PSI=yellow, null=gray)
- BitrateChart: 오실로스코프 스타일 그리드, 형광 그린 라인
- PcrTimeline: 지터 범위 게이지 + 임계값 라인
- CcErrors: 에러 카운터 7-segment 디스플레이 스타일
- PsiViewer: 트리 구조 블록 다이어그램 스타일
- Drop zone: 장비 패널 슬롯 스타일
- 반응형 그리드 레이아웃 최적화
