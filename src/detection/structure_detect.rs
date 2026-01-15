//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use crate::constants::{DETECT_THRESHOLD, DETECT_SAMPLE_SIZE, DEFAULT_DEPTH, MIN_DEPTH, MAX_DEPTH};

#[derive(Debug, Clone, Copy)]
pub struct StructureResult {
    pub score: f64,
    pub has_structure: bool,
    pub recommend_transform: bool,
}

impl StructureResult {
    pub fn is_detected(&self) -> bool { self.has_structure }
}

pub fn compute_structure(data: &[u8]) -> StructureResult {
    let current_score = compute_value(data, DEFAULT_DEPTH);
    let has_structure = current_score > DETECT_THRESHOLD;
    StructureResult { score: current_score, has_structure, recommend_transform: has_structure }
}

pub fn compute_structure_at_depth(data: &[u8], depth: usize) -> StructureResult {
    let depth = depth.clamp(MIN_DEPTH, MAX_DEPTH);
    let current_score = compute_value(data, depth);
    let has_structure = current_score > DETECT_THRESHOLD;
    StructureResult { score: current_score, has_structure, recommend_transform: has_structure }
}

pub fn find_optimal_depth(data: &[u8]) -> (usize, f64) {
    let test_depths = [3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 45, 60, 90, 120, 180];
    let mut best_depth = DEFAULT_DEPTH;
    let mut best_score = 0.0_f64;
    for &depth in &test_depths {
        let current_score = compute_value(data, depth);
        if current_score > best_score { best_score = current_score; best_depth = depth; }
    }
    (best_depth, best_score)
}

fn compute_value(data: &[u8], depth: usize) -> f64 {
    if data.is_empty() { return 0.0; }
    let sample: &[u8] = if data.len() > DETECT_SAMPLE_SIZE { &data[..DETECT_SAMPLE_SIZE] } else { data };
    let signal: Vec<f64> = sample.iter().map(|&b| b as f64).collect();
    let n = signal.len();
    let mean: f64 = signal.iter().sum::<f64>() / n as f64;
    let variance: f64 = signal.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let std = variance.sqrt();
    if std < 1e-10 { return 1.0; }
    let u_norm: Vec<f64> = signal.iter().map(|x| (x - mean) / std).collect();
    let c: Vec<f64> = u_norm.iter().enumerate().map(|(i, &u)| {
        let t = (i + 1) as f64 / n as f64;
        let z = t.powi(depth as i32);
        if z < 1e-10 { u } else { u / z }
    }).collect();
    let c_max = c.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    if c_max < 1e-10 { return 1.0; }
    let c_norm: Vec<f64> = c.iter().map(|x| x / c_max).collect();
    let angles: Vec<f64> = c_norm.iter().map(|c| std::f64::consts::PI * c * 3.0).collect();
    let sum_cos: f64 = angles.iter().map(|p| p.cos()).sum();
    let sum_sin: f64 = angles.iter().map(|p| p.sin()).sum();
    ((sum_cos / n as f64).powi(2) + (sum_sin / n as f64).powi(2)).sqrt()
}

pub fn compare_depths(data: &[u8]) -> Vec<(usize, f64)> {
    let depths = [1, 2, 3, 4, 5, 6, 9, 12];
    depths.iter().map(|&depth| (depth, compute_value(data, depth))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constant_signal() {
        let data = vec![128u8; 1000];
        assert!(compute_structure(&data).score >= 0.99);
    }
    #[test]
    fn test_empty_data() {
        assert_eq!(compute_structure(&[]).score, 0.0);
    }
}
