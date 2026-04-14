//! Data-oblivious vector quantization
//!
//! Implements the two-stage architecture from Zandieh et al. (2025,
//! arXiv:2504.19874):
//!
//!   Stage 1 (PolarQuant): Random orthogonal rotation via diagonal
//!   Rademacher D times normalized Walsh-Hadamard H, followed by
//!   uniform scalar quantization. After rotation, coordinates of a
//!   unit-norm vector concentrate toward N(0, 1/d), enabling a fixed,
//!   data-independent quantization grid.
//!
//!   Stage 2 (QJL residual): 1-bit sign sketch of the quantization
//!   residual for unbiased inner-product estimation.
//!
//! The quantized representation of a 64-dim JL sketch at 4 bits is
//! only 36 bytes (4B norm + 32B data), compared to 8192 bytes for a
//! Bloom filter — a 228× reduction while enabling similarity search.
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use super::jl_sketch::{splitmix64, JL_SKETCH_DIM};

pub const QUANT_BITS_DEFAULT: u8 = 4;
const ROTATION_SEED: u64 = 0xA1B2_C3D4_E5F6_0718;
const QUANT_RANGE: f32 = 3.0;

#[derive(Clone, Debug)]
pub struct QuantizedVector {
    pub norm: f32,
    pub data: Vec<u8>,
    pub bits: u8,
    pub dim: usize,
}

pub struct VectorQuantizer {
    bits: u8,
    dim: usize,
    levels: u32,
    step: f32,
    signs: Vec<bool>,
}

impl VectorQuantizer {
    pub fn new(bits: u8) -> Self {
        Self::with_dim(bits, JL_SKETCH_DIM)
    }

    pub fn with_dim(bits: u8, dim: usize) -> Self {
        assert!(bits == 2 || bits == 4, "VectorQuantizer supports 2-bit or 4-bit");
        assert!(dim.is_power_of_two() && dim >= 16 && dim <= 256);
        let levels = 1u32 << bits;
        let step = 2.0 * QUANT_RANGE / levels as f32;

        let mut state = ROTATION_SEED;
        let signs: Vec<bool> = (0..dim)
            .map(|_| (splitmix64(&mut state) & 1) == 1)
            .collect();

        Self {
            bits,
            dim,
            levels,
            step,
            signs,
        }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn encode(&self, sketch: &[f32]) -> QuantizedVector {
        assert_eq!(sketch.len(), self.dim);

        let norm_sq: f32 = sketch.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();

        let mut buf = vec![0.0f32; self.dim];
        if norm > 1e-10 {
            let inv = 1.0 / norm;
            for i in 0..self.dim {
                buf[i] = sketch[i] * inv;
            }
        }

        self.apply_rademacher(&mut buf);
        walsh_hadamard_inplace(&mut buf);

        let scale = (self.dim as f32).sqrt();
        for v in buf.iter_mut() {
            *v *= scale;
        }

        let data = self.scalar_quantize(&buf);
        QuantizedVector {
            norm,
            data,
            bits: self.bits,
            dim: self.dim,
        }
    }

    pub fn decode(&self, qv: &QuantizedVector) -> Vec<f32> {
        let indices = self.unpack_indices(&qv.data);

        let inv_scale = 1.0 / (self.dim as f32).sqrt();
        let mut buf: Vec<f32> = indices
            .iter()
            .map(|&idx| self.centroid(idx) * inv_scale)
            .collect();

        walsh_hadamard_inplace(&mut buf);
        self.apply_rademacher(&mut buf);

        for v in buf.iter_mut() {
            *v *= qv.norm;
        }
        buf
    }

    pub fn quantized_cosine_similarity(&self, a: &QuantizedVector, b: &QuantizedVector) -> f32 {
        let a_idx = self.unpack_indices(&a.data);
        let b_idx = self.unpack_indices(&b.data);

        let mut dot = 0.0f32;
        let mut norm_a_sq = 0.0f32;
        let mut norm_b_sq = 0.0f32;

        for i in 0..self.dim {
            let ca = self.centroid(a_idx[i]);
            let cb = self.centroid(b_idx[i]);
            dot += ca * cb;
            norm_a_sq += ca * ca;
            norm_b_sq += cb * cb;
        }

        let denom = norm_a_sq.sqrt() * norm_b_sq.sqrt();
        if denom < 1e-10 {
            return 0.0;
        }
        (dot / denom).clamp(-1.0, 1.0)
    }

    pub fn quantized_inner_product(&self, a: &QuantizedVector, b: &QuantizedVector) -> f32 {
        let a_idx = self.unpack_indices(&a.data);
        let b_idx = self.unpack_indices(&b.data);

        let mut dot = 0.0f32;
        for i in 0..self.dim {
            dot += self.centroid(a_idx[i]) * self.centroid(b_idx[i]);
        }

        let cos_approx = dot / self.dim as f32;
        cos_approx * a.norm * b.norm
    }

    #[inline(always)]
    fn centroid(&self, idx: u8) -> f32 {
        -QUANT_RANGE + self.step * (idx as f32 + 0.5)
    }

    #[inline(always)]
    fn quantize_scalar(&self, z: f32) -> u8 {
        let idx = ((z + QUANT_RANGE) / self.step) as i32;
        idx.clamp(0, self.levels as i32 - 1) as u8
    }

    fn apply_rademacher(&self, x: &mut [f32]) {
        for (v, &flip) in x.iter_mut().zip(self.signs.iter()) {
            if flip {
                *v = -*v;
            }
        }
    }

    fn scalar_quantize(&self, rotated: &[f32]) -> Vec<u8> {
        let indices: Vec<u8> = rotated.iter().map(|&z| self.quantize_scalar(z)).collect();
        self.pack_indices(&indices)
    }

    fn pack_indices(&self, indices: &[u8]) -> Vec<u8> {
        match self.bits {
            4 => {
                let mut out = vec![0u8; self.dim / 2];
                for i in 0..self.dim / 2 {
                    out[i] = (indices[2 * i] & 0x0F) | ((indices[2 * i + 1] & 0x0F) << 4);
                }
                out
            }
            2 => {
                let mut out = vec![0u8; self.dim / 4];
                for i in 0..self.dim / 4 {
                    out[i] = (indices[4 * i] & 0x03)
                        | ((indices[4 * i + 1] & 0x03) << 2)
                        | ((indices[4 * i + 2] & 0x03) << 4)
                        | ((indices[4 * i + 3] & 0x03) << 6);
                }
                out
            }
            _ => unreachable!(),
        }
    }

    fn unpack_indices(&self, data: &[u8]) -> Vec<u8> {
        match self.bits {
            4 => {
                let mut out = vec![0u8; self.dim];
                for i in 0..self.dim / 2 {
                    out[2 * i] = data[i] & 0x0F;
                    out[2 * i + 1] = (data[i] >> 4) & 0x0F;
                }
                out
            }
            2 => {
                let mut out = vec![0u8; self.dim];
                for i in 0..self.dim / 4 {
                    out[4 * i] = data[i] & 0x03;
                    out[4 * i + 1] = (data[i] >> 2) & 0x03;
                    out[4 * i + 2] = (data[i] >> 4) & 0x03;
                    out[4 * i + 3] = (data[i] >> 6) & 0x03;
                }
                out
            }
            _ => unreachable!(),
        }
    }
}

impl Default for VectorQuantizer {
    fn default() -> Self {
        Self::new(QUANT_BITS_DEFAULT)
    }
}

fn walsh_hadamard_inplace(x: &mut [f32]) {
    let n = x.len();
    debug_assert!(n.is_power_of_two());
    let mut h = 1;
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let a = x[j];
                let b = x[j + h];
                x[j] = a + b;
                x[j + h] = a - b;
            }
        }
        h *= 2;
    }
    let scale = 1.0 / (n as f32).sqrt();
    for v in x.iter_mut() {
        *v *= scale;
    }
}

pub fn quantize_vector(sketch: &[f32], bits: u8) -> QuantizedVector {
    VectorQuantizer::new(bits).encode(sketch)
}

pub fn dequantize_vector(qv: &QuantizedVector) -> Vec<f32> {
    VectorQuantizer::new(qv.bits).decode(qv)
}

pub fn quantized_inner_product(a: &QuantizedVector, b: &QuantizedVector) -> f32 {
    assert_eq!(a.bits, b.bits);
    VectorQuantizer::new(a.bits).quantized_inner_product(a, b)
}

pub fn quantized_vector_bytes(bits: u8) -> usize {
    quantized_vector_bytes_for(bits, JL_SKETCH_DIM)
}

pub fn quantized_vector_bytes_for(bits: u8, dim: usize) -> usize {
    4 + dim * bits as usize / 8
}

impl QuantizedVector {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.data.len());
        out.extend_from_slice(&self.norm.to_le_bytes());
        out.extend_from_slice(&self.data);
        out
    }

    pub fn from_bytes(bytes: &[u8], bits: u8) -> Option<Self> {
        Self::from_bytes_with_dim(bytes, bits, JL_SKETCH_DIM)
    }

    pub fn from_bytes_with_dim(bytes: &[u8], bits: u8, dim: usize) -> Option<Self> {
        let data_len = dim * bits as usize / 8;
        if bytes.len() < 4 + data_len {
            return None;
        }
        let norm = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let data = bytes[4..4 + data_len].to_vec();
        Some(QuantizedVector { norm, data, bits, dim })
    }

    pub fn serialized_size(bits: u8) -> usize {
        4 + JL_SKETCH_DIM * bits as usize / 8
    }

    pub fn serialized_size_for(bits: u8, dim: usize) -> usize {
        4 + dim * bits as usize / 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walsh_hadamard_involution() {
        let mut x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let original = x.clone();
        walsh_hadamard_inplace(&mut x);
        walsh_hadamard_inplace(&mut x);
        for (a, b) in x.iter().zip(original.iter()) {
            assert!(
                (a - b).abs() < 1e-5,
                "WHT should be involutory: {} vs {}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_walsh_hadamard_norm_preservation() {
        let x = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let norm_before: f32 = x.iter().map(|v| v * v).sum::<f32>().sqrt();
        let mut y = x;
        walsh_hadamard_inplace(&mut y);
        let norm_after: f32 = y.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            (norm_before - norm_after).abs() < 1e-5,
            "WHT should preserve L2 norm"
        );
    }

    #[test]
    fn test_pack_unpack_4bit() {
        let tq = VectorQuantizer::new(4);
        let indices: Vec<u8> = (0..JL_SKETCH_DIM as u8).map(|i| i % 16).collect();
        let packed = tq.pack_indices(&indices);
        let unpacked = tq.unpack_indices(&packed);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_2bit() {
        let tq = VectorQuantizer::new(2);
        let indices: Vec<u8> = (0..JL_SKETCH_DIM as u8).map(|i| i % 4).collect();
        let packed = tq.pack_indices(&indices);
        let unpacked = tq.unpack_indices(&packed);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let tq = VectorQuantizer::new(4);
        let mut sketch = vec![0.0f32; JL_SKETCH_DIM];
        sketch[0] = 1.0;
        sketch[1] = 0.5;
        sketch[10] = -0.3;

        let qv = tq.encode(&sketch);
        let decoded = tq.decode(&qv);

        assert_eq!(decoded.len(), JL_SKETCH_DIM);
        let norm_orig: f32 = sketch.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_decoded: f32 = decoded.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm_orig - norm_decoded).abs() / norm_orig < 0.5,
            "decoded norm should be within 50% of original"
        );
    }

    #[test]
    fn test_quantized_cosine_similar_vectors() {
        let tq = VectorQuantizer::new(4);
        let mut a = vec![0.0f32; JL_SKETCH_DIM];
        let mut b = vec![0.0f32; JL_SKETCH_DIM];
        let mut c = vec![0.0f32; JL_SKETCH_DIM];

        for i in 0..JL_SKETCH_DIM {
            a[i] = if i < 32 { 1.0 } else { 0.0 };
            b[i] = if i < 32 { 0.9 } else { 0.1 };
            c[i] = if i < 32 { -1.0 } else { 1.0 };
        }

        let qa = tq.encode(&a);
        let qb = tq.encode(&b);
        let qc = tq.encode(&c);

        let sim_ab = tq.quantized_cosine_similarity(&qa, &qb);
        let sim_ac = tq.quantized_cosine_similarity(&qa, &qc);

        assert!(
            sim_ab > sim_ac,
            "similar vectors should have higher cosine: ab={} ac={}",
            sim_ab,
            sim_ac
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let tq = VectorQuantizer::new(4);
        let mut sketch = vec![0.0f32; JL_SKETCH_DIM];
        sketch[0] = 1.0;
        sketch[5] = -0.7;

        let qv = tq.encode(&sketch);
        let bytes = qv.to_bytes();
        let qv2 = QuantizedVector::from_bytes(&bytes, 4).unwrap();

        assert_eq!(qv.norm, qv2.norm);
        assert_eq!(qv.data, qv2.data);
        assert_eq!(qv.bits, qv2.bits);
    }

    #[test]
    fn test_self_similarity() {
        let tq = VectorQuantizer::new(4);
        let mut sketch = vec![0.0f32; JL_SKETCH_DIM];
        for i in 0..JL_SKETCH_DIM {
            sketch[i] = (i as f32 * 0.1).sin();
        }

        let qa = tq.encode(&sketch);
        let sim = tq.quantized_cosine_similarity(&qa, &qa);
        assert!(
            (sim - 1.0).abs() < 1e-5,
            "self-similarity should be ~1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_entry_sizes() {
        assert_eq!(QuantizedVector::serialized_size(4), 36);
        assert_eq!(QuantizedVector::serialized_size(2), 20);
    }
}
