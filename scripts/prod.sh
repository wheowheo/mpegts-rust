#!/bin/bash
# 프로덕션 모드: release 빌드 + dashboard static 서빙
# localhost:3200 에서 모든 것을 서빙

cd "$(dirname "$0")/.."

echo "=== building server (release) ==="
cargo build -p ts-server --release || exit 1

echo "=== building dashboard ==="
cd dashboard && npm run build || exit 1
cd ..

echo "=== starting server on :3200 ==="
echo "open http://localhost:3200"
echo ""
./target/release/ts-server
