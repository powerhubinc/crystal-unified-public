use crystal_unified::{
    compress_parallel_with_options, CompressionOptions, CrystalReaderV10,
};
use std::time::Instant;

fn fmt_size(bytes: usize) -> String {
    if bytes >= 1048576 { format!("{:>8.2} MB", bytes as f64 / 1048576.0) }
    else if bytes >= 1024 { format!("{:>8.1} KB", bytes as f64 / 1024.0) }
    else { format!("{:>8} B ", bytes) }
}

fn bench_file(path: &str) {
    let name = std::path::Path::new(path).file_name().unwrap().to_str().unwrap();
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => { eprintln!("  [SKIP] {} -- {}", name, e); return; }
    };
    let original = data.len();

    let modes: Vec<(&str, CompressionOptions)> = vec![
        ("baseline",       CompressionOptions::default()),
        ("+ trigrams",     CompressionOptions::default().with_trigrams(true)),
    ];

    let mut first_hits: Option<usize> = None;

    for (label, opts) in &modes {
        let t0 = Instant::now();
        let compressed = compress_parallel_with_options(&data, opts.clone());
        let comp_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let speed = original as f64 / 1048576.0 / (comp_ms / 1000.0);

        let reader = match CrystalReaderV10::new(&compressed) {
            Ok(r) => r,
            Err(_) => { eprintln!("  {:28} PARSE FAILED", label); continue; }
        };
        let blocks = reader.block_count();

        let t0 = Instant::now();
        let reconstructed = reader.reconstruct();
        let decomp_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let verified = reconstructed == data;

        // Search
        let t0 = Instant::now();
        let hits = reader.count_matches(b"error");
        let search_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // Verify search consistency
        if let Some(expected) = first_hits {
            assert_eq!(hits, expected, "{}: search hits mismatch in mode {}", name, label);
        } else {
            first_hits = Some(hits);
        }

        let ratio = compressed.len() as f64 / original as f64 * 100.0;

        eprintln!(
            "  {:16} {:>5.1}% {} {:>4} blks {:>6.0} MB/s {:>7.1}ms dc {:>7.1}ms srch({:>6}) [{}]",
            label, ratio, fmt_size(compressed.len()), blocks, speed,
            decomp_ms, search_ms, hits,
            if verified { "OK" } else { "FAIL" }
        );
    }
}

#[test]
fn bench_diverse_corpus() {
    let dir = "J:/powerhubinc/data/bench_diverse";
    let sep = "=".repeat(120);

    eprintln!("\n{}", sep);
    eprintln!("  BENCH_DIVERSE FULL COMPARISON -- Compression modes on all file types");
    eprintln!("{}", sep);

    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|e| !e.file_name().to_str().unwrap_or("").ends_with(".md"))
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().map(|m| m.len()).unwrap_or(0)));

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        eprintln!("\n  {} ({})", name, fmt_size(size as usize));
        eprintln!("  {:-<100}", "");
        bench_file(path.to_str().unwrap());
    }

    eprintln!("\n{}", sep);
    eprintln!("  DONE");
    eprintln!("{}", sep);
}
