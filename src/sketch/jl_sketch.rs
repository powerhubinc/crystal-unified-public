//! Johnson-Lindenstrauss random projection for vector indexing
//!
//! Implements the Distributional JL Lemma (Freksen 2021, arXiv:2103.00564):
//! A Rademacher random matrix A ∈ {±1/√m}^{m×d} preserves all pairwise
//! distances within (1 ± ε) with m = O(ε⁻² log(1/δ)).
//!
//! For Crystal, user-provided embedding vectors are projected to a dense
//! sketch in R^dim via a seeded Rademacher matrix. The projection is
//! deterministic (same seed → same matrix), so compressor and reader agree
//! without transmitting the matrix.
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

pub const JL_SKETCH_DIM: usize = 64;
pub const JL_SKETCH_BYTES: usize = JL_SKETCH_DIM * 4;

const JL_SEED: u64 = 0xC7B5_7A1E_4F2D_6B03;

pub type JLSketch = Vec<f32>;

pub struct JLProjection {
    seed: u64,
    dim: usize,
}

#[inline(always)]
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl JLProjection {
    pub fn new() -> Self {
        Self { seed: JL_SEED, dim: JL_SKETCH_DIM }
    }

    pub fn with_dim(dim: usize) -> Self {
        assert!(dim.is_power_of_two() && dim >= 16 && dim <= 256);
        Self { seed: JL_SEED, dim }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn project_sparse(&self, terms: &[(u32, f32)]) -> JLSketch {
        let mut sketch = vec![0.0f32; self.dim];
        let scale = 1.0 / (self.dim as f32).sqrt();

        for &(term_idx, count) in terms {
            let mut state = self.seed ^ (term_idx as u64);
            for k in 0..self.dim {
                let r = splitmix64(&mut state);
                let sign = 1.0 - 2.0 * ((r & 1) as f32);
                sketch[k] += count * sign * scale;
            }
        }
        sketch
    }

    pub fn project_dense(&self, embedding: &[f32]) -> JLSketch {
        if embedding.len() == self.dim {
            return embedding.to_vec();
        }
        let mut sketch = vec![0.0f32; self.dim];
        let scale = 1.0 / (self.dim as f32).sqrt();
        for (j, &val) in embedding.iter().enumerate() {
            if val.abs() < 1e-10 {
                continue;
            }
            let mut state = self.seed ^ (j as u64);
            for k in 0..self.dim {
                let r = splitmix64(&mut state);
                let sign = 1.0 - 2.0 * ((r & 1) as f32);
                sketch[k] += val * sign * scale;
            }
        }
        sketch
    }

    pub fn normalize(sketch: &mut JLSketch) -> f32 {
        let norm_sq: f32 = sketch.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-10 {
            let inv = 1.0 / norm;
            for v in sketch.iter_mut() {
                *v *= inv;
            }
        }
        norm
    }
}

impl Default for JLProjection {
    fn default() -> Self {
        Self::new()
    }
}

pub fn jl_sketch_from_embedding(embedding: &[f32], dim: usize) -> JLSketch {
    let proj = JLProjection::with_dim(dim);
    let mut sketch = if embedding.len() == dim {
        embedding.to_vec()
    } else {
        proj.project_dense(embedding)
    };
    JLProjection::normalize(&mut sketch);
    sketch
}

pub fn jl_sketch_empty() -> JLSketch {
    vec![0.0f32; JL_SKETCH_DIM]
}

pub fn jl_sketch_merge(a: &mut JLSketch, b: &JLSketch) {
    for i in 0..JL_SKETCH_DIM {
        a[i] += b[i];
    }
}

pub fn jl_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splitmix64_deterministic() {
        let mut s1 = 42u64;
        let mut s2 = 42u64;
        for _ in 0..100 {
            assert_eq!(splitmix64(&mut s1), splitmix64(&mut s2));
        }
    }

    #[test]
    fn test_jl_projection_deterministic() {
        let proj = JLProjection::new();
        let terms = vec![(100u32, 3.0f32), (200, 1.0), (500, 2.0)];
        let s1 = proj.project_sparse(&terms);
        let s2 = proj.project_sparse(&terms);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_jl_distance_preservation() {
        let proj = JLProjection::new();

        let a = vec![(10u32, 5.0f32), (20, 3.0), (30, 1.0)];
        let b = vec![(10u32, 4.0f32), (20, 3.0), (30, 1.0)];
        let c = vec![(500u32, 5.0f32), (600, 3.0), (700, 1.0)];

        let sa = proj.project_sparse(&a);
        let sb = proj.project_sparse(&b);
        let sc = proj.project_sparse(&c);

        let sim_ab = jl_cosine_similarity(&sa, &sb);
        let sim_ac = jl_cosine_similarity(&sa, &sc);

        assert!(sim_ab > sim_ac, "similar vectors should have higher cosine similarity");
    }

    #[test]
    fn test_normalize() {
        let mut sketch = vec![3.0f32, 4.0];
        let norm = JLProjection::normalize(&mut sketch);
        assert!((norm - 5.0).abs() < 1e-5);
        let mag: f32 = sketch.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((mag - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_jl_sketch_from_embedding() {
        // Use project_sparse to create a known sketch, then verify from_embedding works
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1).collect();
        let sketch = jl_sketch_from_embedding(&embedding, 64);
        assert_eq!(sketch.len(), 64);
        let norm: f32 = sketch.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "sketch should be unit-normalized");
    }

    #[test]
    fn test_self_similarity() {
        let embedding: Vec<f32> = (0..64).map(|i| (i as f32) * 0.1 + 1.0).collect();
        let sketch = jl_sketch_from_embedding(&embedding, 64);
        let sim = jl_cosine_similarity(&sketch, &sketch);
        assert!((sim - 1.0).abs() < 1e-5);
    }
}
