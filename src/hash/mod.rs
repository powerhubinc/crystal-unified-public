//! Fast hashing and Bloom filters
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

mod wyhash;
mod bloom;
pub use wyhash::{wyhash, WyHasher, WyHashMap};
pub use bloom::{
    BloomFilter, TrigramBloom,
    bloom_empty, bloom_add_line, bloom_add_word, bloom_query, bloom_might_contain,
    bloom_merge, bloom_add_lines_parallel,
    trigram_bloom_empty, trigram_bloom_add_line, trigram_query_pattern, trigram_might_contain,
    trigram_bloom_merge, trigram_bloom_add_lines_parallel,
};