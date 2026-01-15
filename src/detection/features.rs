//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use std::collections::HashMap;

use super::DataType;

#[derive(Debug, Clone)]
pub struct CompressionFeatures {
    pub entropy: f64,
    pub lz_match_ratio: f64,
    pub byte_variety: usize,
    pub data_type: DataType,
}

impl CompressionFeatures {
    pub fn extract(data: &[u8], filename_hint: Option<&str>) -> Self {
        if data.is_empty() {
            return Self {
                entropy: 0.0,
                lz_match_ratio: 0.0,
                byte_variety: 0,
                data_type: DataType::Binary,
            };
        }

        let sample_size = data.len().min(64 * 1024);
        let sample = &data[..sample_size];

        let mut byte_freq = [0u32; 256];
        for &b in sample {
            byte_freq[b as usize] += 1;
        }

        let entropy = DataType::calculate_entropy(&byte_freq, sample_size);
        let byte_variety = byte_freq.iter().filter(|&&c| c > 0).count();

        let lz_match_ratio = Self::estimate_lz_matches(sample);

        let data_type = DataType::detect(data, filename_hint);

        Self {
            entropy,
            lz_match_ratio,
            byte_variety,
            data_type,
        }
    }

    fn estimate_lz_matches(data: &[u8]) -> f64 {
        if data.len() < 8 {
            return 0.0;
        }

        let mut hash_table: HashMap<u32, usize> = HashMap::with_capacity(4096);
        let mut bytes_matched = 0usize;
        let mut pos = 0;

        while pos + 4 <= data.len() {
            let hash = Self::hash4(&data[pos..]);

            if let Some(&prev_pos) = hash_table.get(&hash) {
                if prev_pos + 4 <= pos {
                    let mut match_len = 4;
                    while pos + match_len < data.len()
                        && prev_pos + match_len < pos
                        && data[prev_pos + match_len] == data[pos + match_len]
                        && match_len < 256
                    {
                        match_len += 1;
                    }
                    bytes_matched += match_len;
                    pos += match_len;
                    continue;
                }
            }

            hash_table.insert(hash, pos);
            pos += 1;
        }

        bytes_matched as f64 / data.len() as f64
    }

    #[inline]
    fn hash4(data: &[u8]) -> u32 {
        let v = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        v.wrapping_mul(0x9E3779B9)
    }

    pub fn compressibility(&self) -> f64 {
        let entropy_score = 1.0 - (self.entropy / 8.0);
        let lz_score = self.lz_match_ratio;
        0.5 * entropy_score + 0.5 * lz_score
    }

    pub fn recommended_level(&self) -> i32 {
        match self.compressibility() {
            c if c > 0.7 => 1,
            c if c > 0.5 => 3,
            c if c > 0.3 => 6,
            c if c > 0.1 => 9,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_features() {
        let data = b"Hello world! Hello world! Hello world!";
        let features = CompressionFeatures::extract(data, None);
        assert!(features.entropy > 0.0);
        assert!(features.lz_match_ratio > 0.0);
        assert!(features.byte_variety > 0);
    }

    #[test]
    fn test_compressibility() {
        let repetitive = b"hello world hello world hello world hello world hello world hello world";
        let features = CompressionFeatures::extract(repetitive, None);
        assert!(features.compressibility() > 0.3);
    }
}
