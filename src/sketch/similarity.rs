//! Similarity search index over quantized JL sketches
//!
//! Enables ranked similarity search on Crystal compressed archives:
//! instead of Bloom filter's boolean "block contains exact word X",
//! similarity search answers "which blocks are most similar to query Q"
//! with a continuous score, enabling fuzzy and semantic matching.
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use super::jl_sketch::jl_sketch_from_embedding;
use super::quantize::{QuantizedVector, VectorQuantizer, QUANT_BITS_DEFAULT};

#[derive(Clone, Debug)]
pub struct SimilarityMatch {
    pub block_idx: u64,
    pub score: f32,
}

pub struct SimilarityIndex {
    sketches: Vec<QuantizedVector>,
    quantizer: VectorQuantizer,
}

impl SimilarityIndex {
    pub fn new(bits: u8) -> Self {
        Self {
            sketches: Vec::new(),
            quantizer: VectorQuantizer::new(bits),
        }
    }

    pub fn add_sketch(&mut self, sketch: QuantizedVector) {
        self.sketches.push(sketch);
    }

    pub fn len(&self) -> usize {
        self.sketches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sketches.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<&QuantizedVector> {
        self.sketches.get(idx)
    }

    pub fn query(&self, query_sketch: &QuantizedVector, threshold: f32) -> Vec<SimilarityMatch> {
        let mut matches: Vec<SimilarityMatch> = self
            .sketches
            .iter()
            .enumerate()
            .filter_map(|(idx, block_sketch)| {
                let score = self
                    .quantizer
                    .quantized_cosine_similarity(query_sketch, block_sketch);
                if score >= threshold {
                    Some(SimilarityMatch {
                        block_idx: idx as u64,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    pub fn query_top_k(
        &self,
        query_sketch: &QuantizedVector,
        k: usize,
    ) -> Vec<SimilarityMatch> {
        let mut all: Vec<SimilarityMatch> = self
            .sketches
            .iter()
            .enumerate()
            .map(|(idx, block_sketch)| {
                let score = self
                    .quantizer
                    .quantized_cosine_similarity(query_sketch, block_sketch);
                SimilarityMatch {
                    block_idx: idx as u64,
                    score,
                }
            })
            .collect();

        all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(k);
        all
    }
}

pub fn build_similarity_index(sketches: Vec<QuantizedVector>) -> SimilarityIndex {
    let bits = sketches.first().map(|s| s.bits).unwrap_or(QUANT_BITS_DEFAULT);
    let mut index = SimilarityIndex::new(bits);
    for s in sketches {
        index.add_sketch(s);
    }
    index
}

pub fn query_similar_blocks(
    query_embedding: &[f32],
    index: &SimilarityIndex,
    dim: usize,
    threshold: f32,
) -> Vec<SimilarityMatch> {
    if query_embedding.is_empty() {
        return Vec::new();
    }

    let sketch = jl_sketch_from_embedding(query_embedding, dim);
    let bits = index
        .sketches
        .first()
        .map(|s| s.bits)
        .unwrap_or(QUANT_BITS_DEFAULT);
    let quantizer = VectorQuantizer::new(bits);
    let query_qv = quantizer.encode(&sketch);

    index.query(&query_qv, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketch::jl_sketch::JL_SKETCH_DIM;

    #[test]
    fn test_similarity_index_basic() {
        let quantizer = VectorQuantizer::new(4);

        // Use distinct embedding vectors to represent different "blocks"
        let emb1: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1).collect();
        let emb2: Vec<f32> = (0..64).map(|i| (i as f32) * -0.1).collect();
        let emb3: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1 + 0.01).collect();

        let s1 = quantizer.encode(&jl_sketch_from_embedding(&emb1, 64));
        let s2 = quantizer.encode(&jl_sketch_from_embedding(&emb2, 64));
        let s3 = quantizer.encode(&jl_sketch_from_embedding(&emb3, 64));

        let index = build_similarity_index(vec![s1, s2, s3]);
        assert_eq!(index.len(), 3);

        // Query similar to emb1/emb3
        let query_qv = quantizer.encode(&jl_sketch_from_embedding(&emb1, 64));

        let results = index.query_top_k(&query_qv, 2);
        assert_eq!(results.len(), 2);

        let top_idx = results[0].block_idx;
        assert!(
            top_idx == 0 || top_idx == 2,
            "top match should be similar embedding block, got {}",
            top_idx
        );
    }

    #[test]
    fn test_query_similar_blocks() {
        let quantizer = VectorQuantizer::new(4);

        let emb1: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1).collect();
        let emb2: Vec<f32> = (0..64).map(|i| (i as f32) * -0.2).collect();

        let s1 = quantizer.encode(&jl_sketch_from_embedding(&emb1, 64));
        let s2 = quantizer.encode(&jl_sketch_from_embedding(&emb2, 64));

        let index = build_similarity_index(vec![s1, s2]);

        let results = query_similar_blocks(&emb1, &index, 64, -1.0);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_empty_index() {
        let index = SimilarityIndex::new(4);
        let quantizer = VectorQuantizer::new(4);
        let sketch = vec![0.0f32; JL_SKETCH_DIM];
        let qv = quantizer.encode(&sketch);
        let results = index.query(&qv, 0.0);
        assert!(results.is_empty());
    }
}
