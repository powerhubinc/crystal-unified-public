@echo off
REM Crystal Unified Benchmark Script for Windows CMD
REM Usage: run_benchmark.bat [options]
REM
REM This is a simplified version. For full features, use run_benchmark.ps1

setlocal EnableDelayedExpansion

REM Configuration - relative paths
set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%.."
set "CUZ=%PROJECT_ROOT%\target\release\cuz.exe"
set "OUTPUT_DIR=%SCRIPT_DIR%output"
set "REPORT_FILE=%PROJECT_ROOT%\docs\BENCHMARK_RESULTS.md"

REM Use environment variable for data dir, or default
if defined CUZ_BENCHMARK_DATA (
    set "DATA_DIR=%CUZ_BENCHMARK_DATA%"
) else (
    set "DATA_DIR=%PROJECT_ROOT%\data"
)

REM Create output directory
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo.
echo ================================================================
echo   Crystal Unified Benchmark (CMD)
echo ================================================================
echo.
echo Data directory: %DATA_DIR%
echo Output directory: %OUTPUT_DIR%
echo.

REM Check if CUZ exists
if not exist "%CUZ%" (
    echo Error: CUZ binary not found at %CUZ%
    echo Run 'cargo build --release' first
    exit /b 1
)

REM Simple benchmark - just test a few files
echo Running basic benchmark...
echo.

REM Test loghub files if they exist
for %%f in (Mac.log SSH.log Android.log) do (
    if exist "%DATA_DIR%\%%f" (
        echo Testing: %%f

        REM Compress
        echo   Compressing...
        "%CUZ%" compress "%DATA_DIR%\%%f" -o "%OUTPUT_DIR%\%%f.cuz"

        REM Decompress and verify
        echo   Decompressing...
        "%CUZ%" decompress "%OUTPUT_DIR%\%%f.cuz" -o "%OUTPUT_DIR%\%%f.dec"

        REM Compare using fc
        fc /b "%DATA_DIR%\%%f" "%OUTPUT_DIR%\%%f.dec" > nul 2>&1
        if !errorlevel! equ 0 (
            echo   Verified: OK
        ) else (
            echo   Verified: FAILED
        )

        REM Cleanup
        del "%OUTPUT_DIR%\%%f.cuz" 2>nul
        del "%OUTPUT_DIR%\%%f.dec" 2>nul
        echo.
    )
)

echo ================================================================
echo   Benchmark Complete!
echo ================================================================
echo.
echo For detailed benchmarks with timing and reports, use:
echo   PowerShell: .\run_benchmark.ps1
echo   Bash:       ./run_benchmark.sh
echo.

endlocal
