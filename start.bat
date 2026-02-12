@echo off
cd /d "%~dp0vyuber-rust"

:: MediaMTXが無ければ自動ダウンロード
if not exist "mediamtx\mediamtx.exe" (
    echo [setup] MediaMTX not found. Downloading...
    mkdir mediamtx 2>nul

    curl -L -o mediamtx\mediamtx.zip "https://github.com/bluenviron/mediamtx/releases/download/v1.15.6/mediamtx_v1.15.6_windows_amd64.zip"
    if errorlevel 1 (
        echo [error] Failed to download MediaMTX.
        pause
        exit /b 1
    )

    tar -xf mediamtx\mediamtx.zip -C mediamtx
    del mediamtx\mediamtx.zip
    echo [setup] MediaMTX downloaded successfully.
)

trunk build && npx @infisical/cli run -- cargo run --release
pause