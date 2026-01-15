//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CrystalError>;

#[derive(Error, Debug)]
pub enum CrystalError {
    #[error("Invalid magic bytes: expected {expected:?}, got {got:?}")]
    InvalidMagic { expected: Vec<u8>, got: Vec<u8> },

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Block out of bounds: index {index}, count {count}")]
    BlockOutOfBounds { index: u64, count: u64 },

    #[error("Row out of bounds: index {index}, count {count}")]
    RowOutOfBounds { index: u64, count: u64 },
}
