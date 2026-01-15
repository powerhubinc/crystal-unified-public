# Benchmark Corpora

This document describes the standard benchmark corpora used to evaluate Crystal Unified compression.

## Quick Start

```powershell
# Download all standard corpora (~2.5 GB)
.\scripts\download_benchmark_corpora.ps1

# Download specific corpus
.\scripts\download_benchmark_corpora.ps1 -Silesia
.\scripts\download_benchmark_corpora.ps1 -Loghub

# Download to custom location
.\scripts\download_benchmark_corpora.ps1 -DataDir "D:\benchmark_data"

# Download large log datasets (50+ GB, interactive)
.\scripts\download_benchmark_corpora.ps1 -Large
```

## Standard Corpora

### Silesia Corpus (212 MB)

Industry-standard compression benchmark. Created in 2003 as a modern alternative to Canterbury/Calgary corpora.

| File | Size | Type | Description |
|------|------|------|-------------|
| dickens | 10 MB | Text | Collected works of Charles Dickens |
| mozilla | 51 MB | Binary | Mozilla 1.0 tarball |
| mr | 10 MB | Image | Medical MRI scan |
| nci | 33 MB | Chemistry | Chemical structure database |
| ooffice | 6 MB | Binary | OpenOffice.org DLL |
| osdb | 10 MB | Database | MySQL benchmark database |
| reymont | 6 MB | Text | Polish novel (UTF-8) |
| samba | 21 MB | Source | Samba source code |
| sao | 7 MB | Binary | Star catalog (SAO) |
| webster | 41 MB | Text | Webster's dictionary (HTML) |
| xml | 5 MB | XML | XML documents |
| x-ray | 8 MB | Image | Medical X-ray |

**Source:** [mattmahoney.net/dc/silesia.html](https://mattmahoney.net/dc/silesia.html)

### Canterbury Corpus (3 MB)

Classic compression benchmark from 1997. Small files, useful for quick testing.

| File | Size | Type |
|------|------|------|
| alice29.txt | 152 KB | English text |
| asyoulik.txt | 125 KB | Shakespeare play |
| cp.html | 24 KB | HTML |
| fields.c | 11 KB | C source |
| grammar.lsp | 4 KB | Lisp source |
| kennedy.xls | 1 MB | Excel spreadsheet |
| lcet10.txt | 426 KB | Technical writing |
| plrabn12.txt | 481 KB | Poetry |
| ptt5 | 513 KB | FAX image |
| sum | 38 KB | SPARC executable |
| xargs.1 | 4 KB | Man page |

**Source:** [corpus.canterbury.ac.nz](https://corpus.canterbury.ac.nz/)

### Enwik (100 MB - 1 GB)

Wikipedia XML dumps. Standard benchmark for text compression.

| File | Size | Description |
|------|------|-------------|
| enwik8 | 100 MB | First 10^8 bytes of Wikipedia |
| enwik9 | 1 GB | First 10^9 bytes of Wikipedia |

**Source:** [mattmahoney.net/dc/textdata.html](https://mattmahoney.net/dc/textdata.html)

### Loghub (2-77 GB)

System log datasets for log compression and analysis research.

| Dataset | Size | Source |
|---------|------|--------|
| Android | 183 MB | Android framework logs |
| Apache | 5 MB | Apache web server |
| BGL | 709 MB | BlueGene/L supercomputer |
| Hadoop | 49 MB | Hadoop HDFS/YARN |
| HDFS | 1.5 GB | Hadoop distributed filesystem |
| HealthApp | 22 MB | Mobile health app |
| HPC | 32 MB | High-performance computing |
| Linux | 2 MB | Linux syslog |
| Mac | 16 MB | macOS console |
| OpenSSH | 71 MB | SSH server |
| OpenStack | 59 MB | OpenStack cloud |
| Spark | 33 MB | Apache Spark |
| Thunderbird | 30 GB | Supercomputer (full) |
| Windows | 27 MB | Windows event logs |
| Zookeeper | 10 MB | Apache Zookeeper |

**Source:** [github.com/logpai/loghub](https://github.com/logpai/loghub)
**Citation:** Zhu et al., "Loghub: A Large Collection of System Log Datasets", ISSRE 2023

## Large Log Datasets (50+ GB)

For stress testing and production-scale benchmarks:

### Thunderbird (~30 GB)
Logs from Thunderbird supercomputer at Sandia National Labs. Contains alert and non-alert messages.

```bash
wget http://0b4af6cdc2f0c5998459-c0245c5c937c5dedcca3f1764ecc9b2f.r43.cf2.rackcdn.com/hpc4/tbird2.gz
```

### Spirit (~30 GB)
Logs from Spirit supercomputer. Similar format to Thunderbird.

```bash
wget http://0b4af6cdc2f0c5998459-c0245c5c937c5dedcca3f1764ecc9b2f.r43.cf2.rackcdn.com/hpc4/spirit2.gz
```

### HDFS-v2 (~16 GB)
Extended HDFS logs from Loghub-2.0.

**Source:** [zenodo.org/records/8196385](https://zenodo.org/records/8196385)

## Domain-Specific Datasets

### DNA/Genomic

| Dataset | Size | Source |
|---------|------|--------|
| Human reference genome | 3.2 GB | NCBI/Ensembl |
| Example FASTA files | Varies | Included in `data/DNA/` |

### Firmware/Binary

For delta patching benchmarks, collect firmware images from:
- OpenWRT releases
- ESP32/Arduino firmware
- BIOS/UEFI updates

## Running Benchmarks

```bash
# Run standard benchmark suite
cd crystal-unified-public
.\benchmarks\run_benchmark.ps1

# Run on specific corpus
.\benchmarks\run_benchmark.ps1 -Corpus silesia
.\benchmarks\run_benchmark.ps1 -Corpus loghub

# Generate comparison report
.\benchmarks\run_benchmark.ps1 -Compare gzip,zstd,lz4
```

## Expected Results

### Log Files (Loghub)
- **Compression ratio:** 6-11% (89-94% reduction)
- **Compression speed:** 100-500 MB/s (parallel)
- **Decompression speed:** 500-1500 MB/s

### Silesia Corpus
- **Compression ratio:** 25-35% (varies by file type)
- **Best on:** XML, text, source code
- **Competitive on:** Binary, images

### Text (Enwik)
- **Compression ratio:** 20-30%
- **Note:** Dictionary-based compression excels here

## Adding Custom Datasets

Place files in the `data/` directory:

```
data/
├── silesia/           # Standard Silesia corpus
├── canterbury/        # Canterbury corpus
├── enwik/             # Wikipedia dumps
├── loghub/            # Loghub log files
├── large_logs/        # Thunderbird, Spirit, etc.
├── DNA/               # Genomic data
├── custom/            # Your custom datasets
│   ├── production_logs/
│   ├── firmware_images/
│   └── ...
```

## References

- [Silesia Corpus](https://sun.aei.polsl.pl/~sdeor/index.php?page=silesia)
- [Canterbury Corpus](https://corpus.canterbury.ac.nz/)
- [Large Text Compression Benchmark](https://mattmahoney.net/dc/text.html)
- [Loghub Paper](https://arxiv.org/abs/2008.06448)
- [Computer Failure Data Repository](https://www.usenix.org/cfdr)
