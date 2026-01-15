# Crystal Unified - Numeric Data

## Commands

### Compress Numeric Data
```bash
cuz compress <input> -t numeric [output] [-l level]
```

### Decompress
```bash
cuz decompress <input.cuz> [output]
```

## Examples

```bash
# Compress sensor readings
cuz compress sensor_data.csv -t numeric

# Decompress
cuz decompress sensor_data.csv.cuz
```
