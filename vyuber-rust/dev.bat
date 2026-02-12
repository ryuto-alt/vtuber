@echo off
chcp 65001 >nul
echo ============================================
echo   AIVID Development Environment (RTX 3060 Ti Tuned)
echo ============================================
echo.

:: このバッチファイルのディレクトリを基準にする
set "ROOT_DIR=%~dp0"

:: ---------------------------------------------------------
:: 0. クリーンアップ（二重起動防止）
:: ---------------------------------------------------------
echo Cleaning up previous processes...
taskkill /F /IM mediamtx.exe >nul 2>&1
taskkill /F /IM vyuber-backend.exe >nul 2>&1

:: ---------------------------------------------------------
:: 1. RTX 3060 Ti 用のビルド設定
:: ---------------------------------------------------------
set CUDAARCHS=86
set CMAKE_CUDA_ARCHITECTURES=86
set CUDAFLAGS=--allow-unsupported-compiler
set CMAKE_CUDA_FLAGS=--allow-unsupported-compiler
set NVCC_PREPEND_FLAGS=--allow-unsupported-compiler

:: CUDAパス設定
if exist "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.1" (
    set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.1"
    set "CUDAToolkit_ROOT=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.1"
    set "PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.1\bin;%PATH%"
)

:: ---------------------------------------------------------
:: 2. ツールチェーン確認
:: ---------------------------------------------------------
rustc --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Rust not found.
    pause
    exit /b 1
)
rustup target add wasm32-unknown-unknown >nul 2>&1

:: Whisper モデルチェック
if not exist "%ROOT_DIR%models\ggml-medium.bin" (
    echo Downloading ggml-medium.bin...
    if not exist "%ROOT_DIR%models" mkdir "%ROOT_DIR%models"
    curl -L -o "%ROOT_DIR%models\ggml-medium.bin" "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
)
set "MODEL_PATH=%ROOT_DIR%models\ggml-medium.bin"

:: ---------------------------------------------------------
:: 3. ビルド & 起動
:: ---------------------------------------------------------

echo.
echo Building frontend...
trunk build
if %ERRORLEVEL% neq 0 (
    echo ERROR: Frontend build failed!
    pause
    exit /b 1
)

echo.
echo Building backend (CUDA ENABLED)...
:: ★重要: ここで --features cuda を追加！
cargo build --bin vyuber-backend --features cuda
if %ERRORLEVEL% neq 0 (
    echo ERROR: Backend build failed!
    pause
    exit /b 1
)

echo.
echo Starting backend server...
set "WHISPER_MODEL_PATH=%MODEL_PATH%"
set "RUST_BACKTRACE=1"

:: infisical経由で実行（target内のexeを直接叩く）
start "AIVID Backend" cmd /k "cd /d %ROOT_DIR% && echo Starting Backend... && infisical run -- target\debug\vyuber-backend.exe || (echo. && echo ===== BACKEND CRASHED ===== && pause)"

echo Waiting for backend server...
:wait_loop
timeout /t 2 /nobreak >nul
curl -s -o nul http://localhost:3000
if %ERRORLEVEL% equ 0 goto server_ready
goto wait_loop

:server_ready
echo Backend server is ready!

echo.
echo Starting Tauri...
cd /d "%ROOT_DIR%crates\vyuber-desktop"
cargo tauri dev
cd /d "%ROOT_DIR%"

:: 終了時クリーンアップ
taskkill /F /IM mediamtx.exe >nul 2>&1
taskkill /F /IM vyuber-backend.exe >nul 2>&1
pause