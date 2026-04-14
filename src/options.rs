//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use crate::constants::*;

#[derive(Clone, Debug)]
pub struct CompressionOptions {
    pub block_size: u64,
    pub compression_level: i32,
    pub use_dictionary: bool,
    pub dictionary_size: usize,
    pub dictionary_sample_bytes: usize,
    pub use_trigrams: bool,
    pub raw_mode: bool,
    pub auto_transform: bool,
    pub transform_type: u8,
    pub auto_level: bool,
    pub fast_mode: bool,
    pub use_jl_sketch: bool,
    pub sketch_dim: usize,
    pub sketch_bits: u8,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            block_size: DEFAULT_BLOCK_SIZE,
            compression_level: DEFAULT_COMPRESSION_LEVEL,
            use_dictionary: true,
            dictionary_size: DEFAULT_DICTIONARY_SIZE,
            dictionary_sample_bytes: DEFAULT_DICTIONARY_SAMPLES,
            use_trigrams: false,
            raw_mode: false,
            auto_transform: false,
            transform_type: TRANSFORM_NONE,
            auto_level: false,
            fast_mode: true,
            use_jl_sketch: false,
            sketch_dim: 64,
            sketch_bits: 4,
        }
    }
}

impl CompressionOptions {
    pub fn new() -> Self { Self::default() }
    pub fn realtime() -> Self { Self { compression_level: 1, use_dictionary: false, use_trigrams: false, ..Self::default() } }
    pub fn balanced() -> Self { Self { compression_level: 3, use_dictionary: true, use_trigrams: false, ..Self::default() } }
    pub fn archival() -> Self { Self { compression_level: 9, use_dictionary: true, dictionary_size: 128 * 1024, use_trigrams: false, ..Self::default() } }
    pub fn search_optimized() -> Self { Self { compression_level: 3, use_dictionary: true, use_trigrams: true, ..Self::default() } }
    pub fn adaptive() -> Self { Self { compression_level: 6, use_dictionary: true, dictionary_size: 128 * 1024, dictionary_sample_bytes: 5 * 1024 * 1024, raw_mode: true, auto_transform: true, transform_type: TRANSFORM_NONE, auto_level: true, ..Self::default() } }
    pub fn dna() -> Self { Self { compression_level: 3, use_dictionary: true, dictionary_size: 64 * 1024, dictionary_sample_bytes: 2 * 1024 * 1024, raw_mode: true, auto_transform: false, transform_type: TRANSFORM_DNA_2BIT, auto_level: false, ..Self::default() } }
    pub fn numeric() -> Self { Self { compression_level: 3, use_dictionary: true, dictionary_size: 32 * 1024, dictionary_sample_bytes: 1 * 1024 * 1024, raw_mode: true, auto_transform: false, transform_type: TRANSFORM_NUMERIC_DELTA, auto_level: false, ..Self::default() } }
    pub fn binary() -> Self { Self { compression_level: 6, use_dictionary: true, dictionary_size: 128 * 1024, dictionary_sample_bytes: 5 * 1024 * 1024, raw_mode: true, auto_transform: false, transform_type: TRANSFORM_BINARY_DELTA, auto_level: false, ..Self::default() } }
    pub fn nibble_split() -> Self { Self { compression_level: 6, use_dictionary: true, dictionary_size: 128 * 1024, dictionary_sample_bytes: 5 * 1024 * 1024, raw_mode: true, auto_transform: false, transform_type: TRANSFORM_NIBBLE_SPLIT, auto_level: false, ..Self::default() } }
    pub fn structured() -> Self { Self { compression_level: 6, use_dictionary: true, dictionary_size: 128 * 1024, dictionary_sample_bytes: 5 * 1024 * 1024, raw_mode: true, auto_transform: false, transform_type: TRANSFORM_STRUCTURED, auto_level: false, ..Self::default() } }
    pub fn level(level: i32) -> Self {
        let level = level.clamp(1, 22);
        match level {
            1..=4 => Self { compression_level: level, use_dictionary: false, dictionary_size: 0, dictionary_sample_bytes: 0, use_trigrams: false, ..Self::default() },
            5..=9 => Self { compression_level: level - 2, use_dictionary: true, dictionary_size: (level as usize - 4) * 16 * 1024, dictionary_sample_bytes: (level as usize - 4) * 256 * 1024, use_trigrams: false, ..Self::default() },
            _ => Self { compression_level: 22, use_dictionary: true, dictionary_size: 1024 * 1024, dictionary_sample_bytes: 20 * 1024 * 1024, use_trigrams: false, ..Self::default() },
        }
    }
    pub fn with_compression_level(mut self, level: i32) -> Self { self.compression_level = level; self }
    pub fn with_dictionary(mut self, enabled: bool) -> Self { self.use_dictionary = enabled; self }
    pub fn with_dictionary_size(mut self, size: usize) -> Self { self.dictionary_size = size; self }
    pub fn with_trigrams(mut self, enabled: bool) -> Self { self.use_trigrams = enabled; self }
    pub fn with_block_size(mut self, size: u64) -> Self { self.block_size = size; self }
    pub fn with_raw_mode(mut self, enabled: bool) -> Self { self.raw_mode = enabled; self }
    pub fn with_auto_transform(mut self, enabled: bool) -> Self { self.auto_transform = enabled; self }
    pub fn with_transform(mut self, transform_type: u8) -> Self { self.transform_type = transform_type; self.auto_transform = false; self }
    pub fn with_auto_level(mut self, enabled: bool) -> Self { self.auto_level = enabled; self }
    pub fn with_fast_mode(mut self, enabled: bool) -> Self { self.fast_mode = enabled; self }
    pub fn with_jl_sketch(mut self, enabled: bool) -> Self { self.use_jl_sketch = enabled; self }
    pub fn with_sketch_dim(mut self, dim: usize) -> Self { self.sketch_dim = dim; self }
    pub fn with_sketch_bits(mut self, bits: u8) -> Self { self.sketch_bits = bits; self }
}
