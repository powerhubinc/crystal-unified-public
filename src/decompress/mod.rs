//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

mod helpers;
mod reader;
mod raw;
pub mod block_raw;

pub use helpers::{decompress_no_dict, decompress_with_dict};
pub use reader::CrystalReaderV10;
pub use raw::RawDecompressorV10;
pub use block_raw::BlockRawReader;
