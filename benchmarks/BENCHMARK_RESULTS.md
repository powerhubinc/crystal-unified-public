# Crystal Unified Benchmark Results

**Date:** 2026-01-13 14:00:08
**System:** DANE-PC001
**CPU:** Intel(R) Core(TM) i9-9900X CPU @ 3.50GHz

## Summary

| File | Original | Auto | Fast | Level 6 | Parallel |
|------|----------|------|------|---------|----------|| openstack_abnormal.log | 5.18 MB | 10.2% | 8.2% | 9.1% | 8.2% |
| openstack_normal1.log | 14.78 MB | 10.0% | 8.1% | 8.9% | 8.1% |
| Mac.log | 16.10 MB | 6.2% | 7.1% | 6.6% | 7.1% |
| HealthApp.log | 22.44 MB | 11.6% | 9.9% | 10.5% | 9.9% |
| HPC.log | 32.00 MB | 10.1% | 9.5% | 10.6% | 9.5% |
| openstack_normal2.log | 38.63 MB | 9.3% | 7.5% | 8.2% | 7.5% |
| SSH.log | 70.02 MB | 6.7% | 6.7% | 7.3% | 6.7% |
| Android.log | 183.36 MB | 8.3% | 10.2% | 9.0% | 10.2% |
| BGL.log | 708.76 MB | 10.7% | 8.7% | 9.7% | 8.7% |

## Compression Speed (MB/s)

| File | Auto | Fast | Level 6 | Parallel |
|------|------|------|---------|----------|| openstack_abnormal.log | 103.8 | 105.5 | 86.1 | 157.4 |
| openstack_normal1.log | 137.0 | 112.3 | 107.4 | 233.7 |
| Mac.log | 158.0 | 114.0 | 101.4 | 253.3 |
| HealthApp.log | 135.2 | 119.0 | 108.1 | 273.7 |
| HPC.log | 250.1 | 106.5 | 101.0 | 323.4 |
| openstack_normal2.log | 158.6 | 127.1 | 111.5 | 340.2 |
| SSH.log | 337.7 | 119.8 | 107.5 | 383.1 |
| Android.log | 226.6 | 115.2 | 110.0 | 399.5 |
| BGL.log | 196.9 | 122.1 | 109.6 | 476.5 |

## Decompression Speed (MB/s)

| File | Auto | Fast | Level 6 | Parallel |
|------|------|------|---------|----------|| openstack_abnormal.log | 149.2 | 264.3 | 277.2 | 284.8 |
| openstack_normal1.log | 179.8 | 607.9 | 579.7 | 580.2 |
| Mac.log | 649.5 | 657.9 | 611.1 | 683.8 |
| HealthApp.log | 150.3 | 696.1 | 695.9 | 759.0 |
| HPC.log | 859.2 | 892.2 | 852.3 | 877.7 |
| openstack_normal2.log | 148.0 | 950.5 | 969.2 | 952.2 |
| SSH.log | 1,105.3 | 1,137.4 | 1,071.8 | 1,076.9 |
| Android.log | 1,261.6 | 1,275.6 | 1,249.5 | 1,244.6 |
| BGL.log | 128.8 | 1,358.6 | 1,117.5 | 1,313.6 |

## Detailed Results
### openstack_abnormal.log

- **Original size:** 5.18 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 541.39 KB | 10.20% | 0.05s | 0.03s | True |
| Fast | 437.79 KB | 8.25% | 0.05s | 0.02s | True |
| Level 6 | 481.70 KB | 9.07% | 0.06s | 0.02s | True |
| Parallel | 437.79 KB | 8.25% | 0.03s | 0.02s | True |
### openstack_normal1.log

- **Original size:** 14.78 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 1.49 MB | 10.05% | 0.11s | 0.08s | True |
| Fast | 1.19 MB | 8.07% | 0.13s | 0.02s | True |
| Level 6 | 1.31 MB | 8.88% | 0.14s | 0.03s | True |
| Parallel | 1.19 MB | 8.07% | 0.06s | 0.03s | True |
### Mac.log

- **Original size:** 16.10 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 1,019.38 KB | 6.18% | 0.10s | 0.02s | True |
| Fast | 1.15 MB | 7.14% | 0.14s | 0.02s | True |
| Level 6 | 1.07 MB | 6.62% | 0.16s | 0.03s | True |
| Parallel | 1.15 MB | 7.14% | 0.06s | 0.02s | True |
### HealthApp.log

- **Original size:** 22.44 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 2.60 MB | 11.59% | 0.17s | 0.15s | True |
| Fast | 2.23 MB | 9.93% | 0.19s | 0.03s | True |
| Level 6 | 2.35 MB | 10.46% | 0.21s | 0.03s | True |
| Parallel | 2.23 MB | 9.93% | 0.08s | 0.03s | True |
### HPC.log

- **Original size:** 32.00 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 3.23 MB | 10.10% | 0.13s | 0.04s | True |
| Fast | 3.05 MB | 9.54% | 0.30s | 0.04s | True |
| Level 6 | 3.40 MB | 10.61% | 0.32s | 0.04s | True |
| Parallel | 3.05 MB | 9.54% | 0.10s | 0.04s | True |
### openstack_normal2.log

- **Original size:** 38.63 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 3.60 MB | 9.31% | 0.24s | 0.26s | True |
| Fast | 2.89 MB | 7.49% | 0.30s | 0.04s | True |
| Level 6 | 3.17 MB | 8.21% | 0.35s | 0.04s | True |
| Parallel | 2.89 MB | 7.49% | 0.11s | 0.04s | True |
### SSH.log

- **Original size:** 70.02 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 4.70 MB | 6.72% | 0.21s | 0.06s | True |
| Fast | 4.67 MB | 6.67% | 0.58s | 0.06s | True |
| Level 6 | 5.08 MB | 7.26% | 0.65s | 0.07s | True |
| Parallel | 4.67 MB | 6.67% | 0.18s | 0.07s | True |
### Android.log

- **Original size:** 183.36 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 15.20 MB | 8.29% | 0.81s | 0.15s | True |
| Fast | 18.73 MB | 10.22% | 1.59s | 0.14s | True |
| Level 6 | 16.44 MB | 8.97% | 1.67s | 0.15s | True |
| Parallel | 18.73 MB | 10.22% | 0.46s | 0.15s | True |
### BGL.log

- **Original size:** 708.76 MB

| Mode | Compressed | Ratio | Compress Time | Decompress Time | Verified |
|------|------------|-------|---------------|-----------------|----------|
| Auto | 76.12 MB | 10.74% | 3.60s | 5.50s | True |
| Fast | 61.65 MB | 8.70% | 5.80s | 0.52s | True |
| Level 6 | 68.79 MB | 9.71% | 6.47s | 0.63s | True |
| Parallel | 61.65 MB | 8.70% | 1.49s | 0.54s | True |

---
Generated by Crystal Unified Benchmark Script
