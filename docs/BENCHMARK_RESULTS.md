# Crystal Unified Benchmark Results

**Generated:** 2026-01-13 15:54:33
**System:** DANE-PC001
**CPU:** Intel(R) Core(TM) i9-9900X CPU @ 3.50GHz (10 cores, 20 threads)
**RAM:** 128 GB
**CUZ Version:** v1.0

## Executive Summary
| Metric | Value |
|--------|-------|
| Files Tested | 34 |
| Total Data | 2.83 GB |
| Avg Compression Ratio | 12.3% |
| All Verified | Yes |

## Compression Results by Corpus

### CANTERBURY Corpus

| File | Original | Single | Parallel | Fast | Level 9 | Verified |
|------|----------|--------|----------|------|---------|----------|
| alice29.txt | 148.52 KB | 44.7% | 44.7% | 44.7% | 40.4% | Yes |
| asyoulik.txt | 122.25 KB | 50.2% | 50.2% | 50.2% | 45.1% | Yes |
| cp.html | 24.03 KB | 69.6% | 69.6% | 69.6% | 66.3% | Yes |
| fields.c | 10.89 KB | 106.4% | 106.4% | 106.4% | 102.6% | Yes |
| grammar.lsp | 3.63 KB | 259.3% | 259.3% | 259.3% | 256.7% | Yes |
| kennedy.xls | 1,005.61 KB | 12.0% | 12.0% | 12.0% | 11.2% | Yes |
| lcet10.txt | 416.75 KB | 39.1% | 39.1% | 39.1% | 32.7% | Yes |
| plrabn12.txt | 470.57 KB | 46.9% | 46.9% | 46.9% | 39.2% | Yes |
| ptt5 | 501.19 KB | 11.7% | 11.7% | 11.7% | 11.6% | Yes |
| sum | 37.34 KB | 57.4% | 57.4% | 57.4% | 54.6% | Yes |
| xargs.1 | 4.13 KB | 240.6% | 240.6% | 240.6% | 238.0% | Yes |
### ENWIK Corpus

| File | Original | Single | Parallel | Fast | Level 9 | Verified |
|------|----------|--------|----------|------|---------|----------|
| enwik8 | 95.37 MB | 41.3% | 41.3% | 41.3% | 33.7% | Yes |
### LOGHUB Corpus

| File | Original | Single | Parallel | Fast | Level 9 | Verified |
|------|----------|--------|----------|------|---------|----------|
| openstack_abnormal.log | 5.18 MB | 8.2% | 8.2% | 8.2% | 8.2% | Yes |
| openstack_normal1.log | 14.78 MB | 8.1% | 8.1% | 8.1% | 8.0% | Yes |
| Mac.log | 16.10 MB | 7.1% | 7.1% | 7.1% | 5.6% | Yes |
| HealthApp.log | 22.44 MB | 9.9% | 9.9% | 9.9% | 8.8% | Yes |
| HPC.log | 32.00 MB | 9.5% | 9.5% | 9.5% | 9.0% | Yes |
| openstack_normal2.log | 38.63 MB | 7.5% | 7.5% | 7.5% | 7.4% | Yes |
| SSH.log | 70.02 MB | 6.7% | 6.7% | 6.7% | 6.0% | Yes |
| Android.log | 183.36 MB | 10.2% | 10.2% | 10.2% | 7.7% | Yes |
| BGL.log | 708.76 MB | 8.7% | 8.7% | 8.7% | 8.2% | Yes |
| HDFS.log | 1.47 GB | 9.8% | 9.8% | 9.8% | 8.8% | Yes |
### SILESIA Corpus

| File | Original | Single | Parallel | Fast | Level 9 | Verified |
|------|----------|--------|----------|------|---------|----------|
| dickens | 9.72 MB | 43.2% | 43.2% | 43.2% | 35.5% | Yes |
| mozilla | 48.85 MB | 39.2% | 39.2% | 39.2% | 33.6% | Yes |
| mr | 9.51 MB | 38.9% | 38.9% | 38.9% | 34.7% | Yes |
| nci | 32.00 MB | 9.9% | 9.9% | 9.9% | 8.8% | Yes |
| ooffice | 5.87 MB | 58.5% | 58.5% | 58.5% | 47.1% | Yes |
| osdb | 9.62 MB | 37.2% | 37.2% | 37.2% | 33.5% | Yes |
| reymont | 6.32 MB | 32.7% | 32.7% | 32.7% | 26.2% | Yes |
| samba | 20.61 MB | 27.2% | 27.2% | 27.2% | 24.0% | Yes |
| sao | 6.92 MB | 86.3% | 86.3% | 86.3% | 72.4% | Yes |
| webster | 39.54 MB | 34.3% | 34.3% | 34.3% | 28.1% | Yes |
| xml | 5.10 MB | 13.7% | 13.7% | 13.7% | 10.7% | Yes |
| x-ray | 8.08 MB | 82.2% | 82.2% | 82.2% | 71.0% | Yes |
## Compression Speed Comparison

### Single-Thread vs Parallel (MB/s)

| File | Single | Parallel | Speedup |
|------|--------|----------|---------|
| openstack_abnormal.log | 78.8 | 116.7 | 1.5x |
| openstack_normal1.log | 114.8 | 227.3 | 2.0x |
| Mac.log | 109.3 | 218.2 | 2.0x |
| HealthApp.log | 117.9 | 252.0 | 2.1x |
| HPC.log | 107.0 | 287.4 | 2.7x |
| openstack_normal2.log | 127.2 | 335.2 | 2.6x |
| SSH.log | 119.8 | 362.8 | 3.0x |
| Android.log | 122.2 | 402.7 | 3.3x |
| BGL.log | 123.9 | 504.6 | 4.1x |
| HDFS.log | 126.7 | 483.4 | 3.8x |
| dickens | 60.9 | 146.0 | 2.4x |
| mozilla | 110.0 | 250.3 | 2.3x |
| mr | 87.7 | 143.4 | 1.6x |
| nci | 120.2 | 271.1 | 2.3x |
| ooffice | 59.8 | 84.3 | 1.4x |
| osdb | 82.9 | 133.5 | 1.6x |
| reymont | 65.8 | 90.0 | 1.4x |
| samba | 76.0 | 182.7 | 2.4x |
| sao | 66.0 | 88.4 | 1.3x |
| webster | 74.8 | 232.3 | 3.1x |
| xml | 74.6 | 99.9 | 1.3x |
| x-ray | 67.1 | 114.3 | 1.7x |
| enwik8 | 70.9 | 223.8 | 3.2x |
| alice29.txt | 5.9 | 6.1 | 1.0x |
| asyoulik.txt | 5.3 | 5.0 | 1.0x |
| cp.html | 1.1 | 1.0 | 0.9x |
| fields.c | 0.5 | 0.5 | 1.0x |
| grammar.lsp | 0.2 | 0.2 | 1.0x |
| kennedy.xls | 35.4 | 35.3 | 1.0x |
| lcet10.txt | 14.9 | 14.7 | 1.0x |
| plrabn12.txt | 16.3 | 16.8 | 1.0x |
| ptt5 | 21.0 | 20.4 | 1.0x |
| sum | 1.7 | 1.6 | 1.0x |
| xargs.1 | 0.2 | 0.2 | 1.0x |
### Decompression Speed (MB/s)

| File | Single | Parallel |
|------|--------|----------|
| openstack_abnormal.log | 173.2 | 176.8 |
| openstack_normal1.log | 418.4 | 414.7 |
| Mac.log | 454.4 | 458.7 |
| HealthApp.log | 544.0 | 576.6 |
| HPC.log | 687.3 | 688.1 |
| openstack_normal2.log | 715.8 | 754.0 |
| SSH.log | 970.0 | 966.2 |
| Android.log | 1,221.9 | 1,239.8 |
| BGL.log | 1,397.7 | 1,376.4 |
| HDFS.log | 1,385.0 | 1,371.9 |
| dickens | 280.5 | 290.3 |
| mozilla | 644.6 | 294.0 |
| mr | 281.0 | 278.2 |
| nci | 711.5 | 728.6 |
| ooffice | 116.0 | 152.3 |
| osdb | 237.9 | 222.1 |
| reymont | 163.3 | 164.5 |
| samba | 458.2 | 520.4 |
| sao | 163.6 | 165.2 |
| webster | 693.8 | 698.3 |
| xml | 173.5 | 180.8 |
| x-ray | 221.1 | 225.3 |
| enwik8 | 859.3 | 867.5 |
| alice29.txt | 6.4 | 6.6 |
| asyoulik.txt | 5.4 | 5.7 |
| cp.html | 1.1 | 1.1 |
| fields.c | 0.5 | 0.5 |
| grammar.lsp | 0.1 | 0.2 |
| kennedy.xls | 41.1 | 41.3 |
| lcet10.txt | 17.8 | 18.2 |
| plrabn12.txt | 19.9 | 20.3 |
| ptt5 | 21.0 | 22.2 |
| sum | 1.6 | 1.6 |
| xargs.1 | 0.2 | 0.2 |
## Memory Usage

| File | Compress (Peak) | Decompress (Peak) |
|------|-----------------|-------------------|
| openstack_abnormal.log | 0 B | 0 B |
| openstack_normal1.log | 0 B | 0 B |
| Mac.log | 0 B | 0 B |
| HealthApp.log | 0 B | 0 B |
| HPC.log | 0 B | 0 B |
| openstack_normal2.log | 0 B | 0 B |
| SSH.log | 0 B | 0 B |
| Android.log | 0 B | 0 B |
| BGL.log | 0 B | 0 B |
| HDFS.log | 0 B | 0 B |
| dickens | 0 B | 0 B |
| mozilla | 0 B | 0 B |
| mr | 0 B | 0 B |
| nci | 0 B | 0 B |
| ooffice | 0 B | 0 B |
| osdb | 0 B | 0 B |
| reymont | 0 B | 0 B |
| samba | 0 B | 0 B |
| sao | 0 B | 0 B |
| webster | 0 B | 0 B |
| xml | 0 B | 0 B |
| x-ray | 0 B | 0 B |
| enwik8 | 0 B | 0 B |
| alice29.txt | 0 B | 0 B |
| asyoulik.txt | 0 B | 0 B |
| cp.html | 0 B | 0 B |
| fields.c | 0 B | 0 B |
| grammar.lsp | 0 B | 0 B |
| kennedy.xls | 0 B | 0 B |
| lcet10.txt | 0 B | 0 B |
| plrabn12.txt | 0 B | 0 B |
| ptt5 | 0 B | 0 B |
| sum | 0 B | 0 B |
| xargs.1 | 0 B | 0 B |
## Search Performance

Search without decompression vs grep on original file.

| File | Pattern | CUZ Matches | Grep Matches | CUZ Time | Grep Time | Speedup | Accurate |
|------|---------|-------------|--------------|----------|-----------|---------|----------|
| openstack_abnormal.log | `error` | 0 | 0 | 38.0ms | 56.5ms | 1.5x | Yes |
| openstack_abnormal.log | `ERROR` | 0 | 0 | 37.0ms | 23.8ms | 0.6x | Yes |
| openstack_abnormal.log | `warning` | 0 | 0 | 35.8ms | 29.0ms | 0.8x | Yes |
| openstack_normal1.log | `error` | 1 | 1 | 45.1ms | 64.5ms | 1.4x | Yes |
| openstack_normal1.log | `ERROR` | 25 | 25 | 42.1ms | 61.2ms | 1.5x | Yes |
| openstack_normal1.log | `warning` | 0 | 0 | 42.7ms | 57.4ms | 1.3x | Yes |
| Mac.log | `error` | 6865 | 6865 | 50.9ms | 92.1ms | 1.8x | Yes |
| Mac.log | `ERROR` | 356 | 356 | 47.6ms | 90.9ms | 1.9x | Yes |
| Mac.log | `warning` | 129 | 129 | 48.1ms | 70.9ms | 1.5x | Yes |
| HealthApp.log | `error` | 212 | 212 | 54.2ms | 113.8ms | 2.1x | Yes |
| HealthApp.log | `ERROR` | 10 | 10 | 61.1ms | 148.1ms | 2.4x | Yes |
| HealthApp.log | `warning` | 0 | 0 | 55.2ms | 107.1ms | 1.9x | Yes |
| HPC.log | `error` | 109290 | 109290 | 64.7ms | 279.8ms | 4.3x | Yes |
| HPC.log | `ERROR` | 0 | 0 | 51.9ms | 364.4ms | 7.0x | Yes |
| HPC.log | `warning` | 50682 | 50682 | 54.6ms | 260.8ms | 4.8x | Yes |
| openstack_normal2.log | `error` | 7 | 7 | 62.7ms | 225.4ms | 3.6x | Yes |
| openstack_normal2.log | `ERROR` | 169 | 169 | 61.0ms | 158.9ms | 2.6x | Yes |
| openstack_normal2.log | `warning` | 0 | 0 | 62.9ms | 153.2ms | 2.4x | Yes |
| SSH.log | `error` | 1127 | 1127 | 82.5ms | 336.9ms | 4.1x | Yes |
| SSH.log | `ERROR` | 0 | 0 | 78.0ms | 327.2ms | 4.2x | Yes |
| SSH.log | `warning` | 12 | 12 | 80.3ms | 320.8ms | 4.0x | Yes |
| Android.log | `error` | 26626 | 26626 | 170.8ms | 957.4ms | 5.6x | Yes |
| Android.log | `ERROR` | 1431 | 1431 | 150.9ms | 846.8ms | 5.6x | Yes |
| Android.log | `warning` | 296 | 296 | 167.7ms | 826.8ms | 4.9x | Yes |
| BGL.log | `error` | 427963 | 427963 | 353.6ms | 5,127.2ms | 14.5x | Yes |
| BGL.log | `ERROR` | 113122 | 113122 | 317.6ms | 3,867.2ms | 12.2x | Yes |
| BGL.log | `warning` | 11228 | 11228 | 311.3ms | 3,030.4ms | 9.7x | Yes |
| HDFS.log | `error` | 5545 | 5545 | 689.6ms | 6,768.4ms | 9.8x | Yes |
| HDFS.log | `ERROR` | 0 | 0 | 643.5ms | 6,595.8ms | 10.3x | Yes |
| HDFS.log | `warning` | 0 | 0 | 613.7ms | 6,353.2ms | 10.4x | Yes |
| dickens | `the` | 83474 | 83474 | 44.7ms | 149.1ms | 3.3x | Yes |
| dickens | `and` | 62175 | 62175 | 43.1ms | 196.2ms | 4.6x | Yes |
| dickens | `that` | 22477 | 22477 | 41.4ms | 102.6ms | 2.5x | Yes |
| reymont | `the` | 12 | 12 | 48.1ms | 31.0ms | 0.6x | Yes |
| reymont | `and` | 22 | 22 | 50.0ms | 30.9ms | 0.6x | Yes |
| reymont | `that` | 0 | 0 | 55.3ms | 31.8ms | 0.6x | Yes |
| samba | `int` | 31689 | 31689 | 54.4ms | 238.9ms | 4.4x | Yes |
| samba | `return` | 17282 | 17282 | 58.5ms | 225.4ms | 3.9x | Yes |
| samba | `if` | 1180 | 36830 | 35.7ms | 301.7ms | 8.4x | **NO** |
| webster | `the` | 156381 | 156381 | 83.7ms | 602.1ms | 7.2x | Yes |
| webster | `and` | 73491 | 73491 | 76.8ms | 422.1ms | 5.5x | Yes |
| webster | `that` | 11897 | 11897 | 75.0ms | 557.4ms | 7.4x | Yes |
| xml | `<` | 0 | 51541 | 21.0ms | 136.6ms | 6.5x | **NO** |
| xml | `/>` | 0 | 776 | 20.6ms | 40.5ms | 2.0x | **NO** |
| xml | `xml` | 687 | 689 | 36.5ms | 41.8ms | 1.1x | **NO** |
| enwik8 | `the` | 199654 | 199654 | 256.9ms | 1,197.8ms | 4.7x | Yes |
| enwik8 | `and` | 175927 | 175927 | 247.7ms | 1,328.0ms | 5.4x | Yes |
| enwik8 | `that` | 52679 | 52679 | 170.7ms | 867.6ms | 5.1x | Yes |
| alice29.txt | `the` | 1473 | 1473 | 22.6ms | 2.1ms | 0.1x | Yes |
| alice29.txt | `and` | 742 | 742 | 21.9ms | 1.7ms | 0.1x | Yes |
| alice29.txt | `that` | 268 | 268 | 21.7ms | 1.5ms | 0.1x | Yes |
| asyoulik.txt | `the` | 997 | 997 | 22.5ms | 1.9ms | 0.1x | Yes |
| asyoulik.txt | `and` | 608 | 608 | 22.5ms | 1.7ms | 0.1x | Yes |
| asyoulik.txt | `that` | 272 | 272 | 22.6ms | 1.5ms | 0.1x | Yes |
| cp.html | `<html` | 0 | 0 | 22.0ms | 0.5ms | 0.0x | Yes |
| cp.html | `<div` | 0 | 0 | 20.9ms | 0.5ms | 0.0x | Yes |
| cp.html | `class=` | 0 | 0 | 20.9ms | 0.5ms | 0.0x | Yes |
| fields.c | `int` | 35 | 35 | 22.0ms | 0.5ms | 0.0x | Yes |
| fields.c | `return` | 29 | 29 | 22.1ms | 0.5ms | 0.0x | Yes |
| fields.c | `if` | 0 | 56 | 20.1ms | 0.5ms | 0.0x | **NO** |
| grammar.lsp | `int` | 0 | 0 | 21.7ms | 0.4ms | 0.0x | Yes |
| grammar.lsp | `return` | 0 | 0 | 21.6ms | 0.4ms | 0.0x | Yes |
| grammar.lsp | `if` | 0 | 1 | 19.9ms | 0.4ms | 0.0x | **NO** |
| lcet10.txt | `the` | 3337 | 3337 | 23.8ms | 4.4ms | 0.2x | Yes |
| lcet10.txt | `and` | 1845 | 1845 | 24.7ms | 3.7ms | 0.1x | Yes |
| lcet10.txt | `that` | 961 | 961 | 23.9ms | 3.2ms | 0.1x | Yes |
| plrabn12.txt | `the` | 4241 | 4241 | 26.1ms | 101.8ms | 3.9x | Yes |
| plrabn12.txt | `and` | 2978 | 2978 | 24.6ms | 4.7ms | 0.2x | Yes |
| plrabn12.txt | `that` | 531 | 531 | 24.8ms | 3.7ms | 0.1x | Yes |
| xargs.1 | `the` | 36 | 36 | 21.6ms | 0.4ms | 0.0x | Yes |
| xargs.1 | `and` | 26 | 26 | 21.2ms | 0.4ms | 0.0x | Yes |
| xargs.1 | `that` | 1 | 1 | 21.1ms | 0.4ms | 0.0x | Yes |
### Search Summary

- **Total searches:** 72
- **Accurate results:** 66 / 72 (92%)
- **Average speedup:** 3.0x faster than grep

## Detailed Results

### openstack_abnormal.log

- **Path:** openstack_abnormal.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 5.18 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 437.79 KB | 8.25% | 79 MB/s | 173 MB/s | 0 B | Yes |
| Parallel | 437.79 KB | 8.25% | 117 MB/s | 177 MB/s | 0 B | Yes |
| Fast | 437.79 KB | 8.25% | 46 MB/s | 179 MB/s | 0 B | Yes |
| Level 9 | 437.04 KB | 8.23% | 56 MB/s | 171 MB/s | 0 B | Yes |
### openstack_normal1.log

- **Path:** openstack_normal1.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 14.78 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 1.19 MB | 8.07% | 115 MB/s | 418 MB/s | 0 B | Yes |
| Parallel | 1.19 MB | 8.07% | 227 MB/s | 415 MB/s | 0 B | Yes |
| Fast | 1.19 MB | 8.07% | 117 MB/s | 423 MB/s | 0 B | Yes |
| Level 9 | 1.18 MB | 7.96% | 69 MB/s | 404 MB/s | 0 B | Yes |
### Mac.log

- **Path:** Mac.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 16.10 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 1.15 MB | 7.14% | 109 MB/s | 454 MB/s | 0 B | Yes |
| Parallel | 1.15 MB | 7.14% | 218 MB/s | 459 MB/s | 0 B | Yes |
| Fast | 1.15 MB | 7.14% | 110 MB/s | 460 MB/s | 0 B | Yes |
| Level 9 | 921.32 KB | 5.59% | 67 MB/s | 456 MB/s | 0 B | Yes |
### HealthApp.log

- **Path:** HealthApp.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 22.44 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 2.23 MB | 9.93% | 118 MB/s | 544 MB/s | 0 B | Yes |
| Parallel | 2.23 MB | 9.93% | 252 MB/s | 577 MB/s | 0 B | Yes |
| Fast | 2.23 MB | 9.93% | 118 MB/s | 563 MB/s | 0 B | Yes |
| Level 9 | 1.96 MB | 8.75% | 65 MB/s | 575 MB/s | 0 B | Yes |
### HPC.log

- **Path:** HPC.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 32.00 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 3.05 MB | 9.54% | 107 MB/s | 687 MB/s | 0 B | Yes |
| Parallel | 3.05 MB | 9.54% | 287 MB/s | 688 MB/s | 0 B | Yes |
| Fast | 3.05 MB | 9.54% | 107 MB/s | 698 MB/s | 0 B | Yes |
| Level 9 | 2.89 MB | 9.03% | 57 MB/s | 707 MB/s | 0 B | Yes |
### openstack_normal2.log

- **Path:** openstack_normal2.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 38.63 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 2.89 MB | 7.49% | 127 MB/s | 716 MB/s | 0 B | Yes |
| Parallel | 2.89 MB | 7.49% | 335 MB/s | 754 MB/s | 0 B | Yes |
| Fast | 2.89 MB | 7.49% | 131 MB/s | 762 MB/s | 0 B | Yes |
| Level 9 | 2.85 MB | 7.37% | 74 MB/s | 757 MB/s | 0 B | Yes |
### SSH.log

- **Path:** SSH.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 70.02 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 4.67 MB | 6.67% | 120 MB/s | 970 MB/s | 0 B | Yes |
| Parallel | 4.67 MB | 6.67% | 363 MB/s | 966 MB/s | 0 B | Yes |
| Fast | 4.67 MB | 6.67% | 119 MB/s | 875 MB/s | 0 B | Yes |
| Level 9 | 4.23 MB | 6.04% | 70 MB/s | 970 MB/s | 0 B | Yes |
### Android.log

- **Path:** Android.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 183.36 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 18.73 MB | 10.22% | 122 MB/s | 1 GB/s | 0 B | Yes |
| Parallel | 18.73 MB | 10.22% | 403 MB/s | 1 GB/s | 0 B | Yes |
| Fast | 18.73 MB | 10.22% | 122 MB/s | 1 GB/s | 0 B | Yes |
| Level 9 | 14.06 MB | 7.67% | 66 MB/s | 1 GB/s | 0 B | Yes |
### BGL.log

- **Path:** BGL.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 708.76 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 61.65 MB | 8.70% | 124 MB/s | 1 GB/s | 0 B | Yes |
| Parallel | 61.65 MB | 8.70% | 505 MB/s | 1 GB/s | 0 B | Yes |
| Fast | 61.65 MB | 8.70% | 125 MB/s | 1 GB/s | 0 B | Yes |
| Level 9 | 57.91 MB | 8.17% | 66 MB/s | 1 GB/s | 0 B | Yes |
### HDFS.log

- **Path:** HDFS.log
- **Corpus:** loghub
- **Category:** Log
- **Original Size:** 1.47 GB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 147.48 MB | 9.80% | 127 MB/s | 1 GB/s | 0 B | Yes |
| Parallel | 147.48 MB | 9.80% | 483 MB/s | 1 GB/s | 0 B | Yes |
| Fast | 147.48 MB | 9.80% | 127 MB/s | 1 GB/s | 0 B | Yes |
| Level 9 | 132.57 MB | 8.81% | 70 MB/s | 1 GB/s | 0 B | Yes |
### dickens

- **Path:** silesia\dickens
- **Corpus:** silesia
- **Category:** Text
- **Original Size:** 9.72 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 4.20 MB | 43.16% | 61 MB/s | 281 MB/s | 0 B | Yes |
| Parallel | 4.20 MB | 43.16% | 146 MB/s | 290 MB/s | 0 B | Yes |
| Fast | 4.20 MB | 43.16% | 60 MB/s | 295 MB/s | 0 B | Yes |
| Level 9 | 3.45 MB | 35.45% | 29 MB/s | 287 MB/s | 0 B | Yes |
### mozilla

- **Path:** silesia\mozilla
- **Corpus:** silesia
- **Category:** Binary
- **Original Size:** 48.85 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 19.13 MB | 39.15% | 110 MB/s | 645 MB/s | 0 B | Yes |
| Parallel | 19.13 MB | 39.15% | 250 MB/s | 294 MB/s | 0 B | Yes |
| Fast | 19.13 MB | 39.15% | 111 MB/s | 661 MB/s | 0 B | Yes |
| Level 9 | 16.43 MB | 33.63% | 48 MB/s | 657 MB/s | 0 B | Yes |
### mr

- **Path:** silesia\mr
- **Corpus:** silesia
- **Category:** Image
- **Original Size:** 9.51 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 3.70 MB | 38.91% | 88 MB/s | 281 MB/s | 0 B | Yes |
| Parallel | 3.70 MB | 38.91% | 143 MB/s | 278 MB/s | 0 B | Yes |
| Fast | 3.70 MB | 38.91% | 87 MB/s | 290 MB/s | 0 B | Yes |
| Level 9 | 3.30 MB | 34.72% | 39 MB/s | 293 MB/s | 0 B | Yes |
### nci

- **Path:** silesia\nci
- **Corpus:** silesia
- **Category:** Chemistry
- **Original Size:** 32.00 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 3.16 MB | 9.89% | 120 MB/s | 711 MB/s | 0 B | Yes |
| Parallel | 3.16 MB | 9.89% | 271 MB/s | 729 MB/s | 0 B | Yes |
| Fast | 3.16 MB | 9.89% | 120 MB/s | 699 MB/s | 0 B | Yes |
| Level 9 | 2.81 MB | 8.77% | 69 MB/s | 719 MB/s | 0 B | Yes |
### ooffice

- **Path:** silesia\ooffice
- **Corpus:** silesia
- **Category:** Binary
- **Original Size:** 5.87 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 3.43 MB | 58.54% | 60 MB/s | 116 MB/s | 0 B | Yes |
| Parallel | 3.43 MB | 58.54% | 84 MB/s | 152 MB/s | 0 B | Yes |
| Fast | 3.43 MB | 58.54% | 72 MB/s | 152 MB/s | 0 B | Yes |
| Level 9 | 2.77 MB | 47.15% | 33 MB/s | 142 MB/s | 0 B | Yes |
### osdb

- **Path:** silesia\osdb
- **Corpus:** silesia
- **Category:** Database
- **Original Size:** 9.62 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 3.58 MB | 37.19% | 83 MB/s | 238 MB/s | 0 B | Yes |
| Parallel | 3.58 MB | 37.19% | 133 MB/s | 222 MB/s | 0 B | Yes |
| Fast | 3.58 MB | 37.19% | 83 MB/s | 246 MB/s | 0 B | Yes |
| Level 9 | 3.23 MB | 33.53% | 39 MB/s | 250 MB/s | 0 B | Yes |
### reymont

- **Path:** silesia\reymont
- **Corpus:** silesia
- **Category:** Text
- **Original Size:** 6.32 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 2.07 MB | 32.71% | 66 MB/s | 163 MB/s | 0 B | Yes |
| Parallel | 2.07 MB | 32.71% | 90 MB/s | 165 MB/s | 0 B | Yes |
| Fast | 2.07 MB | 32.71% | 66 MB/s | 168 MB/s | 0 B | Yes |
| Level 9 | 1.66 MB | 26.20% | 31 MB/s | 164 MB/s | 0 B | Yes |
### samba

- **Path:** silesia\samba
- **Corpus:** silesia
- **Category:** Source
- **Original Size:** 20.61 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 5.61 MB | 27.24% | 76 MB/s | 458 MB/s | 0 B | Yes |
| Parallel | 5.61 MB | 27.24% | 183 MB/s | 520 MB/s | 0 B | Yes |
| Fast | 5.61 MB | 27.24% | 76 MB/s | 504 MB/s | 0 B | Yes |
| Level 9 | 4.95 MB | 24.01% | 45 MB/s | 517 MB/s | 0 B | Yes |
### sao

- **Path:** silesia\sao
- **Corpus:** silesia
- **Category:** Binary
- **Original Size:** 6.92 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 5.97 MB | 86.34% | 66 MB/s | 164 MB/s | 0 B | Yes |
| Parallel | 5.97 MB | 86.34% | 88 MB/s | 165 MB/s | 0 B | Yes |
| Fast | 5.97 MB | 86.34% | 68 MB/s | 168 MB/s | 0 B | Yes |
| Level 9 | 5.01 MB | 72.40% | 29 MB/s | 151 MB/s | 0 B | Yes |
### webster

- **Path:** silesia\webster
- **Corpus:** silesia
- **Category:** Text
- **Original Size:** 39.54 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 13.56 MB | 34.30% | 75 MB/s | 694 MB/s | 0 B | Yes |
| Parallel | 13.56 MB | 34.30% | 232 MB/s | 698 MB/s | 0 B | Yes |
| Fast | 13.56 MB | 34.30% | 76 MB/s | 687 MB/s | 0 B | Yes |
| Level 9 | 11.11 MB | 28.11% | 38 MB/s | 708 MB/s | 0 B | Yes |
### xml

- **Path:** silesia\xml
- **Corpus:** silesia
- **Category:** XML
- **Original Size:** 5.10 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 714.92 KB | 13.70% | 75 MB/s | 173 MB/s | 0 B | Yes |
| Parallel | 714.92 KB | 13.70% | 100 MB/s | 181 MB/s | 0 B | Yes |
| Fast | 714.92 KB | 13.70% | 76 MB/s | 180 MB/s | 0 B | Yes |
| Level 9 | 559.52 KB | 10.72% | 47 MB/s | 185 MB/s | 0 B | Yes |
### x-ray

- **Path:** silesia\x-ray
- **Corpus:** silesia
- **Category:** Image
- **Original Size:** 8.08 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 6.64 MB | 82.16% | 67 MB/s | 221 MB/s | 0 B | Yes |
| Parallel | 6.64 MB | 82.16% | 114 MB/s | 225 MB/s | 0 B | Yes |
| Fast | 6.64 MB | 82.16% | 70 MB/s | 225 MB/s | 0 B | Yes |
| Level 9 | 5.74 MB | 71.01% | 31 MB/s | 208 MB/s | 0 B | Yes |
### enwik8

- **Path:** enwik\enwik8
- **Corpus:** enwik
- **Category:** Text
- **Original Size:** 95.37 MB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 39.41 MB | 41.32% | 71 MB/s | 859 MB/s | 0 B | Yes |
| Parallel | 39.41 MB | 41.32% | 224 MB/s | 868 MB/s | 0 B | Yes |
| Fast | 39.41 MB | 41.32% | 72 MB/s | 863 MB/s | 0 B | Yes |
| Level 9 | 32.10 MB | 33.66% | 35 MB/s | 869 MB/s | 0 B | Yes |
### alice29.txt

- **Path:** canterbury\alice29.txt
- **Corpus:** canterbury
- **Category:** Text
- **Original Size:** 148.52 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 66.37 KB | 44.69% | 6 MB/s | 6 MB/s | 0 B | Yes |
| Parallel | 66.37 KB | 44.69% | 6 MB/s | 7 MB/s | 0 B | Yes |
| Fast | 66.37 KB | 44.69% | 6 MB/s | 7 MB/s | 0 B | Yes |
| Level 9 | 60.06 KB | 40.44% | 5 MB/s | 7 MB/s | 0 B | Yes |
### asyoulik.txt

- **Path:** canterbury\asyoulik.txt
- **Corpus:** canterbury
- **Category:** Text
- **Original Size:** 122.25 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 61.35 KB | 50.18% | 5 MB/s | 5 MB/s | 0 B | Yes |
| Parallel | 61.35 KB | 50.18% | 5 MB/s | 6 MB/s | 0 B | Yes |
| Fast | 61.35 KB | 50.18% | 5 MB/s | 5 MB/s | 0 B | Yes |
| Level 9 | 55.13 KB | 45.09% | 5 MB/s | 5 MB/s | 0 B | Yes |
### cp.html

- **Path:** canterbury\cp.html
- **Corpus:** canterbury
- **Category:** HTML
- **Original Size:** 24.03 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 16.73 KB | 69.63% | 1 MB/s | 1 MB/s | 0 B | Yes |
| Parallel | 16.73 KB | 69.63% | 1 MB/s | 1 MB/s | 0 B | Yes |
| Fast | 16.73 KB | 69.63% | 1 MB/s | 1 MB/s | 0 B | Yes |
| Level 9 | 15.93 KB | 66.29% | 1 MB/s | 1 MB/s | 0 B | Yes |
### fields.c

- **Path:** canterbury\fields.c
- **Corpus:** canterbury
- **Category:** Source
- **Original Size:** 10.89 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 11.59 KB | 106.43% | 504 KB/s | 488 KB/s | 0 B | Yes |
| Parallel | 11.59 KB | 106.43% | 496 KB/s | 518 KB/s | 0 B | Yes |
| Fast | 11.59 KB | 106.43% | 518 KB/s | 501 KB/s | 0 B | Yes |
| Level 9 | 11.17 KB | 102.59% | 503 KB/s | 507 KB/s | 0 B | Yes |
### grammar.lsp

- **Path:** canterbury\grammar.lsp
- **Corpus:** canterbury
- **Category:** Source
- **Original Size:** 3.63 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 9.42 KB | 259.29% | 163 KB/s | 135 KB/s | 0 B | Yes |
| Parallel | 9.42 KB | 259.29% | 171 KB/s | 169 KB/s | 0 B | Yes |
| Fast | 9.42 KB | 259.29% | 178 KB/s | 174 KB/s | 0 B | Yes |
| Level 9 | 9.33 KB | 256.73% | 181 KB/s | 84 KB/s | 0 B | Yes |
### kennedy.xls

- **Path:** canterbury\kennedy.xls
- **Corpus:** canterbury
- **Category:** Binary
- **Original Size:** 1,005.61 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 120.80 KB | 12.01% | 35 MB/s | 41 MB/s | 0 B | Yes |
| Parallel | 120.80 KB | 12.01% | 35 MB/s | 41 MB/s | 0 B | Yes |
| Fast | 120.80 KB | 12.01% | 36 MB/s | 42 MB/s | 0 B | Yes |
| Level 9 | 112.21 KB | 11.16% | 25 MB/s | 35 MB/s | 0 B | Yes |
### lcet10.txt

- **Path:** canterbury\lcet10.txt
- **Corpus:** canterbury
- **Category:** Text
- **Original Size:** 416.75 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 162.92 KB | 39.09% | 15 MB/s | 18 MB/s | 0 B | Yes |
| Parallel | 162.92 KB | 39.09% | 15 MB/s | 18 MB/s | 0 B | Yes |
| Fast | 162.92 KB | 39.09% | 15 MB/s | 19 MB/s | 0 B | Yes |
| Level 9 | 136.34 KB | 32.71% | 12 MB/s | 18 MB/s | 0 B | Yes |
### plrabn12.txt

- **Path:** canterbury\plrabn12.txt
- **Corpus:** canterbury
- **Category:** Text
- **Original Size:** 470.57 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 220.85 KB | 46.93% | 16 MB/s | 20 MB/s | 0 B | Yes |
| Parallel | 220.85 KB | 46.93% | 17 MB/s | 20 MB/s | 0 B | Yes |
| Fast | 220.85 KB | 46.93% | 16 MB/s | 20 MB/s | 0 B | Yes |
| Level 9 | 184.55 KB | 39.22% | 12 MB/s | 21 MB/s | 0 B | Yes |
### ptt5

- **Path:** canterbury\ptt5
- **Corpus:** canterbury
- **Category:** Image
- **Original Size:** 501.19 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 58.57 KB | 11.69% | 21 MB/s | 21 MB/s | 0 B | Yes |
| Parallel | 58.57 KB | 11.69% | 20 MB/s | 22 MB/s | 0 B | Yes |
| Fast | 58.57 KB | 11.69% | 21 MB/s | 22 MB/s | 0 B | Yes |
| Level 9 | 58.25 KB | 11.62% | 18 MB/s | 22 MB/s | 0 B | Yes |
### sum

- **Path:** canterbury\sum
- **Corpus:** canterbury
- **Category:** Binary
- **Original Size:** 37.34 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 21.42 KB | 57.35% | 2 MB/s | 2 MB/s | 0 B | Yes |
| Parallel | 21.42 KB | 57.35% | 2 MB/s | 2 MB/s | 0 B | Yes |
| Fast | 21.42 KB | 57.35% | 2 MB/s | 2 MB/s | 0 B | Yes |
| Level 9 | 20.39 KB | 54.59% | 2 MB/s | 2 MB/s | 0 B | Yes |
### xargs.1

- **Path:** canterbury\xargs.1
- **Corpus:** canterbury
- **Category:** Text
- **Original Size:** 4.13 KB

| Mode | Compressed | Ratio | Compress | Decompress | Memory | Verified |
|------|------------|-------|----------|------------|--------|----------|
| Single-thread | 9.93 KB | 240.62% | 195 KB/s | 184 KB/s | 0 B | Yes |
| Parallel | 9.93 KB | 240.62% | 187 KB/s | 194 KB/s | 0 B | Yes |
| Fast | 9.93 KB | 240.62% | 42 KB/s | 188 KB/s | 0 B | Yes |
| Level 9 | 9.82 KB | 237.97% | 200 KB/s | 198 KB/s | 0 B | Yes |
---

*Generated by Crystal Unified Benchmark Script*
