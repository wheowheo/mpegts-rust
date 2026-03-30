#!/bin/bash
# 전체 테스트
cd "$(dirname "$0")/.."

echo "=== rust tests ==="
cargo test

echo ""
echo "=== build check (release) ==="
cargo build --release 2>&1 | tail -3

echo ""
echo "=== dashboard build check ==="
cd dashboard && npm run build 2>&1 | tail -3
