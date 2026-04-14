use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use crystal_unified::{
    compress_parallel_with_options, CompressionOptions, CrystalReaderV10,
    jl_sketch_from_embedding, quantize_vector, VectorQuantizer,
    embed_archive,
};

// ---------------------------------------------------------------------------
// Data generators
// ---------------------------------------------------------------------------

fn make_log_corpus(num_lines: usize) -> Vec<u8> {
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
            (i / 3600) % 24,
            (i / 60) % 60,
            i % 60,
            (i * 7) % 1000,
            cat.replacen("{}", &(i % 100).to_string(), 1)
               .replacen("{}", &(i % 50).to_string(), 1)
        );
        lines.push(line);
    }
    lines.join("\n").into_bytes()
}

fn make_embeddings(block_count: usize) -> Vec<Vec<f32>> {
    (0..block_count)
        .map(|i| (0..64).map(|j| ((i * 64 + j) as f32) * 0.01 + (i as f32)).collect())
        .collect()
}

// ---------------------------------------------------------------------------
// Compression benchmarks
// ---------------------------------------------------------------------------

fn bench_compression(c: &mut Criterion) {
    let data = make_log_corpus(50_000);

    let mut group = c.benchmark_group("compression");
    group.sample_size(10);

    // Baseline: default (fast_mode, no extras)
    group.bench_function("baseline", |b| {
        b.iter(|| {
            let opts = CompressionOptions::default().with_block_size(16384);
            black_box(compress_parallel_with_options(&data, opts))
        })
    });

    // With trigrams only
    group.bench_function("trigrams", |b| {
        b.iter(|| {
            let opts = CompressionOptions::default()
                .with_block_size(16384)
                .with_trigrams(true);
            black_box(compress_parallel_with_options(&data, opts))
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Search benchmarks
// ---------------------------------------------------------------------------

fn bench_search(c: &mut Criterion) {
    let data = make_log_corpus(50_000);

    // Pre-compress
    let opts_bloom = CompressionOptions::default().with_block_size(16384);
    let compressed_bloom = compress_parallel_with_options(&data, opts_bloom);

    let opts_trigram = CompressionOptions::default()
        .with_block_size(16384)
        .with_trigrams(true);
    let compressed_trigram = compress_parallel_with_options(&data, opts_trigram);

    // Create archive with embedded vectors
    let reader_tmp = CrystalReaderV10::new(&compressed_bloom).unwrap();
    let block_count = reader_tmp.block_count() as usize;
    let embeddings = make_embeddings(block_count);
    let compressed_embedded = embed_archive(&compressed_bloom, &embeddings, 64, 4).unwrap();

    let mut group = c.benchmark_group("exact_search");
    group.sample_size(20);

    let term = b"timeout";

    group.bench_function("bloom_only", |b| {
        b.iter(|| {
            let reader = CrystalReaderV10::new(&compressed_bloom).unwrap();
            black_box(reader.count_matches(term))
        })
    });

    group.bench_function("trigram", |b| {
        b.iter(|| {
            let reader = CrystalReaderV10::new(&compressed_trigram).unwrap();
            black_box(reader.count_matches(term))
        })
    });

    group.finish();

    // Similarity search (only available with embedded vectors)
    let mut sim_group = c.benchmark_group("similarity_search");
    sim_group.sample_size(20);

    let query_emb: Vec<f32> = (0..64).map(|i| (i as f32) * 0.05).collect();

    sim_group.bench_function("search_similar_threshold", |b| {
        b.iter(|| {
            let reader = CrystalReaderV10::new(&compressed_embedded).unwrap();
            black_box(reader.search_similar(&query_emb, 0.1))
        })
    });

    sim_group.bench_function("search_similar_top_5", |b| {
        b.iter(|| {
            let reader = CrystalReaderV10::new(&compressed_embedded).unwrap();
            black_box(reader.search_similar_top_k(&query_emb, 5))
        })
    });

    sim_group.finish();
}

// ---------------------------------------------------------------------------
// JL sketch + Vector Quantization micro-benchmarks
// ---------------------------------------------------------------------------

fn bench_sketch_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("sketch_ops");

    let embedding: Vec<f32> = (0..128).map(|i| (i as f32) * 0.1).collect();

    group.bench_function("jl_project_embedding_128d", |b| {
        b.iter(|| black_box(jl_sketch_from_embedding(&embedding, 64)))
    });

    let sketch = jl_sketch_from_embedding(&embedding, 64);

    group.bench_function("quantize_vector_4bit", |b| {
        b.iter(|| black_box(quantize_vector(&sketch, 4)))
    });

    group.bench_function("quantize_vector_2bit", |b| {
        b.iter(|| black_box(quantize_vector(&sketch, 2)))
    });

    let qv_a = quantize_vector(&sketch, 4);
    let embedding_b: Vec<f32> = (0..128).map(|i| (i as f32) * -0.1).collect();
    let sketch_b = jl_sketch_from_embedding(&embedding_b, 64);
    let qv_b = quantize_vector(&sketch_b, 4);
    let tq = VectorQuantizer::new(4);

    group.bench_function("quantized_cosine_similarity", |b| {
        b.iter(|| black_box(tq.quantized_cosine_similarity(&qv_a, &qv_b)))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// File size comparison
// ---------------------------------------------------------------------------

fn bench_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_size_comparison");
    group.sample_size(10);

    for &line_count in &[1_000u64, 10_000, 50_000] {
        let data = make_log_corpus(line_count as usize);

        group.bench_with_input(
            BenchmarkId::new("baseline", line_count),
            &data,
            |b, data| {
                b.iter(|| {
                    let opts = CompressionOptions::default().with_block_size(16384);
                    black_box(compress_parallel_with_options(data, opts))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_compression,
    bench_search,
    bench_sketch_ops,
    bench_file_sizes,
);
criterion_main!(benches);
