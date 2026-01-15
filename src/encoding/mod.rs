//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

mod group_varint;
mod byte_search;
mod timestamp_delta;

pub use group_varint::{group_varint_encode, group_varint_decode, write_varint, read_varint_fast, read_varint_unchecked};
pub use byte_search::contains_bytes;
pub use timestamp_delta::{TimestampEncoder, TimestampDecoder, parse_timestamp};
