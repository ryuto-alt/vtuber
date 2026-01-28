# VYuber Rust開発サーバー起動スクリプト (Windows PowerShell)

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

Write-Host "🚀 Starting VYuber Rust backend..." -ForegroundColor Green
Write-Host ""

npx "@infisical/cli" run -- cargo run --release
