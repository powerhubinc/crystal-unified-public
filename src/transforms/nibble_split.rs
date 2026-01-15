//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

pub fn encode_nibble_split(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return vec![0, 0, 0, 0];
    }

    let packed_len = (data.len() + 1) / 2;
    let mut output = Vec::with_capacity(4 + packed_len * 2);

    output.extend_from_slice(&(data.len() as u32).to_le_bytes());

    let mut high_nibbles = vec![0u8; packed_len];
    let mut low_nibbles = vec![0u8; packed_len];

    for (i, chunk) in data.chunks(2).enumerate() {
        let high1 = chunk[0] >> 4;
        let high2 = if chunk.len() > 1 { chunk[1] >> 4 } else { 0 };
        high_nibbles[i] = (high1 << 4) | high2;

        let low1 = chunk[0] & 0x0F;
        let low2 = if chunk.len() > 1 { chunk[1] & 0x0F } else { 0 };
        low_nibbles[i] = (low1 << 4) | low2;
    }

    output.extend_from_slice(&high_nibbles);
    output.extend_from_slice(&low_nibbles);

    output
}

pub fn decode_nibble_split(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 {
        return Vec::new();
    }

    let original_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if original_len == 0 {
        return Vec::new();
    }

    let packed_len = (original_len + 1) / 2;

    if data.len() < 4 + packed_len * 2 {
        return Vec::new();
    }

    let high_nibbles = &data[4..4 + packed_len];
    let low_nibbles = &data[4 + packed_len..4 + packed_len * 2];

    let mut output = Vec::with_capacity(original_len);

    for i in 0..original_len {
        let packed_idx = i / 2;
        let is_first = i % 2 == 0;

        let high = if is_first {
            high_nibbles[packed_idx] >> 4
        } else {
            high_nibbles[packed_idx] & 0x0F
        };

        let low = if is_first {
            low_nibbles[packed_idx] >> 4
        } else {
            low_nibbles[packed_idx] & 0x0F
        };

        output.push((high << 4) | low);
    }

    output
}

pub fn would_benefit(data: &[u8]) -> bool {
    if data.len() < 256 {
        return false;
    }

    let sample_size = data.len().min(16 * 1024);
    let sample = &data[..sample_size];

    let mut high_freq = [0u32; 16];
    let mut low_freq = [0u32; 16];

    for &b in sample {
        high_freq[(b >> 4) as usize] += 1;
        low_freq[(b & 0x0F) as usize] += 1;
    }

    let high_entropy = nibble_entropy(&high_freq, sample_size);
    let low_entropy = nibble_entropy(&low_freq, sample_size);

    let entropy_diff = (high_entropy - low_entropy).abs();

    let high_zeros = high_freq[0] as f64 / sample_size as f64;

    entropy_diff > 0.5 || high_entropy < 3.0 || low_entropy < 3.0 || high_zeros > 0.25
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let original: Vec<u8> = (0..1000).map(|i| (i * 7 + 13) as u8).collect();
        let encoded = encode_nibble_split(&original);
        let decoded = decode_nibble_split(&encoded);
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_empty() {
        let encoded = encode_nibble_split(&[]);
        let decoded = decode_nibble_split(&encoded);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_single_byte() {
        let original = vec![0xAB];
        let encoded = encode_nibble_split(&original);
        let decoded = decode_nibble_split(&encoded);
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_odd_length() {
        let original = vec![0x12, 0x34, 0x56];
        let encoded = encode_nibble_split(&original);
        let decoded = decode_nibble_split(&encoded);
        assert_eq!(original, decoded);
    }
}
