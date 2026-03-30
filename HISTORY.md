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

## Phase 11 - 다중 스트림 및 안정화
- OutputSessionManager: 세션 ID 기반 다중 송출 동시 관리
- 세션별 독립 start/stop/status
- API 확장: POST start, POST stop/{id}, POST stop (all), GET list, GET {id}
- 자동 경고: 비트레이트 이탈 5% warning, 15% critical
- Dashboard: 다중 세션 목록, 세션별 경고 표시, 전체 요약 카드

## Phase 12 - Dashboard: 계측기 수준 UI 리디자인
- 방송 계측기 스타일 전체 테마
- 7-segment 디스플레이, LED 인디케이터
- PID 상세 파서 (transport/adaptation/PCR/PES/descriptor)
- Dolby 전체 지원

## Phase 13 - HEX 에디터 뷰
- PID별 최근 64개 패킷 원본 저장 (StreamAnalyzer)
- GET /api/pids/:pid/packets (offset, limit)
- HexViewer 컴포넌트: hex dump + ASCII + 컬러 하이라이팅
- sync=빨강, header=시안, AF=노랑, PCR=마젠타, PES=그린
- 마우스 호버 시 필드 이름/값 툴팁

## Phase 14 - 프레임 레벨 디코딩 엔진
- ts-decoder crate 신규 생성
- H.264 NAL 파서: SPS, PPS, slice header 파싱
- H.265/HEVC NAL 파서: VPS, SPS, PPS, slice header
- AC-3/E-AC-3 프레임 파서 (Atmos JOC 감지)
- AAC ADTS 프레임 파서
- BitReader: Exp-Golomb 디코딩
- FrameIndexer: PES → frame 인덱스 구축
- API: GET /api/pids/:pid/frames, /frames/:idx

## Phase 15 - 프레임 정보 대시보드
- FrameTimeline 컴포넌트: I/P/B 시퀀스 시각화
- GOP 경계 마커, 프레임 크기 바 차트
- 프레임 상세 패널 (코덱, 해상도, 프로파일, 타이밍)
- PTS/DTS 타이밍 테이블
- 오디오 프레임 분석 (채널 레이아웃, Atmos)

## Phase 16 - 프록시 썸네일 디코더
- ThumbnailExtractor: I-frame 기반 캡처 인프라
- API: GET /api/pids/:pid/thumbnails, /thumbnail/:idx
- ThumbnailStrip 컴포넌트
- 실제 디코딩은 ffmpeg 라이브러리 필요 (placeholder)

## Phase 17 - 라이브 스트림 입력
- UDP 멀티캐스트 수신 (IGMP join)
- RTP 디캡슐레이션 (헤더 스트리핑, 확장 처리)
- HTTP/HLS 폴링 수신 (reqwest)
- ingest API: POST start/stop, GET status
- IngestControl 컴포넌트 (URL 입력, 프리셋, 프로토콜 자동 감지)
- watch 채널 기반 graceful stop

## Phase 18 - TR 101 290 계측
- P1: sync loss, sync byte error, PAT/PMT/CC/PID error
- P2: TEI, CRC, PCR interval/accuracy/repetition error
- P3: NIT/SDT interval, unreferenced PID
- Tr101290Checker: StreamAnalyzer 파이프라인 통합
- API: GET /api/tr101290, /tr101290/errors
- Tr101290Dashboard 컴포넌트: 우선순위별 카운터, 에러 로그
- /errors 페이지

## Phase 19 - CMA-1820 수준 UI 통합
- 사이드바: 파일 입력, 라이브 입력, TR 101 290 미니 요약
- 탭 기반 상세 뷰: TRANSPORT, PROGRAMS, PIDS, TIMING, ERRORS, OUTPUT
- 듀얼 축 오실로스코프 (비트레이트 + PCR 지터)
- 반응형 그리드 레이아웃

## Phase 20 - 경량 로컬 데이터베이스
- rusqlite + bundled SQLite, WAL 모드
- 스키마: sessions, pid_snapshots, errors, bitrate_history
- 파일 분석 완료 시 자동 기록
- history API: 세션 CRUD, 비트레이트 시계열, 통계
- 30일 보존 정책 + 자동 정리
- /history 페이지: 세션 목록, 통계 요약

## Phase 21 - 성능 최적화 및 프로덕션
- mmap 기반 대용량 파일 분석
- Dockerfile: 멀티 스테이지 빌드 (Rust + Node + slim)
- GitHub Actions CI: cargo build/test + npm build + docker
- .dockerignore
