@echo off
chcp 65001
echo ========================================================
echo  AIVID 設定リセットツール
echo ========================================================
echo.
echo  アプリのキャッシュ（マイク許可設定など）を削除して
echo  初期状態に戻します。
echo.
echo  実行中はアプリが強制終了されます。
echo.
pause

REM 1. アプリを強制終了
taskkill /F /IM "vyuber-backend.exe" /T >nul 2>&1
taskkill /F /IM "vyuber.exe" /T >nul 2>&1
REM ※念のため製品名のプロセスもキルします
taskkill /F /IM "AIVID.exe" /T >nul 2>&1

REM 2. 設定フォルダ（記憶）を削除
echo.
echo  設定を削除中...

REM ★ここが正解のフォルダ名です！
rmdir /S /Q "%LOCALAPPDATA%\com.aivid.desktop" >nul 2>&1

echo.
echo  完了しました！
echo  アプリを再起動すれば、もう一度「許可」ポップアップが出ます。
echo.
pause