# Crystal Unified

**Search your compressed logs. Skip the decompress step.**

[![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](LICENSES/LICENSE.md)
[![Patent Pending](https://img.shields.io/badge/Patent-Pending-orange.svg)](LICENSES/LICENSE.md)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Why Crystal Unified?

### Search Without Decompressing

```bash
# Traditional: streaming decompress, but entire file passes through memory
zstd -dc huge_logs.zst | grep "error"   # 709 MB decompressed through RAM

# Crystal: indexed search, decompress only matching blocks
cuz search huge_logs.cuz "error"        # Jump directly to matches
```

Crystal builds search indexes during compression. Traditional tools must decompress the entire file through memory even when streaming. Crystal jumps directly to matching blocks.

### Compression That Understands Your Data

Crystal doesn't just compress bytes. It recognizes structure:

| Data Type | What Crystal Sees | Result |
|-----------|-------------------|--------|
| Log files | Repeating templates with variable fields | **6-11% of original size** |
| DNA sequences | 4-letter alphabet (ACGT) | **2-bit encoding + reference compression** |
| Time series | Sequential numeric patterns | **Delta-encoded efficiency** |
| Firmware | Binary with sparse changes | **Block-level random access** |

### Real-World Performance

Tested on [Loghub](https://github.com/logpai/loghub) benchmark dataset:

| Log File | Original | Compressed | Ratio | Speed |
|----------|----------|------------|-------|-------|
| BGL.log | 709 MB | 62 MB | **8.7%** | 476 MB/s |
| Android.log | 183 MB | 19 MB | **10.2%** | 400 MB/s |
| SSH.log | 70 MB | 4.7 MB | **6.7%** | 383 MB/s |
| Mac.log | 16 MB | 1.0 MB | **6.2%** | 253 MB/s |

*Parallel mode, decompression verified. Full results in [docs/BENCHMARK_RESULTS.md](docs/BENCHMARK_RESULTS.md).*

### How It Compares

| Feature | Crystal | gzip | zstd | lz4 |
|---------|---------|------|------|-----|
| Search compressed files | Yes | No | No | No |
| Log-aware compression | Yes | No | No | No |
| Domain transforms (DNA, numeric) | Yes | No | No | No |
| Parallel compression | Yes | pigz | pzstd | Yes |
| Block-level random access | Yes | No | Seekable | No |
| Delta patching built-in | Yes | No | No | No |

## Quick Start

### Install

```bash
git clone https://github.com/powerhubinc/crystal-unified-public.git
cd crystal-unified-public

# Build with cargo directly
cargo build --release

# Or use platform-specific build scripts:
# Linux/macOS: ./build.sh
# Windows CMD: build.bat
# Windows PowerShell: .\build.ps1

# Binary: target/release/cuz (or cuz.exe on Windows)
```

### Compress

```bash
# Auto-detect optimal settings
cuz auto server.log

# Parallel compression for large files
cuz compress huge.log -j

# Force DNA mode for genomic data
cuz compress genome.fasta -t dna
```

### Search

```bash
# Find errors without decompressing
cuz search app.cuz "error"
cuz search app.cuz "connection refused" --count

# Combine with standard tools
cuz search logs.cuz "404" | grep "api/users"
```

### Decompress

```bash
cuz decompress archive.cuz
cuz decompress archive.cuz output.txt
```

## Use Cases

### Log Management

Store months of logs at 10x compression. Search any term across archives instantly.

```bash
# Compress daily logs
cuz compress /var/log/app-2026-01-*.log -j -o january-logs.cuz

# Search across compressed logs
cuz search january-logs.cuz "OutOfMemoryError"
```

### Genomic Data

Reference-based compression with true lossless FASTA roundtrip.

```bash
# Build reference index (one-time)
cuz dna-index reference.fa reference.cdni

# Compress against reference (1.7% ratio for same-species samples)
cuz dna-compress sample.fa -r reference.cdni
# 3.3 GB -> 58 MB (headers, line wrapping, N's, lowercase preserved)

# Standalone 2-bit encoding (no reference needed)
cuz compress sequences.fasta -t dna
```

### Firmware Updates

Block-based compression with delta patching. Ship small OTA updates.

```bash
# Create firmware archive with random access
cuz firmware v1.0.bin -b 4096

# Generate delta between versions
cuz delta v1.0.bin v1.1.bin -o update.cuzd

# Apply on device
cuz apply current.bin update.cuzd -o updated.bin
```

### Time Series / IoT

Delta encoding exploits sequential patterns in sensor data.

```bash
cuz compress sensor_readings.csv -t numeric
```

## CLI Reference

### Commands

| Command | Description |
|---------|-------------|
| `auto <file>` | Auto-detect type and compress |
| `compress <file>` | Compress with options |
| `decompress <file>` | Decompress to original |
| `search <file> <term>` | Search without decompressing |
| `analyze <file>` | Detect data type |
| `archive <out> <files...>` | Create multi-file archive |
| `extract <archive> <dir>` | Extract archive |
| `delta <old> <new>` | Create binary patch |
| `apply <base> <patch>` | Apply binary patch |
| `firmware <file>` | Block-based compression |

### Options

| Option | Description |
|--------|-------------|
| `-l <1-22>` | Compression level (1=fast, 22=best) |
| `-t <type>` | Transform: `dna`, `numeric`, `binary`, `nibble`, `struct` |
| `-j` | Parallel compression (uses all cores) |
| `-s` | Streaming mode (constant memory) |
| `-o <path>` | Output path |
| `-b <size>` | Block size (firmware mode) |
| `--fast` | Optimize for speed over ratio |

## Library API

### Basic Usage

```rust
use crystal_unified::{compress, decompress};

fn main() -> crystal_unified::Result<()> {
    let data = b"Hello, World!";

    let compressed = compress(data)?;
    let original = decompress(&compressed)?;

    assert_eq!(data.as_slice(), original.as_slice());
    Ok(())
}
```

### With Options

```rust
use crystal_unified::{compress_with_options, CompressionOptions, TRANSFORM_DNA_2BIT};

let options = CompressionOptions::default()
    .with_compression_level(6)
    .with_transform(TRANSFORM_DNA_2BIT)
    .with_dictionary(true);

let compressed = compress_with_options(data, options)?;
```

### Parallel Compression

```rust
use crystal_unified::{compress_parallel_with_options, CompressionOptions};

// Uses all available CPU cores
let compressed = compress_parallel_with_options(large_data, options)?;
```

### Smart Detection

```rust
use crystal_unified::smart_detect;

let result = smart_detect(data);
println!("Detected: {:?}, transform: {}", result.data_type, result.transform);

// Use detected settings
let options = result.to_options();
```

### Search API

```rust
use crystal_unified::CrystalReaderV10;

let reader = CrystalReaderV10::new(&compressed_data)?;
let matches = reader.search_parallel(b"error");
let count = reader.count_matches(b"error");
```

## File Formats

| Extension | Purpose | Key Feature |
|-----------|---------|-------------|
| `.cuz` | General compression | Searchable, streamable |
| `.cuzb` | Binary/firmware | Random block access |
| `.cuzd` | Delta patches | Minimal update size |

## Requirements

- Rust 1.70+
- Windows, Linux, or macOS

## Documentation

- [Auto Detection](docs/AUTO_DETECTION.md) - How smart detection works
- [Genomic Data](docs/GENOMIC_DATA.md) - DNA/FASTA compression guide
- [Log Compression](docs/LOG_COMPRESSION.md) - Log file optimization
- [Firmware Patching](docs/FIRMWARE_PATCHING.md) - OTA update workflow
- [Benchmark Results](docs/BENCHMARK_RESULTS.md) - Performance data
- [Benchmark Corpora](docs/BENCHMARK_CORPORA.md) - Standard test datasets

## Running Benchmarks

```bash
# Download standard benchmark corpora (see docs/BENCHMARK_CORPORA.md)
# Set data directory via environment variable:
export CUZ_BENCHMARK_DATA=/path/to/benchmark/data  # Linux/macOS
set CUZ_BENCHMARK_DATA=C:\benchmark\data           # Windows

# Linux/macOS
./benchmarks/run_benchmark.sh
./benchmarks/run_benchmark.sh --quick
./benchmarks/run_benchmark.sh --corpus loghub

# Windows PowerShell
.\benchmarks\run_benchmark.ps1
.\benchmarks\run_benchmark.ps1 -Quick
.\benchmarks\run_benchmark.ps1 -Corpus loghub

# Windows CMD (basic version)
benchmarks\run_benchmark.bat
```

## License

[Business Source License 1.1](LICENSES/LICENSE.md)

- Evaluation and non-production use: Free
- Production use: Requires commercial license
- Patent pending

**Commercial licensing:** licensing@powerhub.inc

## Contributing

Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

All contributions require signing our [CLA](LICENSES/CLA.md).

## Security

Report vulnerabilities: security@powerhub.inc

See [SECURITY.md](SECURITY.md).

---

Copyright (c) 2026 Powerhub Inc. All rights reserved.
