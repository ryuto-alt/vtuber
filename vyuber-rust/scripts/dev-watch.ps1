# VYuber Rust開発サーバー起動スクリプト (ホットリロード対応 - Windows PowerShell)

$ErrorActionPreference = "Stop"

# スクリプトのディレクトリから親ディレクトリへ移動
Set-Location (Split-Path -Parent $PSScriptRoot)

# 既存のプロセスを停止
$processes = Get-Process -Name "vyuber-backend" -ErrorAction SilentlyContinue
if ($processes) {
    Write-Host "🛑 Stopping existing vyuber-backend process(es)..." -ForegroundColor Yellow
    $processes | Stop-Process -Force
    Start-Sleep -Milliseconds 500
    Write-Host "✅ Stopped" -ForegroundColor Green
    Write-Host ""
}

# cargo-watchがインストールされているか確認
$cargoWatchInstalled = cargo install --list | Select-String "cargo-watch"
if (-not $cargoWatchInstalled) {
    Write-Host "⚠️  cargo-watch not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-watch
}

Write-Host "🔥 Starting VYuber Rust backend with hot reload..." -ForegroundColor Green
Write-Host "📝 Watching for changes in crates/vyuber-backend/src..." -ForegroundColor Cyan
Write-Host ""

npx "@infisical/cli" run -- cargo watch -x "run --release --bin vyuber-backend" -w crates/vyuber-backend/src
