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
use rayon::prelude::*;

use crate::constants::*;
use crate::error::{Result, CrystalError};
use crate::transforms;
use crate::compress::block_raw::{BlockRawArchive, CompressedBlock};
use super::helpers::decompress_no_dict;

#[derive(Debug, Clone, Copy)]
pub struct BlockIndexEntry {
    pub offset: u64,
    pub compressed_size: u64,
    pub original_size: u64,
}

pub struct BlockRawReader<'a> {
    data: &'a [u8],
    block_size: u64,
    original_size: u64,
    block_count: u64,
    transform: u8,
    level: u8,
    flags: u64,
    index: Vec<BlockIndexEntry>,
    data_offset: usize,
}

impl<'a> BlockRawReader<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < BLOCK_RAW_HEADER_SIZE {
            return Err(CrystalError::DecompressionFailed("Data too short".into()));
        }

        if &data[0..4] != BLOCK_RAW_MAGIC {
            return Err(CrystalError::InvalidMagic {
                expected: BLOCK_RAW_MAGIC.to_vec(),
                got: data[0..4].to_vec(),
            });
        }

        let mut cursor = Cursor::new(&data[4..]);
        let version = cursor.read_u32::<LittleEndian>()?;
        if version != BLOCK_RAW_VERSION {
            return Err(CrystalError::DecompressionFailed(
                format!("Expected block raw v{}, got v{}", BLOCK_RAW_VERSION, version)
            ));
        }

        let flags = cursor.read_u64::<LittleEndian>()?;
        let block_size = cursor.read_u64::<LittleEndian>()?;
        let original_size = cursor.read_u64::<LittleEndian>()?;
        let block_count = cursor.read_u64::<LittleEndian>()?;
        let transform = cursor.read_u8()?;
        let level = cursor.read_u8()?;

       
        let index_offset = BLOCK_RAW_HEADER_SIZE;
        let index_size = block_count as usize * BLOCK_INDEX_ENTRY_SIZE;

        if data.len() < index_offset + index_size {
            return Err(CrystalError::DecompressionFailed("Index truncated".into()));
        }

        let mut index = Vec::with_capacity(block_count as usize);
        let mut idx_cursor = Cursor::new(&data[index_offset..]);

        for _ in 0..block_count {
            let offset = idx_cursor.read_u64::<LittleEndian>()?;
            let compressed_size = idx_cursor.read_u64::<LittleEndian>()?;
            let original_size = idx_cursor.read_u64::<LittleEndian>()?;
            index.push(BlockIndexEntry {
                offset,
                compressed_size,
                original_size,
            });
        }

        let data_offset = index_offset + index_size;

        Ok(Self {
            data,
            block_size,
            original_size,
            block_count,
            transform,
            level,
            flags,
            index,
            data_offset,
        })
    }

    pub fn is_block_raw_format(data: &[u8]) -> bool {
        data.len() >= 4 && &data[0..4] == BLOCK_RAW_MAGIC
    }

    pub fn block_count(&self) -> u64 {
        self.block_count
    }

    pub fn original_size(&self) -> u64 {
        self.original_size
    }

    pub fn block_size(&self) -> u64 {
        self.block_size
    }

    pub fn level(&self) -> i32 {
        self.level as i32
    }

    pub fn transform(&self) -> u8 {
        self.transform
    }

    pub fn decompress_block(&self, block_idx: u64) -> Result<Vec<u8>> {
        if block_idx >= self.block_count {
            return Err(CrystalError::DecompressionFailed(
                format!("Block {} out of range ({})", block_idx, self.block_count)
            ));
        }

        let entry = &self.index[block_idx as usize];
        let start = self.data_offset + entry.offset as usize;
        let end = start + entry.compressed_size as usize;

        if end > self.data.len() {
            return Err(CrystalError::DecompressionFailed("Block data truncated".into()));
        }

        let compressed = &self.data[start..end];
        let decompressed = decompress_no_dict(compressed, entry.original_size as usize)?;

       
        if (self.flags & FLAG_HAS_TRANSFORM) != 0 && self.transform != TRANSFORM_NONE {
            Ok(transforms::reverse_transform(&decompressed, self.transform))
        } else {
            Ok(decompressed)
        }
    }

    pub fn decompress_all(&self) -> Result<Vec<u8>> {
        if self.block_count == 0 {
            return Ok(Vec::new());
        }

       
        let blocks: Result<Vec<Vec<u8>>> = (0..self.block_count)
            .into_par_iter()
            .map(|i| self.decompress_block(i))
            .collect();

        let blocks = blocks?;

       
        let total_size: usize = blocks.iter().map(|b| b.len()).sum();
        let mut output = Vec::with_capacity(total_size);
        for block in blocks {
            output.extend_from_slice(&block);
        }

        Ok(output)
    }

    pub fn decompress_range(&self, start: u64, len: u64) -> Result<Vec<u8>> {
        if start >= self.original_size {
            return Ok(Vec::new());
        }

        let end = (start + len).min(self.original_size);

       
        let start_block = start / self.block_size;
        let end_block = (end - 1) / self.block_size;

       
        let mut result = Vec::with_capacity(len as usize);
        let mut current_offset = start_block * self.block_size;

        for block_idx in start_block..=end_block {
            if block_idx >= self.block_count {
                break;
            }

            let block_data = self.decompress_block(block_idx)?;
            let block_start = if block_idx == start_block {
                (start - current_offset) as usize
            } else {
                0
            };
            let block_end = if block_idx == end_block {
                ((end - current_offset) as usize).min(block_data.len())
            } else {
                block_data.len()
            };

            if block_start < block_data.len() {
                result.extend_from_slice(&block_data[block_start..block_end]);
            }

            current_offset += self.block_size;
        }

        Ok(result)
    }

    pub fn get_raw_block(&self, block_idx: u64) -> Option<(&[u8], usize)> {
        if block_idx >= self.block_count {
            return None;
        }

        let entry = &self.index[block_idx as usize];
        let start = self.data_offset + entry.offset as usize;
        let end = start + entry.compressed_size as usize;

        if end > self.data.len() {
            return None;
        }

        Some((&self.data[start..end], entry.original_size as usize))
    }

    pub fn to_archive(&self) -> BlockRawArchive {
        let blocks: Vec<CompressedBlock> = (0..self.block_count)
            .map(|i| {
                let (data, orig_size) = self.get_raw_block(i).unwrap();
                CompressedBlock {
                    data: data.to_vec(),
                    original_size: orig_size,
                }
            })
            .collect();

        BlockRawArchive {
            block_size: self.block_size as usize,
            original_size: self.original_size,
            transform: self.transform,
            level: self.level as i32,
            blocks,
        }
    }

    pub fn block_for_offset(&self, offset: u64) -> Option<(u64, u64)> {
        if offset >= self.original_size {
            return None;
        }
        let block_idx = offset / self.block_size;
        let offset_in_block = offset % self.block_size;
        Some((block_idx, offset_in_block))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compress::block_raw::BlockRawCompressor;

    #[test]
    fn test_roundtrip() {
        let data: Vec<u8> = (0..=255u8).cycle().take(256 * 1024).collect();
        let compressor = BlockRawCompressor::new().with_block_size(64 * 1024);
        let compressed = compressor.compress(&data);

        let reader = BlockRawReader::new(&compressed).unwrap();
        assert_eq!(reader.block_count(), 4);

        let decompressed = reader.decompress_all().unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_random_access() {
        let data: Vec<u8> = (0..=255u8).cycle().take(256 * 1024).collect();
        let compressor = BlockRawCompressor::new().with_block_size(64 * 1024);
        let compressed = compressor.compress(&data);

        let reader = BlockRawReader::new(&compressed).unwrap();

       
        let range = reader.decompress_range(100 * 1024, 50 * 1024).unwrap();
        assert_eq!(range, &data[100 * 1024..150 * 1024]);
    }
}
