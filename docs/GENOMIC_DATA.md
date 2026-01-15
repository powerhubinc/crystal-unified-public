# Crystal Unified - Genomic Data

## Commands

### Compress DNA/RNA
```bash
cuz compress <input> -t dna [output] [-l level]
```

### Decompress
```bash
cuz decompress <input.cuz> [output]
```

## Examples

```bash
# Compress FASTA file
cuz compress sequences.fasta -t dna

# Decompress
cuz decompress sequences.fasta.cuz
```
