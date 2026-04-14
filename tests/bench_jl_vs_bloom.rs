use crystal_unified::{
    compress_parallel_with_options, CompressionOptions, CrystalReaderV10,
    embed_archive, jl_sketch_from_embedding,
};
use std::time::Instant;

// Uniform mix -- every block has equal distribution of all categories
fn make_log_corpus_uniform(num_lines: usize) -> Vec<u8> {
    let categories = [
        "ERROR connection timeout to server-{} port 5432 retry attempt {}",
        "INFO  database query executed in {}ms rows_affected={} table=users",
        "WARN  authentication failed for user_{} from 192.168.1.{} invalid_password",
        "DEBUG request processed path=/api/v2/resource/{} status=200 latency={}ms",
        "ERROR disk I/O error on volume-{} sector {} read_timeout",
        "INFO  cache hit ratio={}% evictions={} memory_used={}MB",
        "WARN  rate limit exceeded for client_{} endpoint=/search quota={}",
        "ERROR out of memory on worker-{} heap_used={}GB max=4GB",
    ];

    let mut lines = Vec::with_capacity(num_lines);
    for i in 0..num_lines {
        let cat = &categories[i % categories.len()];
        let line = format!(
            "2026-04-14T{:02}:{:02}:{:02}.{:03}Z {}",
            (i / 3600) % 24, (i / 60) % 60, i % 60, (i * 7) % 1000,
            cat.replacen("{}", &(i % 100).to_string(), 1)
               .replacen("{}", &(i % 50).to_string(), 1)
        );
        lines.push(line);
    }
    lines.join("\n").into_bytes()
}

fn try_load_real_log() -> Option<Vec<u8>> {
    let paths = [
        "J:/powerhubinc/data/SSH.log",
        "J:/powerhubinc/data/Mac.log",
        "J:/powerhubinc/data/HPC.log",
        "J:/powerhubinc/data/HealthApp.log",
    ];
    for p in &paths {
        if let Ok(data) = std::fs::read(p) {
            if data.len() > 1024 {
                eprintln!("  Loaded real log: {} ({:.2} MB)", p, data.len() as f64 / 1048576.0);
                return Some(data);
            }
        }
    }
    None
}

struct BenchResult {
    mode: String,
    compress_ms: f64,
    compressed_size: usize,
    index_overhead: usize,
    decompress_ms: f64,
    exact_search_ms: f64,
    exact_search_hits: usize,
    blocks: u64,
    verified: bool,
}

fn run_mode(data: &[u8], mode: &str, opts: CompressionOptions) -> BenchResult {
    let t0 = Instant::now();
    let compressed = compress_parallel_with_options(data, opts);
    let compress_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let blocks = reader.block_count();

    let t0 = Instant::now();
    let reconstructed = reader.reconstruct();
    let decompress_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let verified = reconstructed == data;

    let term = b"timeout";
    let t0 = Instant::now();
    let hits = reader.count_matches(term);
    let exact_search_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let bloom_size = blocks as usize * 8192;
    let trigram_size = if reader.has_trigrams() { blocks as usize * 8192 } else { 0 };
    let index_overhead = bloom_size + trigram_size;

    BenchResult {
        mode: mode.to_string(),
        compress_ms,
        compressed_size: compressed.len(),
        index_overhead,
        decompress_ms,
        exact_search_ms,
        exact_search_hits: hits,
        blocks,
        verified,
    }
}

fn fmt_size(bytes: usize) -> String {
    if bytes >= 1048576 {
        format!("{:.2} MB", bytes as f64 / 1048576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn print_results(name: &str, data_size: usize, results: &[BenchResult]) {
    let sep = "=".repeat(100);
    eprintln!("\n{}", sep);
    eprintln!("  {} | Input: {}", name, fmt_size(data_size));
    eprintln!("{}", sep);
    eprintln!(
        "  {:20} {:>10} {:>10} {:>8} {:>10} {:>10} {:>10}",
        "Mode", "Compressed", "Index OH", "Blocks", "Comp ms", "Decomp ms", "Search ms"
    );
    eprintln!("  {:-<90}", "");

    for r in results {
        let ratio = r.compressed_size as f64 / data_size as f64 * 100.0;
        let verify = if r.verified { "OK" } else { "FAIL" };

        eprintln!(
            "  {:20} {:>7} ({:>4.1}%) {:>10} {:>8} {:>8.1}ms {:>8.1}ms {:>8.2}ms [{}]",
            r.mode,
            fmt_size(r.compressed_size),
            ratio,
            fmt_size(r.index_overhead),
            r.blocks,
            r.compress_ms,
            r.decompress_ms,
            r.exact_search_ms,
            verify,
        );
    }

    eprintln!("\n  Search results for 'timeout':");
    for r in results {
        eprintln!("    {:20} Exact: {} hits in {:.2}ms",
            r.mode, r.exact_search_hits, r.exact_search_ms);
    }
}

#[test]
fn bench_synthetic_uniform() {
    let data = make_log_corpus_uniform(50_000);

    let modes: Vec<(&str, CompressionOptions)> = vec![
        ("baseline", CompressionOptions::default().with_block_size(16384)),
        ("+ trigrams", CompressionOptions::default().with_block_size(16384).with_trigrams(true)),
    ];

    let results: Vec<BenchResult> = modes.iter()
        .map(|(name, opts)| run_mode(&data, name, opts.clone()))
        .collect();

    print_results("Synthetic 50K Uniform", data.len(), &results);
}

#[test]
fn bench_real_logs() {
    if let Some(data) = try_load_real_log() {
        let data = if data.len() > 5 * 1048576 {
            let cut = data[..5 * 1048576].iter().rposition(|&b| b == b'\n').unwrap_or(5 * 1048576);
            data[..cut].to_vec()
        } else {
            data
        };

        let modes: Vec<(&str, CompressionOptions)> = vec![
            ("baseline", CompressionOptions::default().with_block_size(16384)),
            ("+ trigrams", CompressionOptions::default().with_block_size(16384).with_trigrams(true)),
        ];

        let results: Vec<BenchResult> = modes.iter()
            .map(|(name, opts)| run_mode(&data, name, opts.clone()))
            .collect();

        print_results("Real Log File", data.len(), &results);
    } else {
        eprintln!("  [SKIP] No real log files found at J:/powerhubinc/data/");
    }
}

#[test]
fn bench_embed_and_search() {
    let data = make_log_corpus_uniform(50_000);
    let opts = CompressionOptions::default().with_block_size(16384);
    let compressed = compress_parallel_with_options(&data, opts);

    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    // Create distinct embeddings per block
    let embeddings: Vec<Vec<f32>> = (0..block_count)
        .map(|i| (0..64).map(|j| ((i * 64 + j) as f32) * 0.01 + (i as f32)).collect())
        .collect();

    let t0 = Instant::now();
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();
    let embed_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let reader2 = CrystalReaderV10::new(&with_vectors).unwrap();

    let query = &embeddings[0];
    let t0 = Instant::now();
    let results = reader2.search_similar_top_k(query, 3);
    let search_ms = t0.elapsed().as_secs_f64() * 1000.0;

    eprintln!("\n  embed_archive: {} blocks, {:.1}ms", block_count, embed_ms);
    eprintln!("  Overhead: {} bytes ({} per block)",
        with_vectors.len() - compressed.len(),
        (with_vectors.len() - compressed.len()) / block_count);
    eprintln!("  search_similar_top_k(3): {:.2}ms, top block={}, score={:.4}",
        search_ms,
        results.first().map(|m| m.block_idx).unwrap_or(0),
        results.first().map(|m| m.score).unwrap_or(0.0));
}
