@echo off
chcp 65001 >nul
echo ============================================
echo   AIVID Development Environment
echo ============================================
echo.

:: Rust チェック
rustc --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Rust not found. Installing...
    echo Please install Rust from: https://rustup.rs/
    start https://rustup.rs/
    pause
    exit /b 1
)

:: wasm32 ターゲット
rustup target add wasm32-unknown-unknown >nul 2>&1

:: Trunk インストール
trunk --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Installing Trunk...
    cargo install trunk
)

:: Tauri CLI インストール
cargo tauri --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Installing Tauri CLI...
    cargo install tauri-cli --version "^2"
)

:: フロントエンドビルド
echo.
echo Building frontend...
trunk build
if %ERRORLEVEL% neq 0 (
    echo ERROR: Frontend build failed!
    pause
    exit /b 1
)

:: バックエンドサーバー起動
echo.
echo Starting backend server...
start "AIVID Backend" cmd /c "cargo run --bin vyuber-backend"

echo Waiting for server to start...
timeout /t 5 /nobreak >nul
echo Server ready!

:: Tauri dev 起動
echo.
echo Starting Tauri...
cd crates\vyuber-desktop
cargo tauri dev
cd ..\..

:: クリーンアップ
taskkill /FI "WINDOWTITLE eq AIVID Backend" >nul 2>&1
pause
