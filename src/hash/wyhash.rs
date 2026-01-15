//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[inline(always)]
fn wymix(a: u64, b: u64) -> u64 {
    let r = (a as u128).wrapping_mul(b as u128);
    (r as u64) ^ ((r >> 64) as u64)
}

pub fn wyhash(data: &[u8], seed: u64) -> u64 {
    const P0: u64 = 0xa076_1d64_78bd_642f;
    const P1: u64 = 0xe703_7ed1_a0b4_28db;
    const P2: u64 = 0x8ebc_6af0_9c88_c6e3;
    const P3: u64 = 0x5899_65cc_7537_4cc3;

    let len = data.len();
    let mut h = seed ^ P0;
    let mut i = 0usize;

    while i + 32 <= len {
        let a = u64::from_le_bytes(data[i..i + 8].try_into().unwrap());
        let b = u64::from_le_bytes(data[i + 8..i + 16].try_into().unwrap());
        let c = u64::from_le_bytes(data[i + 16..i + 24].try_into().unwrap());
        let d = u64::from_le_bytes(data[i + 24..i + 32].try_into().unwrap());
        h = wymix(a ^ P1, b ^ h) ^ wymix(c ^ P2, d ^ P3);
        i += 32;
    }

    while i + 8 <= len {
        h = wymix(u64::from_le_bytes(data[i..i + 8].try_into().unwrap()) ^ P1, h);
        i += 8;
    }

    if i < len {
        let mut buf = [0u8; 8];
        buf[..len - i].copy_from_slice(&data[i..]);
        h = wymix(u64::from_le_bytes(buf) ^ P2, h);
    }

    wymix(h ^ (len as u64), P3)
}

pub struct WyHasher(u64);

impl Default for WyHasher {
    fn default() -> Self {
        Self(0)
    }
}

impl Hasher for WyHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = wyhash(bytes, self.0);
    }
}

pub type WyHashMap<K, V> = HashMap<K, V, BuildHasherDefault<WyHasher>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wyhash_deterministic() {
        let data = b"hello world";
        let h1 = wyhash(data, 0);
        let h2 = wyhash(data, 0);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_wyhash_different_seeds() {
        let data = b"hello world";
        let h1 = wyhash(data, 0);
        let h2 = wyhash(data, 1);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_wyhash_different_data() {
        let h1 = wyhash(b"hello", 0);
        let h2 = wyhash(b"world", 0);
        assert_ne!(h1, h2);
    }
}
