# Crystal Unified - Firmware & Binary

## Commands

### Compress Firmware
```bash
cuz firmware <input> [output] [-l level] [-b block_kb]
```

### Append Data
```bash
cuz append-raw <archive.cuzb> <data>
```

### Patch Block
```bash
cuz patch <archive.cuzb> <block#> <data>
```

### Create Delta
```bash
cuz delta <old> <new> -o <patch.cuzd>
```

### Apply Delta
```bash
cuz apply <file> <patch.cuzd> [-o output]
```

## Examples

```bash
# Compress firmware
cuz firmware image.bin

# Create delta patch
cuz delta old.bin new.bin -o update.cuzd

# Apply patch
cuz apply old.bin update.cuzd -o patched.bin
```
