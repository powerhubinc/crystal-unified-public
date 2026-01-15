# Build Crystal Unified
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $ScriptDir
cargo build --release
Pop-Location
