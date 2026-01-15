# Crystal Unified - Genomic Data Compression

## Overview

Crystal provides two DNA compression modes:

1. **Standalone 2-bit encoding** - 4:1 compression for any DNA/RNA data
2. **Reference-based compression** - 50-100x compression for same-species samples

## Reference-Based Compression

For maximum compression of genomic data from the same species (e.g., human samples against GRCh38), use reference-based compression.

### Build Reference Index

```bash
cuz dna-index <reference.fa> <output.cdni>
```

The index contains:
- 21-mer position lookup table
- 2-bit encoded reference sequence
- N-position runs (for gaps/unknown bases)
- Lowercase runs (for soft-masked regions)

Example:
```bash
cuz dna-index GRCh38.fa GRCh38.cdni
# Human genome: ~6.5 GB index (one-time build)
```

### Compress Against Reference

```bash
cuz dna-compress <sample.fa> -r <reference.cdni> [-o output.cdnr]
```

Output format (v4):
- Match/Insert segments (delta from reference)
- N-position runs
- Lowercase position runs
- FASTA metadata (headers + line width)

Example:
```bash
cuz dna-compress sample.fa -r GRCh38.cdni
# Human genome: 3.3 GB -> 58 MB (1.7% ratio)
```

### Decompress

```bash
cuz dna-decompress <input.cdnr> -r <reference.cdni> [-o output.fa]
```

Decompression is byte-perfect:
- FASTA headers preserved exactly
- Line wrapping preserved (typically 80 chars)
- N bases preserved (not converted to A)
- Lowercase bases preserved (soft-masking)

### Performance

| Genome | Original | Compressed | Ratio | Verified |
|--------|----------|------------|-------|----------|
| Human (GRCh38) | 3.34 GB | 58 MB | 1.7% | SHA256 match |
| E. coli | 4.7 MB | 40 bytes | 0.0008% | Byte-perfect |

## Standalone 2-bit Encoding

For DNA data without a reference genome, use 2-bit encoding:

```bash
cuz compress <input> -t dna [-o output.cuz]
```

This mode:
- Encodes A=00, C=01, G=10, T=11
- Achieves 4:1 compression on pure ACGT
- Handles N and lowercase with run-length encoding

```bash
cuz compress sequences.fasta -t dna
cuz decompress sequences.fasta.cuz
```

## File Formats

| Extension | Purpose |
|-----------|---------|
| `.cdni` | Reference index (21-mer lookup + 2-bit sequence) |
| `.cdnr` | Reference-compressed sample (delta + metadata) |
| `.cuz` | Standalone compressed (2-bit + zstd) |

## Lossless Guarantees

Version 4 format preserves:

| Feature | Preserved |
|---------|-----------|
| Sequence data (ACGT) | Yes |
| N bases (unknown) | Yes |
| Lowercase (soft-mask) | Yes |
| FASTA headers | Yes |
| Line wrapping | Yes |
| Header order | Yes |

Verification:
```bash
# Compress
cuz dna-compress original.fa -r ref.cdni -o compressed.cdnr

# Decompress
cuz dna-decompress compressed.cdnr -r ref.cdni -o restored.fa

# Verify (files will be byte-identical)
sha256sum original.fa restored.fa
```

## Reference-Based Compression (Extreme Ratios)

When you have a reference genome, you can achieve extreme compression ratios by compressing samples as differences from the reference.

### Build Reference Index
```bash
cuz dna-index <reference.fa> [output.cdni]
```

### Compress with Reference
```bash
cuz dna-compress <input.fa> -r <reference.cdni> [-o output.cdnr]
```

### Decompress with Reference
```bash
cuz dna-decompress <input.cdnr> -r <reference.cdni> [-o output.fa]
```

### Examples
```bash
# Build index from human reference genome (one-time)
cuz dna-index hg38.fa hg38.cdni

# Compress sample using reference
cuz dna-compress patient_sample.fa -r hg38.cdni

# Decompress
cuz dna-decompress patient_sample.fa.cdnr -r hg38.cdni
```

### Compression Performance

| Scenario | Input | Compressed | Ratio |
|----------|-------|------------|-------|
| Human genome (vs hg38) | 3.3 GB | ~30 KB | 0.001% |
| E.coli (vs reference) | 4.7 MB | 40 B | 0.001% |
| Novel regions only | - | - | ~1:1 |

### Notes

- Reference index must be built once and can be reused
- Index size is approximately 2x the reference genome size
- Both compressor and decompressor need access to the same reference index
- Novel sequences (not in reference) are stored separately
- N bases are encoded as A in the compressed output
