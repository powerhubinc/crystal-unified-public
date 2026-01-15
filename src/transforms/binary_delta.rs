//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

pub fn encode_binary_delta(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        let mut output = Vec::with_capacity(8);
        output.extend_from_slice(&0u64.to_le_bytes());
        return output;
    }

    let mut output = Vec::with_capacity(8 + data.len());
    output.extend_from_slice(&(data.len() as u64).to_le_bytes());
    output.push(data[0]);

    for i in 1..data.len() {
        output.push(data[i].wrapping_sub(data[i - 1]));
    }

    output
}

pub fn decode_binary_delta(data: &[u8]) -> Vec<u8> {
    if data.len() < 9 {
        return Vec::new();
    }

    let size = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
    if size == 0 {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(size);
    output.push(data[8]);

    for i in 9..data.len().min(8 + size) {
        let prev = *output.last().unwrap();
        output.push(prev.wrapping_add(data[i]));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_roundtrip() {
        let data: Vec<u8> = (0..100).map(|i| (i * 3) as u8).collect();
        let encoded = encode_binary_delta(&data);
        let decoded = decode_binary_delta(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_binary_empty() {
        let data: Vec<u8> = vec![];
        let encoded = encode_binary_delta(&data);
        let decoded = decode_binary_delta(&encoded);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_binary_sequential() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_binary_delta(&data);
        assert_eq!(encoded[9..].iter().filter(|&&b| b == 1).count(), 255);
    }
}
