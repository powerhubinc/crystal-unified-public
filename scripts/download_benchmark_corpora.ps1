# Crystal Unified Benchmark Corpus Download Script
# Downloads standard compression benchmark datasets
#
# Usage: .\download_benchmark_corpora.ps1 [-DataDir "path"] [-All] [-Silesia] [-Canterbury] [-Enwik] [-Loghub] [-Large]
#
# Corpora included:
# - Silesia Corpus (212 MB) - Industry standard: text, XML, HTML, binary
# - Canterbury Corpus (3 MB) - Classic benchmark suite
# - Enwik8/Enwik9 (100 MB / 1 GB) - Wikipedia text
# - Loghub (77+ GB) - System logs for log compression benchmarks
# - Large Logs (20+ GB) - Thunderbird, Spirit, HDFS logs

param(
    [string]$DataDir = ".\data",
    [switch]$All,
    [switch]$Silesia,
    [switch]$Canterbury,
    [switch]$Enwik,
    [switch]$Loghub,
    [switch]$Large
)

$ErrorActionPreference = "Stop"

# Create data directory
if (-not (Test-Path $DataDir)) {
    New-Item -ItemType Directory -Path $DataDir | Out-Null
}

function Download-File {
    param([string]$Url, [string]$Output)
    Write-Host "Downloading: $Url"
    Write-Host "  -> $Output"

    if (Test-Path $Output) {
        Write-Host "  [SKIP] Already exists"
        return
    }

    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $Url -OutFile $Output -UseBasicParsing
    Write-Host "  [OK] Downloaded"
}

function Extract-Archive {
    param([string]$Archive, [string]$Destination)
    Write-Host "Extracting: $Archive -> $Destination"

    if (-not (Test-Path $Destination)) {
        New-Item -ItemType Directory -Path $Destination | Out-Null
    }

    if ($Archive -match '\.zip$') {
        Expand-Archive -Path $Archive -DestinationPath $Destination -Force
    } elseif ($Archive -match '\.tar\.gz$|\.tgz$') {
        tar -xzf $Archive -C $Destination
    } elseif ($Archive -match '\.gz$') {
        $outFile = Join-Path $Destination ([System.IO.Path]::GetFileNameWithoutExtension($Archive))
        & gzip -dk $Archive 2>$null
        if (Test-Path ([System.IO.Path]::ChangeExtension($Archive, $null))) {
            Move-Item ([System.IO.Path]::ChangeExtension($Archive, $null)) $outFile -Force
        }
    }
    Write-Host "  [OK] Extracted"
}

# ============================================================================
# SILESIA CORPUS (212 MB)
# Industry standard benchmark: dickens, mozilla, mr, nci, ooffice, osdb,
# reymont, samba, sao, webster, xml, x-ray
# ============================================================================
function Download-Silesia {
    Write-Host "`n=== SILESIA CORPUS ===" -ForegroundColor Cyan
    Write-Host "Industry standard compression benchmark (212 MB)"
    Write-Host "Files: text, XML, HTML, executables, databases, images"

    $silesiaDir = Join-Path $DataDir "silesia"
    $zipFile = Join-Path $DataDir "silesia.zip"

    # Primary mirror (Matt Mahoney)
    Download-File "https://mattmahoney.net/dc/silesia.zip" $zipFile
    Extract-Archive $zipFile $silesiaDir

    Write-Host "`nSilesia corpus files:" -ForegroundColor Green
    Get-ChildItem $silesiaDir | ForEach-Object {
        $size = "{0:N2} MB" -f ($_.Length / 1MB)
        Write-Host "  $($_.Name): $size"
    }
}

# ============================================================================
# CANTERBURY CORPUS (3 MB)
# Classic benchmark
# ============================================================================
function Download-Canterbury {
    Write-Host "`n=== CANTERBURY CORPUS ===" -ForegroundColor Cyan
    Write-Host "Classic compression benchmark (3 MB)"

    $canterburyDir = Join-Path $DataDir "canterbury"
    $zipFile = Join-Path $DataDir "canterbury.zip"

    Download-File "https://corpus.canterbury.ac.nz/resources/cantrbry.zip" $zipFile
    Extract-Archive $zipFile $canterburyDir

    # Also get large canterbury corpus
    $largeZip = Join-Path $DataDir "large_canterbury.zip"
    Download-File "https://corpus.canterbury.ac.nz/resources/large.zip" $largeZip
    Extract-Archive $largeZip $canterburyDir
}

# ============================================================================
# ENWIK (100 MB - 1 GB)
# Wikipedia XML dumps for text compression
# ============================================================================
function Download-Enwik {
    Write-Host "`n=== ENWIK ===" -ForegroundColor Cyan
    Write-Host "Wikipedia text benchmark"

    $enwikDir = Join-Path $DataDir "enwik"
    if (-not (Test-Path $enwikDir)) {
        New-Item -ItemType Directory -Path $enwikDir | Out-Null
    }

    # enwik8 (100 MB)
    $enwik8 = Join-Path $enwikDir "enwik8.zip"
    Download-File "https://mattmahoney.net/dc/enwik8.zip" $enwik8
    Extract-Archive $enwik8 $enwikDir

    Write-Host "`nNote: enwik9 (1 GB) available at https://mattmahoney.net/dc/enwik9.zip"
}

# ============================================================================
# LOGHUB STANDARD (2 GB)
# System logs from Zenodo
# ============================================================================
function Download-Loghub {
    Write-Host "`n=== LOGHUB STANDARD ===" -ForegroundColor Cyan
    Write-Host "System log benchmark collection"
    Write-Host "Source: https://github.com/logpai/loghub"

    $loghubDir = Join-Path $DataDir "loghub"
    if (-not (Test-Path $loghubDir)) {
        New-Item -ItemType Directory -Path $loghubDir | Out-Null
    }

    Write-Host "`nDownloading from Zenodo (DOI: 10.5281/zenodo.8196385)..."

    $datasets = @(
        "Android", "Apache", "BGL", "Hadoop", "HDFS_v1", "HealthApp",
        "HPC", "Linux", "Mac", "OpenSSH", "OpenStack", "Spark", "Windows", "Zookeeper"
    )

    foreach ($ds in $datasets) {
        $url = "https://zenodo.org/records/8196385/files/$ds.tar.gz"
        $tarFile = Join-Path $loghubDir "$ds.tar.gz"
        try {
            Download-File $url $tarFile
            Extract-Archive $tarFile $loghubDir
        } catch {
            Write-Host "  [WARN] Failed to download $ds" -ForegroundColor Yellow
        }
    }
}

# ============================================================================
# LARGE LOGS (20+ GB)
# Thunderbird (30 GB), Spirit (30 GB), HDFS-v2 (16 GB)
# ============================================================================
function Download-LargeLogs {
    Write-Host "`n=== LARGE LOG DATASETS ===" -ForegroundColor Cyan
    Write-Host "Multi-gigabyte log files for stress testing"
    Write-Host "WARNING: These are very large downloads (50+ GB total)"

    $largeDir = Join-Path $DataDir "large_logs"
    if (-not (Test-Path $largeDir)) {
        New-Item -ItemType Directory -Path $largeDir | Out-Null
    }

    Write-Host "`nAvailable large datasets:"
    Write-Host "  1. Thunderbird (~30 GB) - Supercomputer system logs"
    Write-Host "  2. Spirit (~30 GB) - Supercomputer system logs"
    Write-Host "  3. HDFS-v2 (~16 GB) - Distributed filesystem logs"
    Write-Host ""

    $confirm = Read-Host "Download Thunderbird logs? (y/N)"
    if ($confirm -eq 'y') {
        $tbird = Join-Path $largeDir "tbird2.gz"
        Download-File "http://0b4af6cdc2f0c5998459-c0245c5c937c5dedcca3f1764ecc9b2f.r43.cf2.rackcdn.com/hpc4/tbird2.gz" $tbird
    }

    $confirm = Read-Host "Download Spirit logs? (y/N)"
    if ($confirm -eq 'y') {
        $spirit = Join-Path $largeDir "spirit2.gz"
        Download-File "http://0b4af6cdc2f0c5998459-c0245c5c937c5dedcca3f1764ecc9b2f.r43.cf2.rackcdn.com/hpc4/spirit2.gz" $spirit
    }
}

# ============================================================================
# MAIN
# ============================================================================

Write-Host "Crystal Unified Benchmark Corpus Downloader" -ForegroundColor Magenta
Write-Host "==========================================="
Write-Host "Data directory: $DataDir"
Write-Host ""

if ($All -or (-not ($Silesia -or $Canterbury -or $Enwik -or $Loghub -or $Large))) {
    Write-Host "Downloading all standard corpora..."
    Download-Silesia
    Download-Canterbury
    Download-Enwik
    Download-Loghub
} else {
    if ($Silesia) { Download-Silesia }
    if ($Canterbury) { Download-Canterbury }
    if ($Enwik) { Download-Enwik }
    if ($Loghub) { Download-Loghub }
    if ($Large) { Download-LargeLogs }
}

Write-Host "`n=== DOWNLOAD COMPLETE ===" -ForegroundColor Green
Write-Host "Data directory: $DataDir"
Get-ChildItem $DataDir -Directory | ForEach-Object {
    $size = (Get-ChildItem $_.FullName -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    $sizeStr = if ($size -gt 1GB) { "{0:N2} GB" -f ($size / 1GB) } else { "{0:N2} MB" -f ($size / 1MB) }
    Write-Host "  $($_.Name): $sizeStr"
}
