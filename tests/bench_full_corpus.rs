use crystal_unified::{
    compress_parallel_with_options, CompressionOptions, CrystalReaderV10,
};
use std::time::Instant;

fn fmt_size(bytes: usize) -> String {
    if bytes >= 1048576 {
        format!("{:.2} MB", bytes as f64 / 1048576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn bench_file(path: &str, name: &str) {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("  [SKIP] {} not found", path);
            return;
        }
    };
    let original_size = data.len();
    eprintln!("\n  {} ({}):", name, fmt_size(original_size));

    let configs: Vec<(&str, CompressionOptions)> = vec![
        ("baseline", CompressionOptions::default()),
        ("+ trigrams", CompressionOptions::default().with_trigrams(true)),
    ];

    for (label, opts) in &configs {
        let t0 = Instant::now();
        let compressed = compress_parallel_with_options(&data, opts.clone());
        let compress_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let speed_mbs = original_size as f64 / 1048576.0 / (compress_ms / 1000.0);

        let reader = CrystalReaderV10::new(&compressed).unwrap();
        let blocks = reader.block_count();

        // Verify roundtrip
        let t0 = Instant::now();
        let reconstructed = reader.reconstruct();
        let _decompress_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let verified = reconstructed == data;

        // Exact search
        let t0 = Instant::now();
        let hits = reader.count_matches(b"error");
        let search_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let ratio = compressed.len() as f64 / original_size as f64 * 100.0;
        let bloom_oh = blocks as usize * 8192;

        eprintln!(
            "    {:22} {:>8} {:>5.1}% {:>5} blks | {:>7.0} MB/s | search: {:>6.1}ms ({} hits) | Bloom: {} [{}]",
            label,
            fmt_size(compressed.len()),
            ratio,
            blocks,
            speed_mbs,
            search_ms,
            hits,
            fmt_size(bloom_oh),
            if verified { "OK" } else { "FAIL" },
        );
    }
}

#[test]
fn bench_full_loghub_corpus() {
    let sep = "=".repeat(120);
    eprintln!("\n{}", sep);
    eprintln!("  FULL LOGHUB CORPUS BENCHMARK -- Compression + Search");
    eprintln!("{}", sep);

    bench_file("J:/powerhubinc/data/SSH.log", "SSH.log");
    bench_file("J:/powerhubinc/data/Mac.log", "Mac.log");
    bench_file("J:/powerhubinc/data/HPC.log", "HPC.log");
    bench_file("J:/powerhubinc/data/HealthApp.log", "HealthApp.log");
    bench_file("J:/powerhubinc/data/Android.log", "Android.log");
    bench_file("J:/powerhubinc/data/BGL.log", "BGL.log");
    bench_file("J:/powerhubinc/data/HDFS.log", "HDFS.log");
}
