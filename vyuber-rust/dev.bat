@echo off
chcp 65001 >nul
echo ============================================
echo   AIVID Development Environment
echo ============================================
echo.

:: このバッチファイルのディレクトリを基準にする
set "ROOT_DIR=%~dp0"

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

:: CMake チェック
cmake --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo CMake not found. Installing...
    winget install Kitware.CMake --accept-package-agreements --accept-source-agreements
    set "PATH=C:\Program Files\CMake\bin;%PATH%"
)

:: LLVM チェック
if not exist "C:\Program Files\LLVM\bin\libclang.dll" (
    echo LLVM not found. Installing...
    winget install LLVM.LLVM --accept-package-agreements --accept-source-agreements
)
set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"

:: Whisper モデルチェック（tinyを優先、なければbaseを探す）
if exist "%ROOT_DIR%models\ggml-tiny.bin" (
    set "MODEL_PATH=%ROOT_DIR%models\ggml-tiny.bin"
    goto model_ready
)
if exist "%ROOT_DIR%models\ggml-base.bin" (
    set "MODEL_PATH=%ROOT_DIR%models\ggml-base.bin"
    echo Using existing base model.
    goto model_ready
)

:: モデルが無いのでダウンロード
echo.
echo Whisper model not found. Downloading ggml-tiny.bin (75MB)...
if not exist "%ROOT_DIR%models" mkdir "%ROOT_DIR%models"
set "MODEL_PATH=%ROOT_DIR%models\ggml-tiny.bin"
curl -L -o "%MODEL_PATH%" "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
if %ERRORLEVEL% neq 0 (
    echo ERROR: Model download failed!
    pause
    exit /b 1
)
echo Model downloaded.

:model_ready
echo Using model: %MODEL_PATH%

:: フロントエンドビルド
echo.
echo Building frontend...
trunk build
if %ERRORLEVEL% neq 0 (
    echo ERROR: Frontend build failed!
    pause
    exit /b 1
)

:: バックエンドを先にビルド（コンパイル待ちを避ける）
echo.
echo Building backend...
cargo build --bin vyuber-backend
if %ERRORLEVEL% neq 0 (
    echo ERROR: Backend build failed!
    pause
    exit /b 1
)

:: バックエンドサーバー起動（環境変数は親プロセスから継承される）
echo.
echo Starting backend server...
set "WHISPER_MODEL_PATH=%MODEL_PATH%"
start "AIVID Backend" cmd /k "cd /d %ROOT_DIR% && cargo run --bin vyuber-backend || (echo. && echo ===== BACKEND CRASHED ===== && pause)"

:: localhost:3000 が応答するまで待つ（最大60秒）
echo Waiting for backend server on http://localhost:3000 ...
set RETRIES=0
:wait_loop
timeout /t 2 /nobreak >nul
curl -s -o nul http://localhost:3000
if %ERRORLEVEL% equ 0 goto server_ready
set /a RETRIES+=1
if %RETRIES% geq 30 (
    echo.
    echo ERROR: Backend server did not start within 60 seconds.
    echo Check the "AIVID Backend" window for errors.
    pause
    exit /b 1
)
goto wait_loop

:server_ready
echo Backend server is ready!

:: Tauri dev 起動
echo.
echo Starting Tauri...
cd /d "%ROOT_DIR%crates\vyuber-desktop"
cargo tauri dev
cd /d "%ROOT_DIR%"

:: クリーンアップ
taskkill /FI "WINDOWTITLE eq AIVID Backend" >nul 2>&1
pause
