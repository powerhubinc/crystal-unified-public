//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

mod helpers;
mod burner;
mod parallel;
mod raw;
mod streaming;
pub mod block_raw;

pub use helpers::{compress_no_dict, compress_with_dict};
pub use burner::BurnerV10;
pub use parallel::{ParallelBurnerV10, compress_ultra_fast};
pub use raw::RawCompressorV10;
pub use streaming::StreamingBurnerV10;
pub use block_raw::{BlockRawCompressor, BlockRawArchive, CompressedBlock};
