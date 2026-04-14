//! Vector index and quantization for similarity search
//!
//! This module enables similarity search on Crystal compressed archives
//! by combining two techniques:
//!
//! 1. **JL Projections** (Freksen 2021): Random Rademacher projections reduce
//!    user-provided embedding vectors to a compact sketch while preserving
//!    pairwise distances within (1 ± ε).
//!
//! 2. **vector quantization** (Zandieh et al. 2025): Data-oblivious quantization
//!    compresses each sketch to just 36 bytes (4-bit) or 20 bytes (2-bit)
//!    via Walsh-Hadamard rotation + uniform scalar quantization.
//!    Inner product estimation is near-optimal (within 2.7× of the
//!    information-theoretic lower bound).
//!
//! Users provide their own embeddings (e.g. from a language model) and
//! attach them to an archive via `embed_archive`. Similarity search then
//! operates on these quantized vectors without decompressing any block data.
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

pub mod jl_sketch;
pub mod quantize;
pub mod similarity;

pub use jl_sketch::{
    JLProjection, JLSketch,
    jl_sketch_from_embedding,
    jl_sketch_empty, jl_sketch_merge,
    jl_cosine_similarity,
    JL_SKETCH_DIM, JL_SKETCH_BYTES,
};
pub use quantize::{
    VectorQuantizer, QuantizedVector,
    quantize_vector, dequantize_vector,
    quantized_inner_product,
    quantized_vector_bytes, quantized_vector_bytes_for,
    QUANT_BITS_DEFAULT,
};
pub use similarity::{
    SimilarityIndex, SimilarityMatch,
    build_similarity_index, query_similar_blocks,
};
