use crystal_unified::{
    compress_parallel_with_options, CompressionOptions, CrystalReaderV10,
};
use std::time::Instant;

fn try_load(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok().filter(|d| d.len() > 1024)
}

fn fmt_size(bytes: usize) -> String {
    if bytes >= 1048576 { format!("{:.2} MB", bytes as f64 / 1048576.0) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}

struct SearchComparison {
    query: String,
    bloom_candidates: usize,
    trigram_candidates: usize,
    exact_hits: usize,
    bloom_false_pos: usize,
    trigram_false_pos: usize,
}

fn run_search_comparison(
    reader_baseline: &CrystalReaderV10,
    reader_trigram: &CrystalReaderV10,
    query: &str,
) -> SearchComparison {
    let term = query.as_bytes();

    let bloom_cands = reader_baseline.filter_candidates(term);
    let bloom_candidates = bloom_cands.len();

    let trigram_cands = reader_trigram.filter_candidates_trigram(term);
    let trigram_candidates = trigram_cands.len();

    let exact_hits = reader_baseline.count_matches(term);

    let mut bloom_false_pos = 0;
    for &block_idx in &bloom_cands {
        if let Ok(rows) = reader_baseline.decompress_block_bytes(block_idx) {
            let block_text = String::from_utf8_lossy(&rows);
            if !block_text.contains(query) {
                bloom_false_pos += 1;
            }
        }
    }

    let mut trigram_false_pos = 0;
    for &block_idx in &trigram_cands {
        if let Ok(rows) = reader_trigram.decompress_block_bytes(block_idx) {
            let block_text = String::from_utf8_lossy(&rows);
            if !block_text.contains(query) {
                trigram_false_pos += 1;
            }
        }
    }

    SearchComparison {
        query: query.to_string(),
        bloom_candidates,
        trigram_candidates,
        exact_hits,
        bloom_false_pos,
        trigram_false_pos,
    }
}

fn bench_file(path: &str, name: &str, queries: &[&str]) {
    let data = match try_load(path) {
        Some(d) => d,
        None => {
            eprintln!("  [SKIP] {} not found", path);
            return;
        }
    };

    eprintln!("\n  {} ({})", name, fmt_size(data.len()));

    let t0 = Instant::now();
    let opts_baseline = CompressionOptions::default();
    let compressed_baseline = compress_parallel_with_options(&data, opts_baseline);
    let baseline_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let t0 = Instant::now();
    let opts_trigram = CompressionOptions::default().with_trigrams(true);
    let compressed_trigram = compress_parallel_with_options(&data, opts_trigram);
    let trigram_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let reader_baseline = CrystalReaderV10::new(&compressed_baseline).unwrap();
    let reader_trigram = CrystalReaderV10::new(&compressed_trigram).unwrap();

    let blocks = reader_baseline.block_count();
    eprintln!("  Blocks: {} | Compressed: baseline={} trigram={}",
        blocks,
        fmt_size(compressed_baseline.len()),
        fmt_size(compressed_trigram.len()),
    );
    eprintln!("  Compress time: baseline={:.0}ms trigram={:.0}ms", baseline_ms, trigram_ms);

    for &q in queries {
        let hits_b = reader_baseline.count_matches(q.as_bytes());
        let hits_t = reader_trigram.count_matches(q.as_bytes());
        assert_eq!(hits_b, hits_t, "trigram mode exact hits mismatch for '{}'", q);
    }
    eprintln!("  [PASS] Exact search consistency: all modes return identical hit counts");

    eprintln!();
    eprintln!("  {:25} {:>7} {:>7} {:>8} {:>10} {:>10}",
        "Query", "Exact", "Bloom", "Bloom FP", "Trigram", "Tri FP");
    eprintln!("  {:-<80}", "");

    for &q in queries {
        let c = run_search_comparison(&reader_baseline, &reader_trigram, q);
        let bloom_fp_pct = if c.bloom_candidates > 0 {
            c.bloom_false_pos as f64 / c.bloom_candidates as f64 * 100.0
        } else { 0.0 };
        let tri_fp_pct = if c.trigram_candidates > 0 {
            c.trigram_false_pos as f64 / c.trigram_candidates as f64 * 100.0
        } else { 0.0 };

        eprintln!("  {:25} {:>7} {:>4}/{:>2} {:>5.0}%FP {:>5}/{:>3} {:>5.0}%FP",
            c.query,
            c.exact_hits,
            c.bloom_candidates, blocks,
            bloom_fp_pct,
            c.trigram_candidates, blocks,
            tri_fp_pct,
        );
    }
}

#[test]
fn bench_search_comparison_full() {
    let sep = "=".repeat(100);
    eprintln!("\n{}", sep);
    eprintln!("  SEARCH COMPARISON: Bloom vs Trigram -- False Positive Analysis");
    eprintln!("{}", sep);

    bench_file(
        "J:/powerhubinc/data/BGL.log",
        "BGL.log",
        &["error", "FATAL", "kernel", "timeout", "connection", "memory", "NFS"],
    );

    bench_file(
        "J:/powerhubinc/data/HDFS.log",
        "HDFS.log",
        &["error", "WARN", "blk_", "Received", "timeout", "Exception", "Deleting"],
    );

    bench_file(
        "J:/powerhubinc/data/SSH.log",
        "SSH.log",
        &["error", "Failed", "password", "Accepted", "invalid", "session", "timeout"],
    );

    bench_file(
        "J:/powerhubinc/data/Android.log",
        "Android.log",
        &["error", "Exception", "ActivityManager", "timeout", "PowerManager", "wifi"],
    );
}
