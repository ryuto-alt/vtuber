#!/bin/bash
# VYuber Rust開発サーバー起動スクリプト

set -e

cd "$(dirname "$0")/.."

# 既存のプロセスを停止
if pgrep -f "vyuber-backend" > /dev/null; then
    echo "🛑 Stopping existing vyuber-backend process(es)..."
    pkill -f "vyuber-backend" || true
    sleep 0.5
    echo "✅ Stopped"
    echo ""
fi

echo "🚀 Starting VYuber Rust backend..."
echo ""

npx @infisical/cli run -- cargo run --release
