#!/bin/bash
# VYuber Rust開発サーバー起動スクリプト (ホットリロード対応)

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

# cargo-watchがインストールされているか確認
if ! command -v cargo-watch &> /dev/null; then
    echo "⚠️  cargo-watch not found. Installing..."
    cargo install cargo-watch
fi

echo "🔥 Starting VYuber Rust backend with hot reload..."
echo "📝 Watching for changes in crates/vyuber-backend/src..."
echo ""

npx @infisical/cli run -- cargo watch -x "run --release --bin vyuber-backend" -w crates/vyuber-backend/src
