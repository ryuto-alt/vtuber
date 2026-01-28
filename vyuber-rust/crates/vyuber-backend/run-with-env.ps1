#!/usr/bin/env pwsh
# VYuber backend runner with manual environment variables

Write-Host "🚀 Starting VYuber backend..." -ForegroundColor Cyan

# Infisicalから環境変数を取得して設定
try {
    # vyuberディレクトリ（.infisical.jsonがある場所）に移動
    Push-Location "..\..\..\.."

    Write-Host "📦 Loading environment variables from Infisical..." -ForegroundColor Yellow

    # infisical export を使って環境変数を取得
    $envOutput = infisical export --format=dotenv 2>&1

    if ($LASTEXITCODE -eq 0) {
        # 環境変数を解析して設定
        $envOutput -split "`n" | ForEach-Object {
            $line = $_.Trim()
            if ($line -and $line -notmatch '^#') {
                $parts = $line -split '=', 2
                if ($parts.Count -eq 2) {
                    $key = $parts[0].Trim()
                    $value = $parts[1].Trim().Trim('"')
                    [Environment]::SetEnvironmentVariable($key, $value, 'Process')
                    Write-Host "  ✓ Set $key" -ForegroundColor Green
                }
            }
        }
    } else {
        Write-Host "⚠️  Failed to load from Infisical: $envOutput" -ForegroundColor Red
        Write-Host "Please run 'infisical login' first" -ForegroundColor Yellow
        Pop-Location
        exit 1
    }

    Pop-Location

    Write-Host "🔨 Building and running backend..." -ForegroundColor Cyan
    Set-Location $PSScriptRoot
    cargo run --release

} catch {
    Write-Host "❌ Error: $_" -ForegroundColor Red
    Pop-Location
    exit 1
}
