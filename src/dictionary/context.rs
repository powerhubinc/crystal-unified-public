//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use zstd::zstd_safe::{CCtx, DCtx, CParameter, ResetDirective, CDict, DDict};
use crate::error::{Result, CrystalError};

pub struct ReusableCCtx {
    cctx: CCtx<'static>,
    cdict: Option<CDict<'static>>,
}

impl ReusableCCtx {
    pub fn new(level: i32) -> Self {
        let mut cctx = CCtx::create();
        cctx.set_parameter(CParameter::CompressionLevel(level)).ok();
        cctx.set_parameter(CParameter::ChecksumFlag(false)).ok();
        cctx.set_parameter(CParameter::ContentSizeFlag(false)).ok();
        cctx.set_parameter(CParameter::NbWorkers(0)).ok();

        Self { cctx, cdict: None }
    }

    pub fn with_dictionary(mut self, dict_data: &[u8], level: i32) -> Self {
        if !dict_data.is_empty() {
            self.cdict = Some(CDict::create(dict_data, level));
        }
        self
    }

    pub fn compress(&mut self, data: &[u8]) -> Vec<u8> {
        self.cctx.reset(ResetDirective::SessionOnly).ok();

       
        if let Some(ref cdict) = self.cdict {
            self.cctx.ref_cdict(cdict).ok();
        }

        let max_size = zstd::zstd_safe::compress_bound(data.len());
        let mut output = vec![0u8; max_size];

        match self.cctx.compress2(&mut output, data) {
            Ok(size) => {
                output.truncate(size);
                output
            }
            Err(_) => {
               
                zstd::stream::encode_all(&data[..], 1).unwrap_or_else(|_| data.to_vec())
            }
        }
    }
}

pub struct ReusableDCtx {
    dctx: DCtx<'static>,
    ddict: Option<DDict<'static>>,
}

impl ReusableDCtx {
    pub fn new() -> Self {
        Self {
            dctx: DCtx::create(),
            ddict: None,
        }
    }

    pub fn with_dictionary(mut self, dict_data: &[u8]) -> Self {
        if !dict_data.is_empty() {
            self.ddict = Some(DDict::create(dict_data));
        }
        self
    }

    pub fn decompress_exact(&mut self, data: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
        self.dctx.reset(ResetDirective::SessionOnly)
            .map_err(|e| CrystalError::DecompressionFailed(e.to_string()))?;

       
        if let Some(ref ddict) = self.ddict {
            self.dctx.ref_ddict(ddict)
                .map_err(|e| CrystalError::DecompressionFailed(e.to_string()))?;
        }

        let mut output = vec![0u8; decompressed_size];
        match self.dctx.decompress(&mut output, data) {
            Ok(_) => Ok(output),
            Err(e) => Err(CrystalError::DecompressionFailed(e.to_string())),
        }
    }
}

impl Default for ReusableDCtx {
    fn default() -> Self {
        Self::new()
    }
}
