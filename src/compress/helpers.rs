//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use zstd::zstd_safe::{CCtx, CParameter, ResetDirective, CDict};

pub fn compress_no_dict(data: &[u8], level: i32) -> Vec<u8> {
    let mut cctx = CCtx::create();
    cctx.reset(ResetDirective::SessionOnly).ok();
    cctx.set_parameter(CParameter::CompressionLevel(level)).ok();
    cctx.set_parameter(CParameter::ChecksumFlag(false)).ok();
    cctx.set_parameter(CParameter::ContentSizeFlag(false)).ok();
    cctx.set_parameter(CParameter::NbWorkers(0)).ok();

    let max_size = zstd::zstd_safe::compress_bound(data.len());
    let mut output = vec![0u8; max_size];

    match cctx.compress2(&mut output, data) {
        Ok(size) => {
            output.truncate(size);
            output
        }
        Err(_) => {
            zstd::stream::encode_all(&data[..], level).unwrap_or_else(|_| data.to_vec())
        }
    }
}

pub fn compress_with_dict(data: &[u8], dict: &CDict, level: i32) -> Vec<u8> {
    let mut cctx = CCtx::create();
    cctx.reset(ResetDirective::SessionOnly).ok();
    cctx.set_parameter(CParameter::CompressionLevel(level)).ok();
    cctx.set_parameter(CParameter::ChecksumFlag(false)).ok();
    cctx.set_parameter(CParameter::ContentSizeFlag(false)).ok();
    cctx.set_parameter(CParameter::NbWorkers(0)).ok();
    cctx.ref_cdict(dict).ok();

    let max_size = zstd::zstd_safe::compress_bound(data.len());
    let mut output = vec![0u8; max_size];

    match cctx.compress2(&mut output, data) {
        Ok(size) => {
            output.truncate(size);
            output
        }
        Err(_) => compress_no_dict(data, level),
    }
}
