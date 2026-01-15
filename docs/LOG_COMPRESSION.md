# Crystal Unified - Log Compression

## Commands

### Compress
```bash
cuz compress <input> [output] [-l level] [-j] [-s]
```

### Decompress
```bash
cuz decompress <input.cuz> [output]
```

### Search
```bash
cuz search <file.cuz> <term> [-n max] [--count]
```

### Append
```bash
cuz append <archive.cuz> <new_data.txt|->
```

## Examples

```bash
# Compress log file
cuz compress app.log

# Search in compressed file
cuz search app.log.cuz "ERROR"

# Append new data
cuz append app.log.cuz new_entries.log
```
