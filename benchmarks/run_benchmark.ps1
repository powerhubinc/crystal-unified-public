# Crystal Unified Comprehensive Benchmark Script
# Usage: .\run_benchmark.ps1 [-Quick] [-NoSearch] [-Corpus <name>]
#
# Options:
#   -Quick      Skip large files (>100MB)
#   -NoSearch   Skip search benchmarks
#   -Corpus     Only benchmark specific corpus (silesia, canterbury, enwik, loghub)

param(
    [switch]$Quick,
    [switch]$NoSearch,
    [string]$Corpus = ""
)

$ErrorActionPreference = "Stop"

# Configuration - uses paths relative to script location
$SCRIPT_ROOT = $PSScriptRoot
$PROJECT_ROOT = (Resolve-Path "$SCRIPT_ROOT\..").Path
$CUZ = "$PROJECT_ROOT\target\release\cuz.exe"
$DATA_DIR = if ($env:CUZ_BENCHMARK_DATA) { $env:CUZ_BENCHMARK_DATA } else { "$PROJECT_ROOT\data" }
$OUTPUT_DIR = "$SCRIPT_ROOT\output"
$REPORT_FILE = "$PROJECT_ROOT\docs\BENCHMARK_RESULTS.md"

# Create output directory
if (-not (Test-Path $OUTPUT_DIR)) {
    New-Item -ItemType Directory -Path $OUTPUT_DIR | Out-Null
}

# Benchmark corpora definitions
$Corpora = @{
    loghub = @(
        @{ Path = "openstack_abnormal.log"; Category = "Log" },
        @{ Path = "openstack_normal1.log"; Category = "Log" },
        @{ Path = "Mac.log"; Category = "Log" },
        @{ Path = "HealthApp.log"; Category = "Log" },
        @{ Path = "HPC.log"; Category = "Log" },
        @{ Path = "openstack_normal2.log"; Category = "Log" },
        @{ Path = "SSH.log"; Category = "Log" },
        @{ Path = "Android.log"; Category = "Log" },
        @{ Path = "BGL.log"; Category = "Log" },
        @{ Path = "HDFS.log"; Category = "Log" }
    )
    silesia = @(
        @{ Path = "silesia\dickens"; Category = "Text" },
        @{ Path = "silesia\mozilla"; Category = "Binary" },
        @{ Path = "silesia\mr"; Category = "Image" },
        @{ Path = "silesia\nci"; Category = "Chemistry" },
        @{ Path = "silesia\ooffice"; Category = "Binary" },
        @{ Path = "silesia\osdb"; Category = "Database" },
        @{ Path = "silesia\reymont"; Category = "Text" },
        @{ Path = "silesia\samba"; Category = "Source" },
        @{ Path = "silesia\sao"; Category = "Binary" },
        @{ Path = "silesia\webster"; Category = "Text" },
        @{ Path = "silesia\xml"; Category = "XML" },
        @{ Path = "silesia\x-ray"; Category = "Image" }
    )
    canterbury = @(
        @{ Path = "canterbury\alice29.txt"; Category = "Text" },
        @{ Path = "canterbury\asyoulik.txt"; Category = "Text" },
        @{ Path = "canterbury\cp.html"; Category = "HTML" },
        @{ Path = "canterbury\fields.c"; Category = "Source" },
        @{ Path = "canterbury\grammar.lsp"; Category = "Source" },
        @{ Path = "canterbury\kennedy.xls"; Category = "Binary" },
        @{ Path = "canterbury\lcet10.txt"; Category = "Text" },
        @{ Path = "canterbury\plrabn12.txt"; Category = "Text" },
        @{ Path = "canterbury\ptt5"; Category = "Image" },
        @{ Path = "canterbury\sum"; Category = "Binary" },
        @{ Path = "canterbury\xargs.1"; Category = "Text" }
    )
    enwik = @(
        @{ Path = "enwik\enwik8"; Category = "Text" }
    )
}

# Search test patterns for different file types
$SearchPatterns = @{
    Log = @("error", "ERROR", "warning", "failed", "exception", "timeout")
    Text = @("the", "and", "that", "which", "from")
    Source = @("int", "return", "if", "function", "void")
    XML = @("<", "/>", "xml", "version", "encoding")
    HTML = @("<html", "<div", "class=", "href=", "<p>")
    Binary = @()  # Skip binary files for search
    Image = @()   # Skip image files for search
    Database = @()
    Chemistry = @()
}

# Results storage
$Results = @()
$SearchResults = @()

function Format-Size {
    param([long]$Bytes)
    if ($Bytes -ge 1GB) { return "{0:N2} GB" -f ($Bytes / 1GB) }
    if ($Bytes -ge 1MB) { return "{0:N2} MB" -f ($Bytes / 1MB) }
    if ($Bytes -ge 1KB) { return "{0:N2} KB" -f ($Bytes / 1KB) }
    return "$Bytes B"
}

function Format-Speed {
    param([double]$BytesPerSecond)
    if ($BytesPerSecond -ge 1GB) { return "{0:N0} GB/s" -f ($BytesPerSecond / 1GB) }
    if ($BytesPerSecond -ge 1MB) { return "{0:N0} MB/s" -f ($BytesPerSecond / 1MB) }
    if ($BytesPerSecond -ge 1KB) { return "{0:N0} KB/s" -f ($BytesPerSecond / 1KB) }
    return "{0:N0} B/s" -f $BytesPerSecond
}

function Get-ProcessMemory {
    param([System.Diagnostics.Process]$Process)
    try {
        return $Process.PeakWorkingSet64
    } catch {
        return 0
    }
}

function Run-Benchmark {
    param(
        [string]$InputFile,
        [string]$Mode,
        [string]$ExtraArgs = "",
        [switch]$TrackMemory
    )

    $OutputFile = Join-Path $OUTPUT_DIR ((Split-Path $InputFile -Leaf) + ".cuz")

    # Build arguments
    $ArgList = "$Mode `"$InputFile`" -o `"$OutputFile`""
    if ($ExtraArgs) {
        $ArgList += " $ExtraArgs"
    }

    # Run compression with memory tracking
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $CUZ
    $psi.Arguments = $ArgList
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true

    $Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $Process = [System.Diagnostics.Process]::Start($psi)
    $Process.WaitForExit()
    $Stopwatch.Stop()

    $CompressTime = $Stopwatch.Elapsed.TotalSeconds
    $CompressMemory = $Process.PeakWorkingSet64

    # Get sizes
    $OriginalSize = (Get-Item $InputFile).Length
    if (-not (Test-Path $OutputFile)) {
        return @{
            Success = $false
            Error = "Compression failed"
        }
    }
    $CompressedSize = (Get-Item $OutputFile).Length
    $Ratio = $CompressedSize / $OriginalSize
    $Speed = $OriginalSize / $CompressTime

    # Run decompression
    $DecompFile = $OutputFile + ".dec"
    $psi.Arguments = "decompress `"$OutputFile`" -o `"$DecompFile`""

    $Stopwatch.Restart()
    $Process = [System.Diagnostics.Process]::Start($psi)
    $Process.WaitForExit()
    $Stopwatch.Stop()

    $DecompressTime = $Stopwatch.Elapsed.TotalSeconds
    $DecompressMemory = $Process.PeakWorkingSet64
    $DecompSpeed = $OriginalSize / $DecompressTime

    # Verify
    $Verified = $false
    if (Test-Path $DecompFile) {
        $OriginalHash = (Get-FileHash $InputFile -Algorithm MD5).Hash
        $DecompHash = (Get-FileHash $DecompFile -Algorithm MD5).Hash
        $Verified = $OriginalHash -eq $DecompHash
        Remove-Item $DecompFile -Force -ErrorAction SilentlyContinue
    }

    return @{
        Success = $true
        OriginalSize = $OriginalSize
        CompressedSize = $CompressedSize
        Ratio = $Ratio
        CompressTime = $CompressTime
        DecompressTime = $DecompressTime
        CompressSpeed = $Speed
        DecompressSpeed = $DecompSpeed
        CompressMemory = $CompressMemory
        DecompressMemory = $DecompressMemory
        Verified = $Verified
    }
}

function Run-SearchBenchmark {
    param(
        [string]$CompressedFile,
        [string]$OriginalFile,
        [string]$Pattern
    )

    # Search compressed file
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $CUZ
    $psi.Arguments = "search `"$CompressedFile`" `"$Pattern`" --count"
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true

    $Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $Process = [System.Diagnostics.Process]::Start($psi)
    $Output = $Process.StandardOutput.ReadToEnd()
    $Process.WaitForExit()
    $Stopwatch.Stop()

    $CuzTime = $Stopwatch.Elapsed.TotalMilliseconds
    $CuzCount = 0
    if ($Output -match '(\d+)') {
        $CuzCount = [int]$Matches[1]
    }

    # Search original with Select-String (grep -c equivalent) - count LINES containing pattern
    # CUZ search --count returns line count, not occurrence count, so we match that behavior
    $Stopwatch.Restart()
    $GrepCount = (Select-String -Path $OriginalFile -Pattern $Pattern -CaseSensitive).Count
    if ($null -eq $GrepCount) { $GrepCount = 0 }
    $Stopwatch.Stop()
    $GrepTime = $Stopwatch.Elapsed.TotalMilliseconds

    return @{
        Pattern = $Pattern
        CuzCount = $CuzCount
        CuzTime = $CuzTime
        GrepCount = $GrepCount
        GrepTime = $GrepTime
        Speedup = if ($CuzTime -gt 0) { $GrepTime / $CuzTime } else { 0 }
        Accurate = ($CuzCount -eq $GrepCount)
    }
}

# Header
Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Crystal Unified Comprehensive Benchmark" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Data directory: $DATA_DIR"
Write-Host "Output directory: $OUTPUT_DIR"
Write-Host "Report file: $REPORT_FILE"
if ($Quick) { Write-Host "Mode: Quick (skipping large files)" -ForegroundColor Yellow }
if ($NoSearch) { Write-Host "Mode: No search benchmarks" -ForegroundColor Yellow }
if ($Corpus) { Write-Host "Corpus filter: $Corpus" -ForegroundColor Yellow }
Write-Host ""

# Determine which corpora to benchmark
$CorporaToTest = @()
if ($Corpus) {
    if ($Corpora.ContainsKey($Corpus.ToLower())) {
        $CorporaToTest = @($Corpus.ToLower())
    } else {
        Write-Host "Unknown corpus: $Corpus" -ForegroundColor Red
        Write-Host "Available: $($Corpora.Keys -join ', ')"
        exit 1
    }
} else {
    $CorporaToTest = $Corpora.Keys
}

# Run benchmarks for each corpus
foreach ($CorpusName in $CorporaToTest) {
    Write-Host ""
    Write-Host "=== $($CorpusName.ToUpper()) CORPUS ===" -ForegroundColor Magenta

    foreach ($File in $Corpora[$CorpusName]) {
        $FilePath = Join-Path $DATA_DIR $File.Path

        if (-not (Test-Path $FilePath)) {
            Write-Host "  [SKIP] $($File.Path) - not found" -ForegroundColor Yellow
            continue
        }

        $FileSize = (Get-Item $FilePath).Length

        # Skip large files in Quick mode
        if ($Quick -and $FileSize -gt 100MB) {
            Write-Host "  [SKIP] $($File.Path) - too large for quick mode" -ForegroundColor Yellow
            continue
        }

        Write-Host "Testing: $($File.Path) ($(Format-Size $FileSize)) [$($File.Category)]" -ForegroundColor Green

        # Test modes
        $TestModes = @(
            @{ Name = "Single-thread"; Args = "" },
            @{ Name = "Parallel"; Args = "-j" },
            @{ Name = "Fast"; Args = "--fast" },
            @{ Name = "Level 9"; Args = "-l 9" }
        )

        $FileResults = @{
            File = $File.Path
            Corpus = $CorpusName
            Category = $File.Category
            OriginalSize = $FileSize
            Modes = @{}
        }

        foreach ($Mode in $TestModes) {
            Write-Host "  $($Mode.Name)..." -NoNewline
            $Result = Run-Benchmark -InputFile $FilePath -Mode "compress" -ExtraArgs $Mode.Args -TrackMemory

            if ($Result.Success) {
                Write-Host " $(Format-Speed $Result.CompressSpeed) | $("{0:P1}" -f $Result.Ratio) | $(if($Result.Verified){'OK'}else{'FAIL'})" -ForegroundColor $(if($Result.Verified){'Gray'}else{'Red'})
                $FileResults.Modes[$Mode.Name] = $Result
            } else {
                Write-Host " FAILED" -ForegroundColor Red
            }
        }

        # Search benchmark (if enabled and applicable)
        if (-not $NoSearch -and $SearchPatterns[$File.Category].Count -gt 0) {
            $CompressedFile = Join-Path $OUTPUT_DIR ((Split-Path $FilePath -Leaf) + ".cuz")

            # Re-compress with compress mode for search test (auto mode may skip search indexes)
            $null = Run-Benchmark -InputFile $FilePath -Mode "compress" -ExtraArgs ""

            if (Test-Path $CompressedFile) {
                Write-Host "  Search tests:" -ForegroundColor Cyan
                foreach ($Pattern in $SearchPatterns[$File.Category] | Select-Object -First 3) {
                    $SearchResult = Run-SearchBenchmark -CompressedFile $CompressedFile -OriginalFile $FilePath -Pattern $Pattern
                    $AccuracyMark = if ($SearchResult.Accurate) { "=" } else { "!=" }
                    Write-Host "    '$Pattern': cuz=$($SearchResult.CuzCount) $AccuracyMark grep=$($SearchResult.GrepCount) | $('{0:N1}ms' -f $SearchResult.CuzTime) vs $('{0:N1}ms' -f $SearchResult.GrepTime)" -ForegroundColor $(if($SearchResult.Accurate){'Gray'}else{'Yellow'})

                    $SearchResults += @{
                        File = $File.Path
                        Pattern = $Pattern
                        CuzCount = $SearchResult.CuzCount
                        CuzTime = $SearchResult.CuzTime
                        GrepCount = $SearchResult.GrepCount
                        GrepTime = $SearchResult.GrepTime
                        Speedup = $SearchResult.Speedup
                        Accurate = $SearchResult.Accurate
                    }
                }
            }
        }

        $Results += $FileResults
    }
}

# Generate comprehensive report
$Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$CPUInfo = (Get-CimInstance Win32_Processor).Name
$Cores = (Get-CimInstance Win32_Processor).NumberOfCores
$LogicalProcessors = (Get-CimInstance Win32_Processor).NumberOfLogicalProcessors
$RAM = "{0:N0} GB" -f ((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB)

$ReportContent = @"
# Crystal Unified Benchmark Results

**Generated:** $Timestamp
**System:** $env:COMPUTERNAME
**CPU:** $CPUInfo ($Cores cores, $LogicalProcessors threads)
**RAM:** $RAM
**CUZ Version:** v1.0

## Executive Summary

"@

# Calculate summary statistics
$TotalOriginal = ($Results | ForEach-Object { $_.OriginalSize } | Measure-Object -Sum).Sum
$TotalCompressed = ($Results | Where-Object { $_.Modes["Single-thread"] } | ForEach-Object { $_.Modes["Single-thread"].CompressedSize } | Measure-Object -Sum).Sum
$AvgRatio = if ($TotalOriginal -gt 0) { $TotalCompressed / $TotalOriginal } else { 0 }
$AllVerified = ($Results | Where-Object { $_.Modes["Single-thread"] } | ForEach-Object { $_.Modes["Single-thread"].Verified } | Where-Object { -not $_ }).Count -eq 0

$ReportContent += @"
| Metric | Value |
|--------|-------|
| Files Tested | $($Results.Count) |
| Total Data | $(Format-Size $TotalOriginal) |
| Avg Compression Ratio | $("{0:P1}" -f $AvgRatio) |
| All Verified | $(if($AllVerified){'Yes'}else{'**NO**'}) |

## Compression Results by Corpus

"@

# Results by corpus
foreach ($CorpusName in ($Results | ForEach-Object { $_.Corpus } | Sort-Object -Unique)) {
    $CorpusResults = $Results | Where-Object { $_.Corpus -eq $CorpusName }

    $ReportContent += @"

### $($CorpusName.ToUpper()) Corpus

| File | Original | Single | Parallel | Fast | Level 9 | Verified |
|------|----------|--------|----------|------|---------|----------|
"@
    foreach ($R in $CorpusResults) {
        $SingleRatio = if ($R.Modes["Single-thread"]) { "{0:P1}" -f $R.Modes["Single-thread"].Ratio } else { "-" }
        $ParallelRatio = if ($R.Modes["Parallel"]) { "{0:P1}" -f $R.Modes["Parallel"].Ratio } else { "-" }
        $FastRatio = if ($R.Modes["Fast"]) { "{0:P1}" -f $R.Modes["Fast"].Ratio } else { "-" }
        $Level9Ratio = if ($R.Modes["Level 9"]) { "{0:P1}" -f $R.Modes["Level 9"].Ratio } else { "-" }
        $Verified = if ($R.Modes["Single-thread"]) { if ($R.Modes["Single-thread"].Verified) { "Yes" } else { "**NO**" } } else { "-" }

        $ReportContent += "`n| $(Split-Path $R.File -Leaf) | $(Format-Size $R.OriginalSize) | $SingleRatio | $ParallelRatio | $FastRatio | $Level9Ratio | $Verified |"
    }
}

# Speed comparison table
$ReportContent += @"

## Compression Speed Comparison

### Single-Thread vs Parallel (MB/s)

| File | Single | Parallel | Speedup |
|------|--------|----------|---------|
"@

foreach ($R in $Results) {
    if ($R.Modes["Single-thread"] -and $R.Modes["Parallel"]) {
        $SingleSpeed = $R.Modes["Single-thread"].CompressSpeed / 1MB
        $ParallelSpeed = $R.Modes["Parallel"].CompressSpeed / 1MB
        $Speedup = if ($SingleSpeed -gt 0) { $ParallelSpeed / $SingleSpeed } else { 0 }

        $ReportContent += "`n| $(Split-Path $R.File -Leaf) | $("{0:N1}" -f $SingleSpeed) | $("{0:N1}" -f $ParallelSpeed) | $("{0:N1}x" -f $Speedup) |"
    }
}

# Decompression speed
$ReportContent += @"

### Decompression Speed (MB/s)

| File | Single | Parallel |
|------|--------|----------|
"@

foreach ($R in $Results) {
    if ($R.Modes["Single-thread"]) {
        $SingleDecomp = $R.Modes["Single-thread"].DecompressSpeed / 1MB
        $ParallelDecomp = if ($R.Modes["Parallel"]) { $R.Modes["Parallel"].DecompressSpeed / 1MB } else { 0 }

        $ReportContent += "`n| $(Split-Path $R.File -Leaf) | $("{0:N1}" -f $SingleDecomp) | $("{0:N1}" -f $ParallelDecomp) |"
    }
}

# Memory usage
$ReportContent += @"

## Memory Usage

| File | Compress (Peak) | Decompress (Peak) |
|------|-----------------|-------------------|
"@

foreach ($R in $Results) {
    if ($R.Modes["Single-thread"]) {
        $CompMem = Format-Size $R.Modes["Single-thread"].CompressMemory
        $DecompMem = Format-Size $R.Modes["Single-thread"].DecompressMemory

        $ReportContent += "`n| $(Split-Path $R.File -Leaf) | $CompMem | $DecompMem |"
    }
}

# Search benchmarks
if ($SearchResults.Count -gt 0) {
    $ReportContent += @"

## Search Performance

Search without decompression vs grep on original file.

| File | Pattern | CUZ Matches | Grep Matches | CUZ Time | Grep Time | Speedup | Accurate |
|------|---------|-------------|--------------|----------|-----------|---------|----------|
"@

    foreach ($S in $SearchResults) {
        $Accurate = if ($S.Accurate) { "Yes" } else { "**NO**" }
        $ReportContent += "`n| $(Split-Path $S.File -Leaf) | ``$($S.Pattern)`` | $($S.CuzCount) | $($S.GrepCount) | $("{0:N1}ms" -f $S.CuzTime) | $("{0:N1}ms" -f $S.GrepTime) | $("{0:N1}x" -f $S.Speedup) | $Accurate |"
    }

    # Search summary
    $AccurateSearches = ($SearchResults | Where-Object { $_.Accurate }).Count
    $TotalSearches = $SearchResults.Count
    $AvgSpeedup = ($SearchResults | Where-Object { $_.Speedup -gt 0 } | ForEach-Object { $_.Speedup } | Measure-Object -Average).Average

    $ReportContent += @"

### Search Summary

- **Total searches:** $TotalSearches
- **Accurate results:** $AccurateSearches / $TotalSearches ($("{0:P0}" -f ($AccurateSearches / $TotalSearches)))
- **Average speedup:** $("{0:N1}x" -f $AvgSpeedup) faster than grep

"@
}

# Detailed results per file
$ReportContent += @"

## Detailed Results

"@

foreach ($R in $Results) {
    $ReportContent += @"

### $(Split-Path $R.File -Leaf)

- **Path:** $($R.File)
- **Corpus:** $($R.Corpus)
- **Category:** $($R.Category)
- **Original Size:** $(Format-Size $R.OriginalSize)

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
"@

    foreach ($ModeName in @("Single-thread", "Parallel", "Fast", "Level 9")) {
        if ($R.Modes[$ModeName]) {
            $M = $R.Modes[$ModeName]
            $ReportContent += "`n| $ModeName | $(Format-Size $M.CompressedSize) | $("{0:P2}" -f $M.Ratio) | $(Format-Speed $M.CompressSpeed) | $(Format-Speed $M.DecompressSpeed) | $(Format-Size $M.CompressMemory) | $(if($M.Verified){'Yes'}else{'**NO**'}) |"
        }
    }
}

$ReportContent += @"

---

*Generated by Crystal Unified Benchmark Script*
"@

# Save report
Set-Content -Path $REPORT_FILE -Value $ReportContent -Encoding UTF8

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Benchmark Complete!" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Files tested: $($Results.Count)"
Write-Host "Total data: $(Format-Size $TotalOriginal)"
Write-Host "Average ratio: $("{0:P1}" -f $AvgRatio)"
Write-Host "All verified: $(if($AllVerified){'Yes'}else{'NO - CHECK REPORT'})"
Write-Host ""
Write-Host "Report saved to: $REPORT_FILE" -ForegroundColor Green
Write-Host ""

# Cleanup compressed files
Write-Host "Cleaning up temporary files..."
Remove-Item "$OUTPUT_DIR\*.cuz" -Force -ErrorAction SilentlyContinue
Remove-Item "$OUTPUT_DIR\*.dec" -Force -ErrorAction SilentlyContinue

Write-Host "Done!" -ForegroundColor Green
