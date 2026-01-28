@echo off
echo Starting VYuber backend with Infisical...
cd /d %~dp0
infisical run -- cargo run --release
