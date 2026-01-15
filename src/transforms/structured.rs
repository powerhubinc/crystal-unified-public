//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use crate::constants::{DETECT_THRESHOLD, DEFAULT_DEPTH};

const STRUCT_MAGIC: &[u8; 4] = b"RP31";
const HEADER_SIZE: usize = 12;

pub fn encode_structured(data: &[u8]) -> Vec<u8> {
    if data.is_empty() { return vec![]; }
    let mut output = Vec::with_capacity(HEADER_SIZE + data.len());
    output.extend_from_slice(STRUCT_MAGIC);
    output.extend_from_slice(&(data.len() as u64).to_le_bytes());
    let n = data.len();
    let signal: Vec<f64> = data.iter().map(|&b| b as f64).collect();
    let mean: f64 = signal.iter().sum::<f64>() / n as f64;
    let variance: f64 = signal.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let std = variance.sqrt().max(1.0);
    let mut prev_state: u8 = 0;
    for (i, &b) in data.iter().enumerate() {
        let u_norm = (b as f64 - mean) / std;
        let t = (i + 1) as f64 / n as f64;
        let z = t.powi(DEFAULT_DEPTH as i32);
        let c = if z < 1e-10 { u_norm } else { u_norm / z };
        let state = ((c.abs() * 3.0) as u8) % 3;
        let state_diff = (state + 3 - prev_state) % 3;
        let value_6bit = b >> 2;
        let encoded = (value_6bit << 2) | state_diff;
        output.push(encoded);
        prev_state = state;
    }
    output
}

pub fn decode_structured(data: &[u8]) -> Vec<u8> {
    if data.len() < HEADER_SIZE { return data.to_vec(); }
    if &data[0..4] != STRUCT_MAGIC { return data.to_vec(); }
    let orig_len = u64::from_le_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]) as usize;
    let encoded = &data[HEADER_SIZE..];
    if encoded.len() != orig_len { return data.to_vec(); }
    let mut output = Vec::with_capacity(orig_len);
    let mut prev_state: u8 = 0;
    for &encoded_byte in encoded {
        let value_6bit = encoded_byte >> 2;
        let state_diff = encoded_byte & 0x03;
        let state = (prev_state + state_diff) % 3;
        let original = (value_6bit << 2) | state;
        output.push(original);
        prev_state = state;
    }
    output
}

pub fn would_benefit(data: &[u8]) -> bool {
    if data.len() < 64 { return false; }
    let sample_size = data.len().min(4096);
    let sample = &data[..sample_size];
    let score = compute_quick_score(sample);
    score > DETECT_THRESHOLD
}

fn compute_quick_score(data: &[u8]) -> f64 {
    let n = data.len();
    if n == 0 { return 0.0; }
    let mut state_counts = [0usize; 3];
    for &b in data { state_counts[(b % 3) as usize] += 1; }
    let expected = n as f64 / 3.0;
    let variance: f64 = state_counts.iter().map(|&c| (c as f64 - expected).powi(2)).sum::<f64>() / 3.0;
    let max_variance = expected.powi(2) * 2.0 / 3.0;
    if max_variance < 1e-10 { return 1.0; }
    1.0 - (variance / max_variance).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_roundtrip_basic() {
        let data = b"Hello, World! This is a test.";
        let encoded = encode_structured(data);
        let decoded = decode_structured(&encoded);
        assert_eq!(decoded.len(), data.len());
    }
    #[test]
    fn test_empty_data() {
        let data: &[u8] = &[];
        let encoded = encode_structured(data);
        assert!(encoded.is_empty());
    }
}
