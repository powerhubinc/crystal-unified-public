//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use super::wyhash;
use crate::constants::*;
use rayon::prelude::*;

pub type BloomFilter = Vec<u64>;

pub type TrigramBloom = Vec<u64>;

pub fn bloom_empty() -> BloomFilter {
    vec![0u64; BLOOM_SIZE_U64]
}

#[inline(always)]
pub fn bloom_add_word(bloom: &mut BloomFilter, word: &[u8]) {
    let h1 = wyhash(word, 0x1234_5678_9abc_def0);
    let h2 = wyhash(word, 0xfed_cba9_8765_4321);

   
    for i in 0..3u64 {
        let h = h1.wrapping_add(i.wrapping_mul(h2));
        let bit_pos = (h as usize) % BLOOM_SIZE_BITS;
        let word_idx = bit_pos / 64;
        let bit_idx = bit_pos % 64;
        bloom[word_idx] |= 1u64 << bit_idx;
    }
}

#[inline(always)]
pub fn bloom_query(word: &[u8]) -> BloomFilter {
    let mut bloom = bloom_empty();
    bloom_add_word(&mut bloom, word);
    bloom
}

#[inline(always)]
pub fn bloom_might_contain(filter: &BloomFilter, query: &BloomFilter) -> bool {
   
    let mut m0 = 0u64;
    let mut m1 = 0u64;
    let mut m2 = 0u64;
    let mut m3 = 0u64;

    let chunks = BLOOM_SIZE_U64 / 4;
    for i in 0..chunks {
        let base = i * 4;
        m0 |= (filter[base] & query[base]) ^ query[base];
        m1 |= (filter[base + 1] & query[base + 1]) ^ query[base + 1];
        m2 |= (filter[base + 2] & query[base + 2]) ^ query[base + 2];
        m3 |= (filter[base + 3] & query[base + 3]) ^ query[base + 3];
    }

    (m0 | m1 | m2 | m3) == 0
}

pub fn bloom_add_line(bloom: &mut BloomFilter, line: &[u8]) {
    let len = line.len();
    let mut word_start = 0;
    let mut in_word = false;

    for i in 0..len {
        let b = line[i];
        let is_word_char = b.is_ascii_alphanumeric() || b == b'_';
        if is_word_char {
            if !in_word {
                word_start = i;
                in_word = true;
            }
        } else if in_word {
            if i - word_start >= 3 {
                bloom_add_word(bloom, &line[word_start..i]);
            }
            in_word = false;
        }
    }
    if in_word && len - word_start >= 3 {
        bloom_add_word(bloom, &line[word_start..len]);
    }
}

// ============================================================================
// Trigram Bloom Filter
// ============================================================================

pub fn trigram_bloom_empty() -> TrigramBloom {
    vec![0u64; TRIGRAM_BLOOM_SIZE_U64]
}

#[inline(always)]
fn trigram_bloom_add(bloom: &mut TrigramBloom, trigram: &[u8; 3]) {
    let h1 = wyhash(trigram, 0xdead_beef_cafe_babe);
    let h2 = wyhash(trigram, 0x1337_c0de_f00d_d00d);

   
    for i in 0..3u64 {
        let h = h1.wrapping_add(i.wrapping_mul(h2));
        let bit_pos = (h as usize) % TRIGRAM_BLOOM_SIZE_BITS;
        let word_idx = bit_pos / 64;
        let bit_idx = bit_pos % 64;
        bloom[word_idx] |= 1u64 << bit_idx;
    }
}

pub fn trigram_bloom_add_line(bloom: &mut TrigramBloom, line: &[u8]) {
    if line.len() < 3 {
        return;
    }

   
    let len = line.len();
    let mut word_start = 0;
    let mut in_word = false;

    for i in 0..len {
        let b = line[i];
        let is_word_char = b.is_ascii_alphanumeric() || b == b'_';
        if is_word_char {
            if !in_word {
                word_start = i;
                in_word = true;
            }
        } else if in_word {
           
            let word = &line[word_start..i];
            if word.len() >= 3 {
                for j in 0..=(word.len() - 3) {
                    let trigram: [u8; 3] = [
                        word[j].to_ascii_lowercase(),
                        word[j + 1].to_ascii_lowercase(),
                        word[j + 2].to_ascii_lowercase(),
                    ];
                    trigram_bloom_add(bloom, &trigram);
                }
            }
            in_word = false;
        }
    }

   
    if in_word {
        let word = &line[word_start..len];
        if word.len() >= 3 {
            for j in 0..=(word.len() - 3) {
                let trigram: [u8; 3] = [
                    word[j].to_ascii_lowercase(),
                    word[j + 1].to_ascii_lowercase(),
                    word[j + 2].to_ascii_lowercase(),
                ];
                trigram_bloom_add(bloom, &trigram);
            }
        }
    }
}

pub fn trigram_query_pattern(pattern: &[u8]) -> TrigramBloom {
    let mut bloom = trigram_bloom_empty();

   
    let lower: Vec<u8> = pattern.iter().map(|b| b.to_ascii_lowercase()).collect();

    if lower.len() >= 3 {
        for i in 0..=(lower.len() - 3) {
            let trigram: [u8; 3] = [lower[i], lower[i + 1], lower[i + 2]];
            trigram_bloom_add(&mut bloom, &trigram);
        }
    }

    bloom
}

#[inline(always)]
pub fn trigram_might_contain(filter: &TrigramBloom, query: &TrigramBloom) -> bool {
   
    let mut m0 = 0u64;
    let mut m1 = 0u64;
    let mut m2 = 0u64;
    let mut m3 = 0u64;

    let chunks = TRIGRAM_BLOOM_SIZE_U64 / 4;
    for i in 0..chunks {
        let base = i * 4;
        m0 |= (filter[base] & query[base]) ^ query[base];
        m1 |= (filter[base + 1] & query[base + 1]) ^ query[base + 1];
        m2 |= (filter[base + 2] & query[base + 2]) ^ query[base + 2];
        m3 |= (filter[base + 3] & query[base + 3]) ^ query[base + 3];
    }

    (m0 | m1 | m2 | m3) == 0
}

// ============================================================================
// Parallel Bloom Operations (for multi-threaded compression)
// ============================================================================

#[inline]
pub fn bloom_merge(dest: &mut BloomFilter, src: &BloomFilter) {
    for i in 0..BLOOM_SIZE_U64 {
        dest[i] |= src[i];
    }
}

pub fn bloom_add_lines_parallel(lines: &[Vec<u8>]) -> BloomFilter {
    if lines.len() < 100 {
       
        let mut bloom = bloom_empty();
        for line in lines {
            bloom_add_line(&mut bloom, line);
        }
        return bloom;
    }

   
    let partial_blooms: Vec<BloomFilter> = lines
        .par_chunks(500)
        .map(|chunk| {
            let mut bloom = bloom_empty();
            for line in chunk {
                bloom_add_line(&mut bloom, line);
            }
            bloom
        })
        .collect();

   
    let mut result = bloom_empty();
    for partial in partial_blooms {
        bloom_merge(&mut result, &partial);
    }
    result
}

#[inline]
pub fn trigram_bloom_merge(dest: &mut TrigramBloom, src: &TrigramBloom) {
    for i in 0..TRIGRAM_BLOOM_SIZE_U64 {
        dest[i] |= src[i];
    }
}

pub fn trigram_bloom_add_lines_parallel(lines: &[Vec<u8>]) -> TrigramBloom {
    if lines.len() < 100 {
        let mut bloom = trigram_bloom_empty();
        for line in lines {
            trigram_bloom_add_line(&mut bloom, line);
        }
        return bloom;
    }

    let partial_blooms: Vec<TrigramBloom> = lines
        .par_chunks(500)
        .map(|chunk| {
            let mut bloom = trigram_bloom_empty();
            for line in chunk {
                trigram_bloom_add_line(&mut bloom, line);
            }
            bloom
        })
        .collect();

    let mut result = trigram_bloom_empty();
    for partial in partial_blooms {
        trigram_bloom_merge(&mut result, &partial);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_add_query() {
        let mut bloom = bloom_empty();
        bloom_add_word(&mut bloom, b"error");

        let query = bloom_query(b"error");
        assert!(bloom_might_contain(&bloom, &query));

        let _query2 = bloom_query(b"nothere");
       
       
    }

    #[test]
    fn test_trigram_bloom() {
        let mut bloom = trigram_bloom_empty();
        trigram_bloom_add_line(&mut bloom, b"error_message");

        let query = trigram_query_pattern(b"err");
        assert!(trigram_might_contain(&bloom, &query));
    }
}
