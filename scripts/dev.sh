#!/bin/bash
# 개발 모드: Rust 서버 (debug) + Svelte dev 서버
# Rust 서버: localhost:3200
# Svelte dev: localhost:5200 (API는 3200으로 프록시)

cd "$(dirname "$0")/.."

cleanup() {
    echo ""
    echo "shutting down..."
    kill $SERVER_PID $DASHBOARD_PID 2>/dev/null
    wait $SERVER_PID $DASHBOARD_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM

echo "=== building server (debug) ==="
cargo build -p ts-server || exit 1

echo "=== starting server on :3200 ==="
cargo run -p ts-server &
SERVER_PID=$!

echo "=== starting dashboard on :5200 ==="
cd dashboard && npm run dev &
DASHBOARD_PID=$!

echo ""
echo "server:    http://localhost:3200"
echo "dashboard: http://localhost:5200"
echo "press Ctrl+C to stop"
echo ""

wait
