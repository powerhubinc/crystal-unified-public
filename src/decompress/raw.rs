//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use zstd::zstd_safe::DDict;

use crate::constants::*;
use crate::error::{Result, CrystalError};
use crate::transforms;
use super::helpers::{decompress_no_dict, decompress_with_ddict};

pub struct RawDecompressorV10;

impl RawDecompressorV10 {
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < RAW_HEADER_SIZE {
            return Err(CrystalError::DecompressionFailed("Data too short".into()));
        }

        if &data[0..4] != RAW_MAGIC {
            return Err(CrystalError::InvalidMagic {
                expected: RAW_MAGIC.to_vec(),
                got: data[0..4].to_vec(),
            });
        }

        let mut cursor = Cursor::new(&data[4..]);
        let version = cursor.read_u32::<LittleEndian>()?;
        if version != RAW_VERSION {
            return Err(CrystalError::DecompressionFailed(
                format!("Expected raw v{}, got v{}", RAW_VERSION, version)
            ));
        }

        let flags = cursor.read_u64::<LittleEndian>()?;
        let transform_type = cursor.read_u8()?;
        let _level = cursor.read_u8()?;
       
        for _ in 0..6 {
            cursor.read_u8()?;
        }

        let original_size = cursor.read_u64::<LittleEndian>()? as usize;
        let transformed_size = cursor.read_u64::<LittleEndian>()? as usize;
        let dict_size = cursor.read_u64::<LittleEndian>()? as usize;
        let _orig_dict_size = cursor.read_u64::<LittleEndian>()? as usize;

        if original_size == 0 {
            return Ok(Vec::new());
        }

        let has_dictionary = (flags & FLAG_HAS_DICTIONARY) != 0;
        let has_transform = (flags & FLAG_HAS_TRANSFORM) != 0;

        let dict_offset = RAW_HEADER_SIZE;
        let data_offset = dict_offset + dict_size;

       
        let ddict = if has_dictionary && dict_size > 0 {
            let compressed_dict = &data[dict_offset..dict_offset + dict_size];
            if let Ok(raw_dict) = zstd::stream::decode_all(compressed_dict) {
                Some(DDict::create(&raw_dict))
            } else {
                None
            }
        } else {
            None
        };

       
        let compressed_data = &data[data_offset..];
        let decompressed = if let Some(ref ddict) = ddict {
            decompress_with_ddict(compressed_data, ddict, transformed_size)?
        } else {
            decompress_no_dict(compressed_data, transformed_size)?
        };

       
        let output = if has_transform && transform_type != TRANSFORM_NONE {
            transforms::reverse_transform(&decompressed, transform_type)
        } else {
            decompressed
        };

        Ok(output)
    }

    pub fn is_raw_format(data: &[u8]) -> bool {
        data.len() >= 4 && &data[0..4] == RAW_MAGIC
    }
}
