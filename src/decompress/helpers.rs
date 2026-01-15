//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use zstd::zstd_safe::{DCtx, ResetDirective, DDict};
use crate::error::{Result, CrystalError};

pub fn decompress_no_dict(data: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
    let mut output = vec![0u8; decompressed_size];
    match zstd::bulk::decompress_to_buffer(data, &mut output) {
        Ok(_) => Ok(output),
        Err(e) => Err(CrystalError::DecompressionFailed(e.to_string())),
    }
}

pub fn decompress_with_ddict(data: &[u8], ddict: &DDict<'_>, decompressed_size: usize) -> Result<Vec<u8>> {
    let mut dctx = DCtx::create();
    dctx.reset(ResetDirective::SessionOnly)
        .map_err(|e| CrystalError::DecompressionFailed(e.to_string()))?;

    dctx.ref_ddict(ddict)
        .map_err(|e| CrystalError::DecompressionFailed(e.to_string()))?;

    let mut output = vec![0u8; decompressed_size];
    match dctx.decompress(&mut output, data) {
        Ok(_) => Ok(output),
        Err(e) => Err(CrystalError::DecompressionFailed(e.to_string())),
    }
}

pub fn decompress_with_dict(data: &[u8], dict: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
    let decoder_dict = DDict::create(dict);
    decompress_with_ddict(data, &decoder_dict, decompressed_size)
}
