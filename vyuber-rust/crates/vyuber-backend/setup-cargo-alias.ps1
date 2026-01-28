# Setup script to create a cargo wrapper that automatically uses Infisical
# Run this once: .\setup-cargo-alias.ps1

Write-Host "Setting up cargo wrapper for Infisical..." -ForegroundColor Cyan

# PowerShellプロファイルのパス
$profilePath = $PROFILE

# プロファイルが存在しない場合は作成
if (!(Test-Path -Path $profilePath)) {
    New-Item -ItemType File -Path $profilePath -Force
    Write-Host "Created PowerShell profile at: $profilePath" -ForegroundColor Green
}

# 追加する内容
$functionContent = @'

# VYuber Cargo wrapper - automatically uses Infisical for vyuber-backend
function cargo {
    $currentDir = (Get-Location).Path
    if ($currentDir -like "*vyuber*backend*" -and $args[0] -eq "run") {
        Write-Host "🔐 Using Infisical for environment variables..." -ForegroundColor Cyan
        infisical run -- cargo.exe $args
    } else {
        cargo.exe $args
    }
}
'@

# すでに追加されているか確認
$currentContent = Get-Content -Path $profilePath -Raw -ErrorAction SilentlyContinue
if ($currentContent -notlike "*VYuber Cargo wrapper*") {
    Add-Content -Path $profilePath -Value $functionContent
    Write-Host "✅ Added cargo wrapper to PowerShell profile" -ForegroundColor Green
    Write-Host ""
    Write-Host "Please restart your PowerShell or run:" -ForegroundColor Yellow
    Write-Host "  . `$PROFILE" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "After that, just run 'cargo run --release' in vyuber-backend directory!" -ForegroundColor Green
} else {
    Write-Host "ℹ️  Cargo wrapper already exists in profile" -ForegroundColor Yellow
}
