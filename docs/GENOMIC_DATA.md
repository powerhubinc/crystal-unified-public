# Crystal Unified - Genomic Data

## Standard Compression (4:1)

### Compress DNA/RNA
```bash
cuz compress <input> -t dna [output] [-l level]
```

### Decompress
```bash
cuz decompress <input.cuz> [output]
```

### Examples
```bash
# Compress FASTA file
cuz compress sequences.fasta -t dna

# Decompress
cuz decompress sequences.fasta.cuz
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
