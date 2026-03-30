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

## Phase 7 - ts-core: 패킷 생성 및 먹싱 [done]
- packet_builder.rs: TS 패킷 생성 (header + adaptation field + payload 조립)
- psi_builder.rs: PAT/PMT 섹션 생성 (CRC32 포함)
- muxer.rs: 여러 PID 스트림을 하나의 TS 스트림으로 먹싱
- null stuffing, PCR 삽입
- 단위 테스트: round-trip 검증

## Phase 8 - ts-server: UDP/RTP 멀티캐스트 송출 [done]
- output/udp.rs: UDP 멀티캐스트 송출
- output/rtp.rs: RTP 멀티캐스트 송출 (RFC 2250)
- output/pacer.rs: CBR 페이싱
- output/session.rs: 송출 세션 관리
- api/output.rs: 송출 제어 REST API

## Phase 9 - 송출 품질 모니터링 엔진 [done]
- output_stats.rs: 비트레이트, 지터, PCR 드리프트 수집
- system_stats.rs: CPU/메모리 모니터링
- capacity.rs: 여유 스트림 수 추정
- api/system.rs: 시스템 통계 API

## Phase 10 - Dashboard: 송출 모니터링 UI [done]
- OutputControl, TxBitrateChart, SystemLoad 컴포넌트
- /output 페이지

## Phase 11 - 다중 스트림 및 안정화 [done]
- OutputSessionManager: 세션 ID 기반 다중 동시 관리
- 비트레이트 이탈 자동 경고

## Phase 12 - Dashboard: 계측기 수준 UI 리디자인 [done]
- 방송 계측기 스타일 전체 테마
- 7-segment 디스플레이, LED 인디케이터, 오실로스코프 차트
- PID 상세 파서 (transport/adaptation/PCR/PES/descriptor)
- Dolby 전체 지원 (AC-3, E-AC-3, Atmos JOC, AC-4, Vision, TrueHD)

---

## Phase 13 - HEX 에디터 뷰
- 백엔드: 패킷 raw bytes 저장 (PID별 최근 N개 패킷 원본 보관)
- API: GET /api/pids/:pid/packets (offset, limit)
- HEX 뷰어 컴포넌트: 주소 | hex dump | ASCII
- 영역별 컬러 하이라이팅
  - sync byte (0x47): 빨강
  - TS header (byte 1-3): 시안
  - adaptation field: 노랑
  - PCR 6 bytes: 마젠타
  - PES start code (00 00 01): 그린
  - PES header: 라임
  - payload: 기본색
- 마우스 호버 시 해당 필드 이름/값 툴팁 표시
- PID 상세 페이지에서 클릭 시 해당 패킷 HEX 뷰 열기
- PSI 섹션도 HEX 뷰 지원 (table_id, section_length, CRC 하이라이팅)

## Phase 14 - 프레임 레벨 디코딩 엔진
- crates/ts-decoder 신규 crate 생성
- NAL unit 파서: H.264 (AnnexB, start code 탐색)
  - SPS 파싱: profile, level, resolution, frame_mbs, chroma_format
  - PPS 파싱: entropy_coding_mode, num_ref_frames
  - slice header: slice_type (I/P/B), frame_num, POC
- NAL unit 파서: H.265/HEVC
  - VPS 파싱: max_layers, max_sub_layers, general_profile
  - SPS 파싱: chroma_format, resolution, bit_depth, log2_max_poc
  - PPS 파싱: tiles, WPP, deblocking
  - slice segment header: slice_type, POC, reference picture set
- AC-3/E-AC-3 프레임 파서
  - syncinfo: fscod, frmsizecod, bsid
  - BSI: bsmod, acmod, lfeon, dialnorm, compre
  - Atmos JOC 메타데이터 감지
- AAC 프레임 파서 (ADTS header)
  - profile, sampling_frequency_index, channel_configuration
  - frame_length, number_of_raw_data_blocks
- 프레임 인덱스 구축: packet_index → frame 매핑
- API: GET /api/pids/:pid/frames (frame list with type, size, PTS, DTS)
- API: GET /api/pids/:pid/frames/:idx (개별 프레임 상세)

## Phase 15 - 프레임 정보 대시보드
- 프레임 타임라인 뷰: I/P/B 프레임 시퀀스 시각화
  - I 프레임: 큰 빨강 블록, P: 중간 파랑, B: 작은 초록
  - GOP 구조 시각화 (GOP 경계선, IDR 마커)
  - 프레임 크기 바 차트 (프레임별 byte size)
- 프레임 상세 패널
  - 비디오: slice_type, QP, reference frames, POC, display order
  - 오디오: 채널 레이아웃, 샘플레이트, 비트레이트, 다이얼로그 노멀라이제이션
- SPS/PPS/VPS 뷰어: 모든 필드 계측기 스타일 표시
- PTS/DTS 그래프: 프레임별 타이밍 + AV sync 차이
- 프레임 간격 분석: frame duration 일정성 검증

## Phase 16 - 프록시 썸네일 디코더
- ffmpeg 연동 (Command 또는 FFI)
  - I-frame only 디코딩 → JPEG/WebP 썸네일 추출
  - 지정 간격 (예: 1초마다) 썸네일 생성
  - 해상도: 320px 폭 고정 (프록시 용도)
- API: GET /api/pids/:pid/thumbnails (thumbnail list)
- API: GET /api/pids/:pid/thumbnail/:idx (개별 이미지)
- 썸네일 스트립 뷰: 타임라인에 썸네일 나열
- 프레임 클릭 시 해당 위치 썸네일 표시
- 라이브 스트림: 주기적 snapshot 캡처

## Phase 17 - 라이브 스트림 입력 (실시간 계측)
- ingest/udp.rs 본격 구현
  - UDP 멀티캐스트 수신 (igmp join)
  - RTP 디캡슐레이션 (RTP header strip → TS 패킷 추출)
  - SRT 수신 지원 (libsrt FFI 또는 srt-rs)
- ingest/http.rs: HLS/DASH 폴링 수신
- 스트림 주소 입력 UI
  - udp://239.x.x.x:port
  - rtp://239.x.x.x:port
  - srt://host:port
  - http(s)://url (HLS m3u8)
- API: POST /api/ingest/start (url, protocol)
- API: POST /api/ingest/stop
- API: GET /api/ingest/status
- 실시간 분석 파이프라인: 수신 → 파싱 → 분석 → WebSocket push
- 대시보드: 실시간 갱신 (비트레이트, PID, CC 에러, PCR 지터 연속 모니터링)
- 스트림 전환 시 자동 PAT/PMT 재감지

## Phase 18 - TR 101 290 계측 (Priority 1/2/3)
- Priority 1 (필수 모니터링 - 서비스 불가 수준)
  - TS sync loss: 연속 sync byte 실패 감지
  - Sync byte error: 0x47 아닌 바이트
  - PAT error: PAT 미수신 또는 interval > 500ms
  - PAT error 2: scrambled PAT
  - CC error: continuity counter 불연속
  - PMT error: PMT 미수신 또는 interval 초과
  - PID error: PID 소실 (있던 PID가 사라짐)
- Priority 2 (권장 모니터링 - 품질 저하)
  - Transport error: TEI bit 감지
  - CRC error: PSI 테이블 CRC32 불일치
  - PCR error: PCR 미수신 interval > 100ms
  - PCR accuracy error: PCR 정확도 ±500ns 초과
  - PCR repetition error: PCR 반복 간격 > 40ms
  - PTS error: PTS 미수신
  - CAT error: CAT 오류 (스크램블 스트림 시)
- Priority 3 (정보성 모니터링)
  - NIT error: NIT 미수신
  - NIT actual: NIT 실제 네트워크 interval
  - SI repetition error: SDT/EIT/TDT 반복 간격
  - Unreferenced PID: PAT/PMT에 없는 PID 존재
  - SDT error: SDT 미수신
  - EIT error: EIT 미수신
  - RST error: RST 오류
  - TDT error: TDT 미수신
- 에러 카운터 대시보드: Priority별 그룹핑
  - P1: 빨강 배경 경고, 즉시 알림
  - P2: 노랑 배경 경고
  - P3: 정보성 파랑 표시
- 에러 히스토리 타임라인: 시간축 에러 발생 이력
- 임계값 설정 UI: 각 항목별 경고 임계값 커스터마이즈
- 에러 로그 CSV/JSON 내보내기

## Phase 19 - CMA-1820 수준 UI 통합
- 메인 대시보드 재구성: 멀티 패널 레이아웃
  - 좌측: 스트림 셀렉터 (파일 목록 / 라이브 입력 목록)
  - 중앙 상단: 오실로스코프 (비트레이트 + PCR 지터 듀얼 축)
  - 중앙 하단: PID 점유율 스택 바 차트 (실시간 갱신)
  - 우측 상단: 스트림 요약 패널 (비트레이트, 패킷수, 에러율)
  - 우측 하단: TR 101 290 에러 요약 (P1/P2/P3 카운터)
- 탭 기반 상세 뷰
  - [TRANSPORT] TS 헤더 통계, 패킷 에러율, sync 상태
  - [PROGRAMS] PSI 트리 (PAT → PMT → ES), SDT 서비스명
  - [PIDS] PID 테이블 + 점유율 파이 차트
  - [TIMING] PCR/PTS/DTS 그래프, AV sync, 프레임 간격
  - [VIDEO] 프레임 타임라인, GOP 구조, SPS/PPS, 썸네일 스트립
  - [AUDIO] 오디오 프레임 분석, 채널 레이아웃, Dolby 메타데이터
  - [ERRORS] TR 101 290 전체 계측 대시보드
  - [HEX] 패킷 HEX 에디터 뷰
  - [OUTPUT] 송출 제어 및 모니터링
- 다중 스트림 비교 모드: 2개 스트림 나란히 분석
- 알람 시스템
  - 에러 발생 시 화면 깜빡임 + 사운드 알림
  - 에러 이력 패널 (시간순 정렬, 필터링)
  - 이메일/웹훅 알림 설정
- 세션 저장/불러오기: 분석 결과 스냅샷 저장
- 보고서 생성: PDF/HTML 리포트 (분석 요약, 에러 목록, 차트 포함)

## Phase 20 - 경량 로컬 데이터베이스 (히스토리 관리)
- SQLite 탑재 (rusqlite 또는 sqlx + SQLite)
- 스키마 설계
  - sessions: 분석 세션 (id, filename/url, start_time, end_time, duration, total_packets, bitrate)
  - pid_snapshots: 세션별 PID 통계 스냅샷 (session_id, pid, label, stream_type, packets, cc_errors, bitrate)
  - errors: 에러 이벤트 (session_id, timestamp, type, priority, pid, detail)
  - pcr_history: PCR 샘플 시계열 (session_id, pid, packet_index, pcr_value, jitter_ms)
  - bitrate_history: 비트레이트 시계열 (session_id, timestamp, total_bps, per_pid JSON)
  - alerts: 알림 이력 (session_id, timestamp, level, message, acknowledged)
  - output_logs: 송출 세션 이력 (session_id, config, start, end, packets_sent, avg_bitrate)
- 자동 기록
  - 파일 분석 완료 시 세션 + PID 스냅샷 자동 저장
  - 라이브 모니터링 시 1초 간격 bitrate/pcr 시계열 기록
  - TR 101 290 에러 발생 시 즉시 기록
  - 송출 start/stop 이벤트 기록
- API
  - GET /api/history/sessions (목록, 필터, 페이징)
  - GET /api/history/sessions/:id (세션 상세 + PID + 에러)
  - GET /api/history/sessions/:id/bitrate (비트레이트 시계열)
  - GET /api/history/sessions/:id/errors (에러 목록, type/priority 필터)
  - DELETE /api/history/sessions/:id
  - GET /api/history/stats (전체 통계: 총 세션수, 총 에러수, 가동 시간)
- Dashboard
  - /history 페이지: 세션 목록 테이블 (날짜, 파일명, 비트레이트, 에러 수)
  - 세션 클릭 → 과거 분석 결과 재열람
  - 에러 트렌드 차트: 일별/주별 에러 발생 추이
  - 비트레이트 히스토리 차트: 장기 시계열 (시간대별 평균/최대)
  - 세션 비교: 2개 세션 나란히 diff 뷰
  - 데이터 보존 정책: 설정 가능한 retention (기본 30일, 오래된 자동 삭제)
- DB 마이그레이션: 버전별 스키마 업그레이드 (embedded migrations)
- 단일 바이너리: SQLite 파일 하나로 portable (data/ts-engine.db)

## Phase 21 - 성능 최적화 및 프로덕션
- Rust 멀티스레드 분석 파이프라인 (rayon 또는 tokio 병렬)
- 대용량 파일 랜덤 액세스 (mmap 기반)
- WebSocket 메시지 압축 (MessagePack 또는 CBOR)
- Dashboard 가상 스크롤 (대량 PID/프레임 목록)
- 도커 이미지 빌드 (multi-stage)
- CI/CD: GitHub Actions (cargo test + npm build + docker push)
- 크로스 컴파일: linux-x86_64, linux-aarch64 (방송 장비 ARM 지원)
