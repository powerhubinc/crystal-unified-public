//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use crate::constants::DETECT_THRESHOLD;

const BASE_TO_MOD3: [u8; 256] = {
    let mut table = [0u8; 256];
    table[b'A' as usize] = 0;
    table[b'a' as usize] = 0;
    table[b'C' as usize] = 1;
    table[b'c' as usize] = 1;
    table[b'G' as usize] = 2;
    table[b'g' as usize] = 2;
    table[b'T' as usize] = 0;
    table[b't' as usize] = 0;
    table[b'N' as usize] = 0;
    table[b'n' as usize] = 0;
    table
};

#[inline]
pub fn codon_index(b0: u8, b1: u8, b2: u8) -> u8 {
    BASE_TO_MOD3[b0 as usize] * 9 + BASE_TO_MOD3[b1 as usize] * 3 + BASE_TO_MOD3[b2 as usize]
}

pub fn compute_dna_mod3_score(data: &[u8]) -> (f64, u8) {
    let bases: Vec<u8> = data
        .iter()
        .filter(|&&b| matches!(b, b'A' | b'a' | b'C' | b'c' | b'G' | b'g' | b'T' | b't' | b'N' | b'n'))
        .copied()
        .collect();

    if bases.len() < 3 {
        return (0.0, 0);
    }

    let mut state_counts = [0usize; 27];
    let codon_count = bases.len() / 3;

    for i in 0..codon_count {
        let idx = i * 3;
        let state = codon_index(bases[idx], bases[idx + 1], bases[idx + 2]);
        state_counts[state as usize] += 1;
    }

    if codon_count == 0 {
        return (0.0, 0);
    }

    let (dominant_idx, _) = state_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &c)| c)
        .unwrap();

    let expected = codon_count as f64 / 27.0;
    let variance: f64 = state_counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2))
        .sum::<f64>()
        / 27.0;

    let max_variance = expected.powi(2) * 26.0;
    let score = if max_variance > 0.0 {
        (variance / max_variance).sqrt().min(1.0)
    } else {
        0.0
    };

    (score, dominant_idx as u8)
}

pub fn would_benefit_mod3(data: &[u8]) -> bool {
    if data.len() < 100 {
        return false;
    }
    let (dna_score, _) = compute_dna_mod3_score(data);
    dna_score > DETECT_THRESHOLD
}

pub fn encode_dna_2bit(data: &[u8]) -> Vec<u8> {
    let base_count: usize = data
        .iter()
        .filter(|&&b| matches!(b, b'A' | b'a' | b'C' | b'c' | b'G' | b'g' | b'T' | b't' | b'N' | b'n'))
        .count();

    if base_count == 0 {
        let mut output = Vec::with_capacity(8);
        output.extend_from_slice(&0u64.to_le_bytes());
        return output;
    }

    let packed_size = (base_count + 3) / 4;
    let mut output = Vec::with_capacity(8 + packed_size);

    output.extend_from_slice(&(base_count as u64).to_le_bytes());

    let mut current_byte: u8 = 0;
    let mut bit_pos = 0;

    for &byte in data {
        let bits = match byte {
            b'A' | b'a' => Some(0b00),
            b'C' | b'c' => Some(0b01),
            b'G' | b'g' => Some(0b10),
            b'T' | b't' => Some(0b11),
            b'N' | b'n' => Some(0b00),
            _ => None,
        };

        if let Some(b) = bits {
            current_byte |= b << (6 - bit_pos);
            bit_pos += 2;

            if bit_pos == 8 {
                output.push(current_byte);
                current_byte = 0;
                bit_pos = 0;
            }
        }
    }

    if bit_pos > 0 {
        output.push(current_byte);
    }

    output
}

pub fn decode_dna_2bit(data: &[u8]) -> Vec<u8> {
    if data.len() < 8 {
        return Vec::new();
    }

    let base_count = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
    if base_count == 0 {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(base_count);
    let packed_data = &data[8..];

    let mut bases_decoded = 0;
    for &byte in packed_data {
        for shift in [6, 4, 2, 0] {
            if bases_decoded >= base_count {
                break;
            }
            let bits = (byte >> shift) & 0b11;
            let base = match bits {
                0b00 => b'A',
                0b01 => b'C',
                0b10 => b'G',
                0b11 => b'T',
                _ => unreachable!(),
            };
            output.push(base);
            bases_decoded += 1;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dna_roundtrip() {
        let dna = b"ACGTACGTACGT";
        let encoded = encode_dna_2bit(dna);
        let decoded = decode_dna_2bit(&encoded);
        assert_eq!(decoded, dna.to_vec());
    }

    #[test]
    fn test_dna_compression_ratio() {
        let dna = b"ACGTACGTACGTACGTACGTACGTACGTACGT";
        let encoded = encode_dna_2bit(dna);
        assert_eq!(encoded.len(), 16);
    }

    #[test]
    fn test_dna_with_n() {
        let dna = b"ACGTNNNNT";
        let encoded = encode_dna_2bit(dna);
        let decoded = decode_dna_2bit(&encoded);
        assert_eq!(decoded, b"ACGTAAAAT".to_vec());
    }

    #[test]
    fn test_codon_index() {
        assert_eq!(codon_index(b'A', b'A', b'A'), 0);
        assert_eq!(codon_index(b'C', b'C', b'C'), 13);
        assert_eq!(codon_index(b'G', b'G', b'G'), 26);
        assert_eq!(codon_index(b'T', b'T', b'T'), 0);
        assert_eq!(codon_index(b'A', b'C', b'G'), 5);
    }

    #[test]
    fn test_dna_mod3_detection() {
        let uniform = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let (detection_score, dominant) = compute_dna_mod3_score(uniform);
        assert!(detection_score > 0.9);
        assert_eq!(dominant, 0);

        let mixed = b"ACGTACGTACGTACGTACGTACGTACGTACGT";
        let (score2, _) = compute_dna_mod3_score(mixed);
        assert!(score2 < detection_score);
    }

    #[test]
    fn test_would_benefit_mod3() {
        assert!(!would_benefit_mod3(b"ACG"));

        let uniform: Vec<u8> = std::iter::repeat(b'A').take(300).collect();
        assert!(would_benefit_mod3(&uniform));
    }
}
