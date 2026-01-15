//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use crate::constants::{
    TRANSFORM_BINARY_DELTA, TRANSFORM_DNA_2BIT, TRANSFORM_NONE,
    TRANSFORM_NUMERIC_DELTA, TRANSFORM_NIBBLE_SPLIT,
    DETECTION_SAMPLE_SIZE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Dna,
    Numeric,
    Text,
    Binary,
    StructuredBinary,
    Random,
}

impl DataType {
    pub fn detect(data: &[u8], filename_hint: Option<&str>) -> Self {
        if let Some(name) = filename_hint {
            let lower = name.to_lowercase();
            if lower.ends_with(".fa")
                || lower.ends_with(".fna")
                || lower.ends_with(".fasta")
                || lower.contains("dna")
                || lower.contains("genome")
            {
                return DataType::Dna;
            }
            if lower.ends_with(".csv") || lower.ends_with(".tsv") {
                return DataType::Numeric;
            }
            if lower.ends_with(".xls")
                || lower.ends_with(".xlsx")
                || lower.ends_with(".doc")
                || lower.ends_with(".docx")
                || lower.ends_with(".ppt")
                || lower.ends_with(".pptx")
                || lower.ends_with(".db")
                || lower.ends_with(".sqlite")
                || lower.ends_with(".mdb")
            {
                return DataType::StructuredBinary;
            }
        }

        if data.is_empty() {
            return DataType::Binary;
        }

        let sample_size = data.len().min(DETECTION_SAMPLE_SIZE);
        let sample = &data[..sample_size];

        let mut byte_freq = [0u32; 256];
        for &b in sample {
            byte_freq[b as usize] += 1;
        }

        let total = sample_size as f64;

        let printable: u32 = byte_freq[32..=126].iter().sum();
        let printable_ratio = printable as f64 / total;

        let dna_bases = byte_freq[b'A' as usize]
            + byte_freq[b'C' as usize]
            + byte_freq[b'G' as usize]
            + byte_freq[b'T' as usize]
            + byte_freq[b'a' as usize]
            + byte_freq[b'c' as usize]
            + byte_freq[b'g' as usize]
            + byte_freq[b't' as usize]
            + byte_freq[b'N' as usize]
            + byte_freq[b'n' as usize];
        let newlines = byte_freq[b'\n' as usize] + byte_freq[b'\r' as usize];
        let dna_ratio = (dna_bases + newlines) as f64 / total;

        let digits: u32 = byte_freq[b'0' as usize..=b'9' as usize].iter().sum();
        let numeric_chars = digits
            + byte_freq[b',' as usize]
            + byte_freq[b'.' as usize]
            + byte_freq[b'-' as usize]
            + byte_freq[b'+' as usize]
            + newlines;
        let numeric_ratio = numeric_chars as f64 / total;

        let unique_bytes = byte_freq.iter().filter(|&&c| c > 0).count();
        let entropy = Self::calculate_entropy(&byte_freq, sample_size);

        if entropy > 7.5 && unique_bytes > 240 {
            return DataType::Random;
        }

        if dna_ratio > 0.90 {
            return DataType::Dna;
        }

        if numeric_ratio > 0.70 && digits as f64 / total > 0.3 {
            return DataType::Numeric;
        }

        if printable_ratio < 0.80 {
            if Self::is_structured_binary(sample) {
                return DataType::StructuredBinary;
            }
            return DataType::Binary;
        }

        DataType::Text
    }

    fn is_structured_binary(data: &[u8]) -> bool {
        if data.len() < 256 {
            return false;
        }

        let sample_size = data.len().min(DETECTION_SAMPLE_SIZE);
        let sample = &data[..sample_size];

        let mut high_freq = [0u32; 16];
        let mut low_freq = [0u32; 16];

        for &b in sample {
            high_freq[(b >> 4) as usize] += 1;
            low_freq[(b & 0x0F) as usize] += 1;
        }

        let high_entropy = Self::nibble_entropy(&high_freq, sample_size);
        let low_entropy = Self::nibble_entropy(&low_freq, sample_size);

        let entropy_diff = (high_entropy - low_entropy).abs();

        entropy_diff > 2.0
    }

    fn nibble_entropy(freq: &[u32; 16], total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        let total_f = total as f64;
        let mut entropy = 0.0;
        for &count in freq {
            if count > 0 {
                let p = count as f64 / total_f;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    pub fn calculate_entropy(freq: &[u32; 256], total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        let total_f = total as f64;
        let mut entropy = 0.0;
        for &count in freq {
            if count > 0 {
                let p = count as f64 / total_f;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    pub fn recommended_transform(&self) -> u8 {
        match self {
            DataType::Dna => TRANSFORM_DNA_2BIT,
            DataType::Numeric => TRANSFORM_NUMERIC_DELTA,
            DataType::Binary => TRANSFORM_BINARY_DELTA,
            DataType::StructuredBinary => TRANSFORM_NIBBLE_SPLIT,
            DataType::Text => TRANSFORM_NONE,
            DataType::Random => TRANSFORM_NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_dna() {
        let dna = b"ACGTACGTACGTACGTNNNNACGTACGTACGT";
        assert_eq!(DataType::detect(dna, None), DataType::Dna);
    }

    #[test]
    fn test_detect_text() {
        let text = b"The quick brown fox jumps over the lazy dog";
        assert_eq!(DataType::detect(text, None), DataType::Text);
    }

    #[test]
    fn test_detect_by_filename() {
        let data = b"some data";
        assert_eq!(DataType::detect(data, Some("genome.fa")), DataType::Dna);
        assert_eq!(DataType::detect(data, Some("data.csv")), DataType::Numeric);
    }
}
