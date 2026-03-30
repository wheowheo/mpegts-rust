# CLAUDE.md - 프로젝트 지침

## 프로젝트
MPEG-TS Engine + Dashboard: Rust 백엔드(ts-core, ts-analyzer, ts-server) + Svelte 5 프론트엔드

## 커밋 규칙
- 페이즈별로 커밋한다
- 커밋 메시지는 자연스러운 개발자 스타일로 작성한다
- AI가 작성했다는 티가 나지 않도록 한다
- Co-Authored-By 태그를 넣지 않는다
- 커밋 메시지는 간결하게, 한글 또는 영어로 통일 (영어 선호)
- 커밋 메시지에 이모지를 사용하지 않는다

## 개발 규칙
- 설계서(ts-engine-dashboard-ARCHITECTURE.md)를 기준으로 구현한다
- PLAN.md의 페이즈 순서를 따른다
- 각 페이즈 완료 시 HISTORY.md에 기록한다
- 불필요한 주석, docstring을 남발하지 않는다
- 과도한 에러 핸들링이나 추상화를 하지 않는다

## 빌드 & 테스트
- `cargo build` / `cargo test` (Rust)
- `cd dashboard && npm run dev` (Svelte)
- Rust 서버 포트: 3200, Svelte dev 포트: 5200
