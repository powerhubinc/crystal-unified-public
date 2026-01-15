@echo off
REM Build Crystal Unified
cd /d "%~dp0"
cargo build --release
