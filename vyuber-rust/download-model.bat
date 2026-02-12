@echo off
echo ========================================
echo Whisper Model Downloader
echo ========================================
echo.

set MODEL_DIR=%~dp0models
if not exist "%MODEL_DIR%" mkdir "%MODEL_DIR%"

echo [1] ggml-tiny.bin   (75MB)  - 最速・低精度 (非推奨)
echo [2] ggml-base.bin   (142MB) - バランス型 (CPU環境)
echo [3] ggml-small.bin  (466MB) - 高精度 (推奨：CPU/GPU両対応)
echo [4] ggml-medium.bin (1.5GB) - 最高精度 (GPU専用：GTX 1060以上)
echo.
echo 推奨設定:
echo   CPU専用    : [2] base または [3] small
echo   GPU (CUDA) : [4] medium (推奨)
echo.
set /p CHOICE="番号を選んでください (デフォルト: 4): "

if "%CHOICE%"=="" set CHOICE=4

if "%CHOICE%"=="1" (
    set MODEL_NAME=ggml-tiny.bin
) else if "%CHOICE%"=="2" (
    set MODEL_NAME=ggml-base.bin
) else if "%CHOICE%"=="3" (
    set MODEL_NAME=ggml-small.bin
) else if "%CHOICE%"=="4" (
    set MODEL_NAME=ggml-medium.bin
) else (
    echo 無効な選択です
    pause
    exit /b 1
)

set MODEL_FILE=%MODEL_DIR%\%MODEL_NAME%
set MODEL_URL=https://huggingface.co/ggerganov/whisper.cpp/resolve/main/%MODEL_NAME%

if exist "%MODEL_FILE%" (
    echo Model already exists: %MODEL_FILE%
    echo Delete it first if you want to re-download.
    pause
    exit /b 0
)

echo Downloading %MODEL_NAME%...
echo URL: %MODEL_URL%
echo.

curl -L -o "%MODEL_FILE%" "%MODEL_URL%"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Download failed!
    del "%MODEL_FILE%" 2>nul
    pause
    exit /b 1
)

echo.
echo Download complete: %MODEL_FILE%
pause
