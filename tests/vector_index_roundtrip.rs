use crystal_unified::{
    compress_parallel_with_options, CompressionOptions, CrystalReaderV10,
    embed_archive, jl_sketch_from_embedding,
};

fn make_log_data() -> Vec<u8> {
    let mut lines = Vec::new();
    // Block 1: Error/network-related lines
    for i in 0..200 {
        lines.push(format!(
            "2026-04-14T10:{:02}:{:02} ERROR connection timeout to server-{} port 5432 retry attempt {}",
            i / 60, i % 60, i % 5, i % 3
        ));
    }
    // Block 2: Database-related lines
    for i in 0..200 {
        lines.push(format!(
            "2026-04-14T11:{:02}:{:02} INFO database query executed in {}ms rows_affected={} table=users",
            i / 60, i % 60, 10 + i * 2, i * 5
        ));
    }
    // Block 3: Authentication-related lines
    for i in 0..200 {
        lines.push(format!(
            "2026-04-14T12:{:02}:{:02} WARN authentication failed for user_{} from 192.168.1.{} invalid_password",
            i / 60, i % 60, i % 20, i % 255
        ));
    }
    lines.join("\n").into_bytes()
}

/// Compress data, then embed vectors for each block.
fn compress_and_embed(data: &[u8], embeddings: &[Vec<f32>]) -> Vec<u8> {
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(data, options);
    embed_archive(&compressed, embeddings, 64, 4).expect("embed_archive should succeed")
}

fn make_test_embeddings(block_count: usize) -> Vec<Vec<f32>> {
    // Create distinct embeddings for each block
    (0..block_count)
        .map(|i| {
            (0..64)
                .map(|j| ((i * 64 + j) as f32) * 0.01 + (i as f32))
                .collect()
        })
        .collect()
}

#[test]
fn test_embed_archive_roundtrip() {
    let data = make_log_data();

    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);
    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    assert!(!reader.has_jl_sketch(), "plain archive should not have JL sketch");

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();

    let reader2 = CrystalReaderV10::new(&with_vectors).unwrap();
    assert!(reader2.has_jl_sketch(), "embedded archive should have JL sketch flag");
    assert_eq!(reader2.block_count(), block_count as u64);

    // Verify data roundtrips correctly
    let reconstructed = reader2.reconstruct();
    assert_eq!(data, reconstructed, "data should roundtrip exactly");
}

#[test]
fn test_embed_archive_rejects_double_embed() {
    let data = make_log_data();
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);
    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();

    // Second embed should fail
    let result = embed_archive(&with_vectors, &embeddings, 64, 4);
    assert!(result.is_err(), "double embed should fail");
}

#[test]
fn test_embed_archive_wrong_count() {
    let data = make_log_data();
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);

    // Wrong number of embeddings
    let embeddings = vec![vec![0.0f32; 64]]; // only 1
    let result = embed_archive(&compressed, &embeddings, 64, 4);
    assert!(result.is_err(), "wrong embedding count should fail");
}

#[test]
fn test_similarity_search_with_embeddings() {
    let data = make_log_data();
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);
    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();

    let reader2 = CrystalReaderV10::new(&with_vectors).unwrap();

    // Search with embedding similar to block 0
    let query = &embeddings[0];
    let results = reader2.search_similar(query, 0.0);
    assert!(!results.is_empty(), "should find at least one similar block");

    // The top result should be block 0
    let top = &results[0];
    assert_eq!(top.block_idx, 0, "top match should be block 0");
    assert!(top.score > 0.5, "self-similarity score should be high");
}

#[test]
fn test_similarity_search_top_k() {
    let data = make_log_data();
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);
    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();

    let reader2 = CrystalReaderV10::new(&with_vectors).unwrap();

    let top2 = reader2.search_similar_top_k(&embeddings[1], 2);
    assert_eq!(top2.len(), 2, "should return exactly 2 results");

    // Top result should be block 1 (most similar to its own embedding)
    assert_eq!(top2[0].block_idx, 1, "top match should be block 1");
}

#[test]
fn test_similarity_with_results_decompresses() {
    let data = make_log_data();
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);
    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();

    let reader2 = CrystalReaderV10::new(&with_vectors).unwrap();

    let results = reader2.search_similar_with_results(&embeddings[2], 0.0);
    assert!(!results.is_empty(), "should find results");

    let (score, lines) = &results[0];
    assert!(*score > 0.0);
    assert!(!lines.is_empty(), "should return actual lines");
}

#[test]
fn test_get_block_sketch() {
    let data = make_log_data();
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);
    let reader = CrystalReaderV10::new(&compressed).unwrap();
    let block_count = reader.block_count() as usize;

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed, &embeddings, 64, 4).unwrap();

    let reader2 = CrystalReaderV10::new(&with_vectors).unwrap();

    // Should be able to read each block's sketch
    for i in 0..reader2.block_count() {
        let sketch = reader2.get_block_sketch(i);
        assert!(sketch.is_some(), "block {} should have a sketch", i);
        let qv = sketch.unwrap();
        assert_eq!(qv.bits, 4);
        assert_eq!(qv.data.len(), 32); // 64 dims * 4 bits / 8
    }

    // Out of bounds should return None
    assert!(reader2.get_block_sketch(reader2.block_count()).is_none());
}

#[test]
fn test_no_jl_sketch_backward_compat() {
    let data = make_log_data();

    // Compress without embedding
    let options = CompressionOptions::default().with_block_size(200);
    let compressed = compress_parallel_with_options(&data, options);

    let reader = CrystalReaderV10::new(&compressed).unwrap();
    assert!(!reader.has_jl_sketch(), "old archive should not have JL sketch");

    // Similarity search should return empty on archives without sketches
    let query = vec![1.0f32; 64];
    let results = reader.search_similar(&query, 0.0);
    assert!(results.is_empty(), "search_similar should return empty for non-sketch archives");

    // Normal search should still work
    let count = reader.count_matches(b"ERROR");
    assert!(count > 0, "normal search should still work");
}

#[test]
fn test_jl_sketch_size_overhead() {
    let data = make_log_data();

    // Compress without embedding
    let options_no_jl = CompressionOptions::default().with_block_size(200);
    let compressed_no_jl = compress_parallel_with_options(&data, options_no_jl);

    let reader = CrystalReaderV10::new(&compressed_no_jl).unwrap();
    let block_count = reader.block_count() as usize;

    let embeddings = make_test_embeddings(block_count);
    let with_vectors = embed_archive(&compressed_no_jl, &embeddings, 64, 4).unwrap();

    // JL sketch overhead = block_count * 36 bytes (4 header + 32 data)
    let sketch_overhead = block_count * 36;
    let actual_overhead = with_vectors.len() - compressed_no_jl.len();

    assert_eq!(
        actual_overhead, sketch_overhead,
        "overhead ({}) should equal sketch entries ({} blocks * 36 = {})",
        actual_overhead, block_count, sketch_overhead
    );
}
