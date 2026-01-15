//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use crate::constants::*;
use crate::detection::{DataType, compute_structure};
use crate::options::CompressionOptions;

#[derive(Debug, Clone)]
pub struct SmartDetectionResult {
    pub data_type: DataType,
    pub transform: u8,
    pub level: i32,
    pub use_dictionary: bool,
    pub dictionary_size: usize,
    pub score: f64,
    pub is_small_file: bool,
    pub reason: String,
}

impl SmartDetectionResult {
    pub fn to_options(&self) -> CompressionOptions {
        // Always use searchable format (CUZ1) - never raw_mode which produces CUZR
        // Users who want raw mode can explicitly request it via CLI flags
        CompressionOptions::default()
            .with_compression_level(self.level)
            .with_dictionary(self.use_dictionary)
            .with_dictionary_size(self.dictionary_size)
            .with_transform(self.transform)
            .with_raw_mode(false)
            .with_trigrams(true)
    }
}

pub fn smart_detect(data: &[u8]) -> SmartDetectionResult {
    let file_size = data.len();

    if file_size == 0 {
        return SmartDetectionResult {
            data_type: DataType::Text, transform: TRANSFORM_NONE, level: 1,
            use_dictionary: false, dictionary_size: 0, score: 0.0,
            is_small_file: true, reason: "Empty file".to_string(),
        };
    }

    if file_size <= TINY_FILE_THRESHOLD {
        return SmartDetectionResult {
            data_type: DataType::Text, transform: TRANSFORM_NONE, level: 1,
            use_dictionary: false, dictionary_size: 0, score: 0.0,
            is_small_file: true, reason: format!("Tiny file ({} bytes)", file_size),
        };
    }

    let sample_size = file_size.min(DETECTION_SAMPLE_SIZE);
    let sample = &data[..sample_size];
    let data_type = detect_type_from_sample(sample);
    let structure_result = compute_structure(sample);
    let structure_score = structure_result.score;
    let is_small_file = file_size <= SMALL_FILE_THRESHOLD;

    if is_small_file {
        return smart_detect_small_file(data_type, structure_score, file_size);
    }
    smart_detect_normal_file(data_type, structure_score, file_size)
}

fn detect_type_from_sample(sample: &[u8]) -> DataType {
    let n = sample.len();
    if n == 0 { return DataType::Text; }

    let mut dna_count = 0usize;
    let mut digit_count = 0usize;
    let mut text_char_count = 0usize;
    let mut high_byte_count = 0usize;

    for &b in sample {
        match b {
            b'A' | b'C' | b'G' | b'T' | b'N' | b'a' | b'c' | b'g' | b't' | b'n' => dna_count += 1,
            b'0'..=b'9' => digit_count += 1,
            _ => {}
        }
        match b {
            9 | 10 | 13 | 32..=126 => text_char_count += 1,
            128..=255 => high_byte_count += 1,
            _ => {}
        }
    }

    let dna_ratio = dna_count as f64 / n as f64;
    let digit_ratio = digit_count as f64 / n as f64;
    let text_ratio = text_char_count as f64 / n as f64;
    let high_ratio = high_byte_count as f64 / n as f64;

    if dna_ratio > 0.90 { DataType::Dna }
    else if digit_ratio > 0.30 && text_ratio > 0.70 { DataType::Numeric }
    else if high_ratio > 0.20 || text_ratio < 0.80 {
        if has_nibble_pattern(sample) { DataType::StructuredBinary } else { DataType::Binary }
    } else { DataType::Text }
}

fn has_nibble_pattern(sample: &[u8]) -> bool {
    if sample.len() < 64 { return false; }
    let mut high_counts = [0u32; 16];
    for &b in sample.iter().take(256) { high_counts[(b >> 4) as usize] += 1; }
    high_counts[0] > (sample.len().min(256) as u32 / 4)
}

fn smart_detect_small_file(data_type: DataType, structure_score: f64, size: usize) -> SmartDetectionResult {
    let (transform, reason) = match data_type {
        DataType::Dna if size > 100 => (TRANSFORM_DNA_FASTA, "DNA detected"),
        _ if structure_score > DETECT_THRESHOLD && size > 512 => (TRANSFORM_STRUCTURED, "Structure detected"),
        DataType::StructuredBinary if size > 512 => (TRANSFORM_NIBBLE_SPLIT, "Structured binary"),
        _ => (TRANSFORM_NONE, "Small file"),
    };
    SmartDetectionResult {
        data_type, transform, level: 1, use_dictionary: false, dictionary_size: 0,
        score: structure_score, is_small_file: true, reason: format!("{} ({} bytes)", reason, size),
    }
}

fn smart_detect_normal_file(data_type: DataType, structure_score: f64, size: usize) -> SmartDetectionResult {
    let (transform, level, dict_size, reason) = match data_type {
        DataType::Dna => (TRANSFORM_DNA_FASTA, 3, 64 * 1024, "DNA sequence"),
        DataType::Numeric => (TRANSFORM_NUMERIC_DELTA, 3, 32 * 1024, "Numeric data"),
        DataType::StructuredBinary => (TRANSFORM_NIBBLE_SPLIT, 6, 128 * 1024, "Structured binary"),
        DataType::Binary => {
            if structure_score > DETECT_THRESHOLD { (TRANSFORM_STRUCTURED, 6, 128 * 1024, "Binary with structure") }
            else { (TRANSFORM_BINARY_DELTA, 6, 128 * 1024, "Binary data") }
        }
        DataType::Text => {
            if structure_score > DETECT_THRESHOLD { (TRANSFORM_NONE, 6, 128 * 1024, "Text data") }
            else { (TRANSFORM_NONE, 3, 64 * 1024, "Text data") }
        }
        DataType::Random => (TRANSFORM_NONE, 1, 0, "Random data"),
    };
    let (use_dict, actual_dict_size) = if dict_size == 0 { (false, 0) }
    else { (true, dict_size.min(size / 4).max(16 * 1024)) };
    SmartDetectionResult {
        data_type, transform, level, use_dictionary: use_dict, dictionary_size: actual_dict_size,
        score: structure_score, is_small_file: false, reason: format!("{} ({} bytes)", reason, size),
    }
}

pub fn would_bloat(original_size: usize, compressed_size: usize) -> bool {
    if original_size == 0 { return compressed_size > 0; }
    (compressed_size as f64 / original_size as f64) > MIN_COMPRESSION_BENEFIT
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tiny_file() { let result = smart_detect(b"Hi"); assert!(result.is_small_file); }
    #[test]
    fn test_dna_detection() {
        let dna: Vec<u8> = b"ACGTACGTACGTACGTACGTACGTACGT".iter().cycle().take(300).copied().collect();
        assert_eq!(smart_detect(&dna).data_type, DataType::Dna);
    }
}
