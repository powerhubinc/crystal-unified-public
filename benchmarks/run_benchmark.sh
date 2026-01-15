#!/bin/bash
# Crystal Unified Comprehensive Benchmark Script
# Usage: ./run_benchmark.sh [-q|--quick] [-n|--no-search] [-c|--corpus <name>]
#
# Options:
#   -q, --quick      Skip large files (>100MB)
#   -n, --no-search  Skip search benchmarks
#   -c, --corpus     Only benchmark specific corpus (silesia, canterbury, enwik, loghub)

set -e

# Parse arguments
QUICK=false
NO_SEARCH=false
CORPUS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -q|--quick) QUICK=true; shift ;;
        -n|--no-search) NO_SEARCH=true; shift ;;
        -c|--corpus) CORPUS="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Configuration - uses paths relative to script location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CUZ="$PROJECT_ROOT/target/release/cuz"
DATA_DIR="${CUZ_BENCHMARK_DATA:-$PROJECT_ROOT/data}"
OUTPUT_DIR="$SCRIPT_DIR/output"
REPORT_FILE="$PROJECT_ROOT/docs/BENCHMARK_RESULTS.md"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Helper functions
format_size() {
    local bytes=$1
    if (( bytes >= 1073741824 )); then
        printf "%.2f GB" "$(echo "scale=2; $bytes / 1073741824" | bc)"
    elif (( bytes >= 1048576 )); then
        printf "%.2f MB" "$(echo "scale=2; $bytes / 1048576" | bc)"
    elif (( bytes >= 1024 )); then
        printf "%.2f KB" "$(echo "scale=2; $bytes / 1024" | bc)"
    else
        printf "%d B" "$bytes"
    fi
}

format_speed() {
    local bps=$1
    if (( $(echo "$bps >= 1073741824" | bc -l) )); then
        printf "%.0f GB/s" "$(echo "scale=0; $bps / 1073741824" | bc)"
    elif (( $(echo "$bps >= 1048576" | bc -l) )); then
        printf "%.0f MB/s" "$(echo "scale=0; $bps / 1048576" | bc)"
    elif (( $(echo "$bps >= 1024" | bc -l) )); then
        printf "%.0f KB/s" "$(echo "scale=0; $bps / 1024" | bc)"
    else
        printf "%.0f B/s" "$bps"
    fi
}

# Results arrays
declare -a RESULTS
declare -a SEARCH_RESULTS

# Benchmark function
run_benchmark() {
    local input_file="$1"
    local mode="$2"
    local extra_args="$3"

    local output_file="$OUTPUT_DIR/$(basename "$input_file").cuz"
    local original_size=$(stat -f%z "$input_file" 2>/dev/null || stat -c%s "$input_file" 2>/dev/null)

    # Compression
    local start_time=$(python3 -c "import time; print(time.time())")
    if ! "$CUZ" $mode "$input_file" -o "$output_file" $extra_args 2>/dev/null; then
        echo "FAILED"
        return 1
    fi
    local end_time=$(python3 -c "import time; print(time.time())")
    local compress_time=$(echo "$end_time - $start_time" | bc)

    if [[ ! -f "$output_file" ]]; then
        echo "FAILED"
        return 1
    fi

    local compressed_size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null)
    local ratio=$(echo "scale=4; $compressed_size / $original_size" | bc)
    local speed=$(echo "scale=2; $original_size / $compress_time" | bc)

    # Decompression
    local decomp_file="${output_file}.dec"
    start_time=$(python3 -c "import time; print(time.time())")
    "$CUZ" decompress "$output_file" -o "$decomp_file" 2>/dev/null
    end_time=$(python3 -c "import time; print(time.time())")
    local decomp_time=$(echo "$end_time - $start_time" | bc)
    local decomp_speed=$(echo "scale=2; $original_size / $decomp_time" | bc)

    # Verify
    local verified="NO"
    if [[ -f "$decomp_file" ]]; then
        local orig_hash=$(md5sum "$input_file" 2>/dev/null | cut -d' ' -f1 || md5 -q "$input_file" 2>/dev/null)
        local decomp_hash=$(md5sum "$decomp_file" 2>/dev/null | cut -d' ' -f1 || md5 -q "$decomp_file" 2>/dev/null)
        if [[ "$orig_hash" == "$decomp_hash" ]]; then
            verified="Yes"
        fi
        rm -f "$decomp_file"
    fi

    echo "$original_size|$compressed_size|$ratio|$compress_time|$decomp_time|$speed|$decomp_speed|$verified"
}

# Search benchmark function
run_search_benchmark() {
    local compressed_file="$1"
    local original_file="$2"
    local pattern="$3"

    # Search compressed
    local start_time=$(python3 -c "import time; print(time.time() * 1000)")
    local cuz_count=$("$CUZ" search "$compressed_file" "$pattern" --count 2>/dev/null | grep -oE '[0-9]+' | head -1 || echo "0")
    local end_time=$(python3 -c "import time; print(time.time() * 1000)")
    local cuz_time=$(echo "$end_time - $start_time" | bc)

    # Search original with grep -c (count lines)
    start_time=$(python3 -c "import time; print(time.time() * 1000)")
    local grep_count=$(grep -c "$pattern" "$original_file" 2>/dev/null || echo "0")
    end_time=$(python3 -c "import time; print(time.time() * 1000)")
    local grep_time=$(echo "$end_time - $start_time" | bc)

    local accurate="NO"
    if [[ "$cuz_count" == "$grep_count" ]]; then
        accurate="Yes"
    fi

    local speedup="0"
    if (( $(echo "$cuz_time > 0" | bc -l) )); then
        speedup=$(echo "scale=1; $grep_time / $cuz_time" | bc)
    fi

    echo "$pattern|$cuz_count|$cuz_time|$grep_count|$grep_time|$speedup|$accurate"
}

# Header
echo ""
echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}  Crystal Unified Comprehensive Benchmark${NC}"
echo -e "${CYAN}================================================================${NC}"
echo ""
echo "Data directory: $DATA_DIR"
echo "Output directory: $OUTPUT_DIR"
echo "Report file: $REPORT_FILE"
if $QUICK; then echo -e "${YELLOW}Mode: Quick (skipping large files)${NC}"; fi
if $NO_SEARCH; then echo -e "${YELLOW}Mode: No search benchmarks${NC}"; fi
if [[ -n "$CORPUS" ]]; then echo -e "${YELLOW}Corpus filter: $CORPUS${NC}"; fi
echo ""

# Check if CUZ exists
if [[ ! -x "$CUZ" ]]; then
    echo -e "${RED}Error: CUZ binary not found at $CUZ${NC}"
    echo "Run 'cargo build --release' first"
    exit 1
fi

# Define corpora
declare -A LOGHUB_FILES=(
    ["openstack_abnormal.log"]="Log"
    ["openstack_normal1.log"]="Log"
    ["Mac.log"]="Log"
    ["HealthApp.log"]="Log"
    ["HPC.log"]="Log"
    ["SSH.log"]="Log"
    ["Android.log"]="Log"
    ["BGL.log"]="Log"
    ["HDFS.log"]="Log"
)

declare -A SILESIA_FILES=(
    ["silesia/dickens"]="Text"
    ["silesia/mozilla"]="Binary"
    ["silesia/mr"]="Image"
    ["silesia/nci"]="Chemistry"
    ["silesia/ooffice"]="Binary"
    ["silesia/osdb"]="Database"
    ["silesia/reymont"]="Text"
    ["silesia/samba"]="Source"
    ["silesia/sao"]="Binary"
    ["silesia/webster"]="Text"
    ["silesia/xml"]="XML"
    ["silesia/x-ray"]="Image"
)

declare -A CANTERBURY_FILES=(
    ["canterbury/alice29.txt"]="Text"
    ["canterbury/asyoulik.txt"]="Text"
    ["canterbury/cp.html"]="HTML"
    ["canterbury/fields.c"]="Source"
    ["canterbury/grammar.lsp"]="Source"
    ["canterbury/kennedy.xls"]="Binary"
    ["canterbury/lcet10.txt"]="Text"
    ["canterbury/plrabn12.txt"]="Text"
    ["canterbury/ptt5"]="Image"
    ["canterbury/sum"]="Binary"
    ["canterbury/xargs.1"]="Text"
)

declare -A ENWIK_FILES=(
    ["enwik/enwik8"]="Text"
)

# Search patterns by category
get_search_patterns() {
    local category="$1"
    case "$category" in
        Log) echo "error ERROR warning failed exception" ;;
        Text) echo "the and that which from" ;;
        Source) echo "int return if function void" ;;
        XML) echo "xml version encoding" ;;
        HTML) echo "class href" ;;
        *) echo "" ;;
    esac
}

# Initialize report
TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")
CPU_INFO=$(uname -p 2>/dev/null || cat /proc/cpuinfo 2>/dev/null | grep "model name" | head -1 | cut -d: -f2 | xargs || echo "Unknown")
CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "?")
RAM=$(free -h 2>/dev/null | grep Mem | awk '{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{printf "%.0f GB", $1/1073741824}' || echo "Unknown")

REPORT="# Crystal Unified Benchmark Results

**Generated:** $TIMESTAMP
**System:** $(hostname)
**CPU:** $CPU_INFO ($CORES cores)
**RAM:** $RAM
**CUZ Version:** v1.0

## Compression Results

| File | Corpus | Original | Compressed | Ratio | Compress Speed | Decompress Speed | Verified |
|------|--------|----------|------------|-------|----------------|------------------|----------|"

# Benchmark each corpus
benchmark_corpus() {
    local corpus_name="$1"
    local -n files_ref=$2

    echo ""
    echo -e "${MAGENTA}=== ${corpus_name^^} CORPUS ===${NC}"

    for file_path in "${!files_ref[@]}"; do
        local category="${files_ref[$file_path]}"
        local full_path="$DATA_DIR/$file_path"

        if [[ ! -f "$full_path" ]]; then
            echo -e "  ${YELLOW}[SKIP] $file_path - not found${NC}"
            continue
        fi

        local file_size=$(stat -f%z "$full_path" 2>/dev/null || stat -c%s "$full_path" 2>/dev/null)

        # Skip large files in quick mode
        if $QUICK && (( file_size > 104857600 )); then
            echo -e "  ${YELLOW}[SKIP] $file_path - too large for quick mode${NC}"
            continue
        fi

        echo -e "${GREEN}Testing: $file_path ($(format_size $file_size)) [$category]${NC}"

        # Run single-thread benchmark
        echo -n "  Single-thread... "
        local result=$(run_benchmark "$full_path" "compress" "")
        if [[ "$result" != "FAILED" ]]; then
            IFS='|' read -r orig comp ratio ctime dtime cspeed dspeed verified <<< "$result"
            local ratio_pct=$(echo "scale=1; $ratio * 100" | bc)
            echo -e "$(format_speed $cspeed) | ${ratio_pct}% | $verified"

            REPORT="$REPORT
| $(basename "$file_path") | $corpus_name | $(format_size $orig) | $(format_size $comp) | ${ratio_pct}% | $(format_speed $cspeed) | $(format_speed $dspeed) | $verified |"
        fi

        # Search benchmarks
        if ! $NO_SEARCH && [[ -n "$(get_search_patterns "$category")" ]]; then
            local compressed_file="$OUTPUT_DIR/$(basename "$full_path").cuz"
            if [[ -f "$compressed_file" ]]; then
                echo -e "  ${CYAN}Search tests:${NC}"
                local count=0
                for pattern in $(get_search_patterns "$category"); do
                    if (( count >= 3 )); then break; fi
                    local search_result=$(run_search_benchmark "$compressed_file" "$full_path" "$pattern")
                    IFS='|' read -r pat cuz_cnt cuz_t grep_cnt grep_t speedup acc <<< "$search_result"
                    local mark="="
                    local color="$NC"
                    if [[ "$acc" != "Yes" ]]; then
                        mark="!="
                        color="$YELLOW"
                    fi
                    echo -e "    ${color}'$pat': cuz=$cuz_cnt $mark grep=$grep_cnt | ${cuz_t}ms vs ${grep_t}ms${NC}"
                    ((count++))
                done
            fi
        fi
    done
}

# Run benchmarks based on corpus filter
if [[ -z "$CORPUS" ]] || [[ "$CORPUS" == "loghub" ]]; then
    benchmark_corpus "loghub" LOGHUB_FILES
fi
if [[ -z "$CORPUS" ]] || [[ "$CORPUS" == "silesia" ]]; then
    benchmark_corpus "silesia" SILESIA_FILES
fi
if [[ -z "$CORPUS" ]] || [[ "$CORPUS" == "canterbury" ]]; then
    benchmark_corpus "canterbury" CANTERBURY_FILES
fi
if [[ -z "$CORPUS" ]] || [[ "$CORPUS" == "enwik" ]]; then
    benchmark_corpus "enwik" ENWIK_FILES
fi

# Finalize report
REPORT="$REPORT

---

*Generated by Crystal Unified Benchmark Script*"

# Save report
echo "$REPORT" > "$REPORT_FILE"

echo ""
echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}  Benchmark Complete!${NC}"
echo -e "${CYAN}================================================================${NC}"
echo ""
echo -e "Report saved to: ${GREEN}$REPORT_FILE${NC}"
echo ""

# Cleanup
echo "Cleaning up temporary files..."
rm -f "$OUTPUT_DIR"/*.cuz "$OUTPUT_DIR"/*.dec 2>/dev/null

echo -e "${GREEN}Done!${NC}"
