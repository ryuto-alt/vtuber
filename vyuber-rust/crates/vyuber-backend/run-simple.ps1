#!/usr/bin/env pwsh
# Simple runner for VYuber backend with Infisical

Write-Host "🔐 Starting VYuber backend with Infisical..." -ForegroundColor Cyan
Set-Location $PSScriptRoot
& infisical run -- cargo run --release
