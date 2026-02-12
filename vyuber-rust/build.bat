@echo off
chcp 65001 >nul
echo ============================================
echo   AIVID Production Build
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
echo Building frontend (release)...
trunk build --release
if %ERRORLEVEL% neq 0 (
    echo ERROR: Frontend build failed!
    pause
    exit /b 1
)

:: Tauri ビルド
echo.
echo Building Tauri application...
echo This may take several minutes...
cd crates\vyuber-desktop
cargo tauri build
set BUILD_RESULT=%ERRORLEVEL%
cd ..\..

if %BUILD_RESULT% neq 0 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo ============================================
echo   Build Complete!
echo ============================================
echo.
echo Installers:
echo   target\release\bundle\nsis\AIVID_0.1.0_x64-setup.exe
echo   target\release\bundle\msi\AIVID_0.1.0_x64_en-US.msi
echo.
pause
