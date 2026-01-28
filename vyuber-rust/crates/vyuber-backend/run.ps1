# VYuber Backend Runner with Infisical
# Usage: .\run.ps1

Write-Host "Building backend..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Starting backend with Infisical..." -ForegroundColor Green
    infisical run -- cargo run --release
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
