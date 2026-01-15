//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use byteorder::{LittleEndian, WriteBytesExt};
use rayon::prelude::*;

use crate::constants::*;
use crate::transforms;
use super::helpers::compress_no_dict;

pub struct BlockRawCompressor {
    block_size: usize,
    level: i32,
    transform: u8,
}

pub struct BlockRawArchive {
    pub block_size: usize,
    pub original_size: u64,
    pub transform: u8,
    pub level: i32,
    pub blocks: Vec<CompressedBlock>,
}

pub struct CompressedBlock {
    pub data: Vec<u8>,
    pub original_size: usize,
}

impl BlockRawCompressor {
    pub fn new() -> Self {
        Self {
            block_size: DEFAULT_BINARY_BLOCK_SIZE,
            level: 3,
            transform: TRANSFORM_NONE,
        }
    }

    pub fn with_block_size(mut self, size: usize) -> Self {
        self.block_size = size.max(1024);
        self
    }

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level.clamp(1, 22);
        self
    }

    pub fn with_transform(mut self, transform: u8) -> Self {
        self.transform = transform;
        self
    }

    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return self.write_empty();
        }

       
        let transformed = if self.transform != TRANSFORM_NONE {
            transforms::apply_transform(data, self.transform)
        } else {
            data.to_vec()
        };

       
        let chunks: Vec<&[u8]> = transformed.chunks(self.block_size).collect();
        let blocks: Vec<CompressedBlock> = chunks
            .par_iter()
            .map(|chunk| {
                let compressed = compress_no_dict(chunk, self.level);
                CompressedBlock {
                    data: compressed,
                    original_size: chunk.len(),
                }
            })
            .collect();

        self.write_archive(data.len(), &blocks)
    }

    pub fn compress_to_archive(&self, data: &[u8]) -> BlockRawArchive {
        if data.is_empty() {
            return BlockRawArchive {
                block_size: self.block_size,
                original_size: 0,
                transform: self.transform,
                level: self.level,
                blocks: Vec::new(),
            };
        }

       
        let transformed = if self.transform != TRANSFORM_NONE {
            transforms::apply_transform(data, self.transform)
        } else {
            data.to_vec()
        };

       
        let chunks: Vec<&[u8]> = transformed.chunks(self.block_size).collect();
        let blocks: Vec<CompressedBlock> = chunks
            .par_iter()
            .map(|chunk| {
                let compressed = compress_no_dict(chunk, self.level);
                CompressedBlock {
                    data: compressed,
                    original_size: chunk.len(),
                }
            })
            .collect();

        BlockRawArchive {
            block_size: self.block_size,
            original_size: data.len() as u64,
            transform: self.transform,
            level: self.level,
            blocks,
        }
    }

    fn write_empty(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(BLOCK_RAW_HEADER_SIZE);
        output.extend_from_slice(BLOCK_RAW_MAGIC);
        output.write_u32::<LittleEndian>(BLOCK_RAW_VERSION).unwrap();
        output.write_u64::<LittleEndian>(FLAG_RAW_MODE).unwrap();
        output.write_u64::<LittleEndian>(self.block_size as u64).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.push(self.transform);
        output.push(self.level as u8);
       
        while output.len() < BLOCK_RAW_HEADER_SIZE {
            output.push(0);
        }
        output
    }

    fn write_archive(&self, original_size: usize, blocks: &[CompressedBlock]) -> Vec<u8> {
        let block_count = blocks.len();
        let index_size = block_count * BLOCK_INDEX_ENTRY_SIZE;
        let data_size: usize = blocks.iter().map(|b| b.data.len()).sum();

        let mut output = Vec::with_capacity(BLOCK_RAW_HEADER_SIZE + index_size + data_size);

       
        let mut flags = FLAG_RAW_MODE;
        if self.transform != TRANSFORM_NONE {
            flags |= FLAG_HAS_TRANSFORM;
        }

        output.extend_from_slice(BLOCK_RAW_MAGIC);
        output.write_u32::<LittleEndian>(BLOCK_RAW_VERSION).unwrap();
        output.write_u64::<LittleEndian>(flags).unwrap();
        output.write_u64::<LittleEndian>(self.block_size as u64).unwrap();
        output.write_u64::<LittleEndian>(original_size as u64).unwrap();
        output.write_u64::<LittleEndian>(block_count as u64).unwrap();
        output.push(self.transform);
        output.push(self.level as u8);
       
        while output.len() < BLOCK_RAW_HEADER_SIZE {
            output.push(0);
        }

       
        let mut offset: u64 = 0;
        for block in blocks {
            output.write_u64::<LittleEndian>(offset).unwrap();
            output.write_u64::<LittleEndian>(block.data.len() as u64).unwrap();
            output.write_u64::<LittleEndian>(block.original_size as u64).unwrap();
            offset += block.data.len() as u64;
        }

       
        for block in blocks {
            output.extend_from_slice(&block.data);
        }

        output
    }
}

impl BlockRawArchive {
    pub fn to_bytes(&self) -> Vec<u8> {
        let block_count = self.blocks.len();
        let index_size = block_count * BLOCK_INDEX_ENTRY_SIZE;
        let data_size: usize = self.blocks.iter().map(|b| b.data.len()).sum();

        let mut output = Vec::with_capacity(BLOCK_RAW_HEADER_SIZE + index_size + data_size);

       
        let mut flags = FLAG_RAW_MODE;
        if self.transform != TRANSFORM_NONE {
            flags |= FLAG_HAS_TRANSFORM;
        }

        output.extend_from_slice(BLOCK_RAW_MAGIC);
        output.write_u32::<LittleEndian>(BLOCK_RAW_VERSION).unwrap();
        output.write_u64::<LittleEndian>(flags).unwrap();
        output.write_u64::<LittleEndian>(self.block_size as u64).unwrap();
        output.write_u64::<LittleEndian>(self.original_size).unwrap();
        output.write_u64::<LittleEndian>(block_count as u64).unwrap();
        output.push(self.transform);
        output.push(self.level as u8);
        while output.len() < BLOCK_RAW_HEADER_SIZE {
            output.push(0);
        }

       
        let mut offset: u64 = 0;
        for block in &self.blocks {
            output.write_u64::<LittleEndian>(offset).unwrap();
            output.write_u64::<LittleEndian>(block.data.len() as u64).unwrap();
            output.write_u64::<LittleEndian>(block.original_size as u64).unwrap();
            offset += block.data.len() as u64;
        }

       
        for block in &self.blocks {
            output.extend_from_slice(&block.data);
        }

        output
    }

    pub fn append(&self, new_data: &[u8], level: i32) -> BlockRawArchive {
        if new_data.is_empty() {
            return BlockRawArchive {
                block_size: self.block_size,
                original_size: self.original_size,
                transform: self.transform,
                level: self.level,
                blocks: self.blocks.clone(),
            };
        }

       
        let transformed = if self.transform != TRANSFORM_NONE {
            transforms::apply_transform(new_data, self.transform)
        } else {
            new_data.to_vec()
        };

       
        let chunks: Vec<&[u8]> = transformed.chunks(self.block_size).collect();
        let new_blocks: Vec<CompressedBlock> = chunks
            .par_iter()
            .map(|chunk| {
                let compressed = compress_no_dict(chunk, level);
                CompressedBlock {
                    data: compressed,
                    original_size: chunk.len(),
                }
            })
            .collect();

       
        let mut all_blocks = self.blocks.clone();
        all_blocks.extend(new_blocks);

        BlockRawArchive {
            block_size: self.block_size,
            original_size: self.original_size + new_data.len() as u64,
            transform: self.transform,
            level: self.level,
            blocks: all_blocks,
        }
    }

    pub fn patch_block(&self, block_idx: usize, new_data: &[u8], level: i32) -> Option<BlockRawArchive> {
        if block_idx >= self.blocks.len() {
            return None;
        }

       
        let transformed = if self.transform != TRANSFORM_NONE {
            transforms::apply_transform(new_data, self.transform)
        } else {
            new_data.to_vec()
        };

        let compressed = compress_no_dict(&transformed, level);
        let new_block = CompressedBlock {
            data: compressed,
            original_size: transformed.len(),
        };

        let mut new_blocks = self.blocks.clone();
        let old_size = new_blocks[block_idx].original_size;
        new_blocks[block_idx] = new_block;

       
        let size_diff = new_data.len() as i64 - old_size as i64;
        let new_original_size = (self.original_size as i64 + size_diff) as u64;

        Some(BlockRawArchive {
            block_size: self.block_size,
            original_size: new_original_size,
            transform: self.transform,
            level: self.level,
            blocks: new_blocks,
        })
    }
}

impl Clone for CompressedBlock {
    fn clone(&self) -> Self {
        CompressedBlock {
            data: self.data.clone(),
            original_size: self.original_size,
        }
    }
}

impl Default for BlockRawCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let data = vec![0u8; 256 * 1024]; // 256KB of zeros
        let compressor = BlockRawCompressor::new().with_block_size(64 * 1024);
        let compressed = compressor.compress(&data);

       
        assert!(compressed.len() < data.len());
        assert!(&compressed[0..4] == BLOCK_RAW_MAGIC);
    }

    #[test]
    fn test_append() {
        let data1 = vec![1u8; 64 * 1024];
        let data2 = vec![2u8; 64 * 1024];

        let compressor = BlockRawCompressor::new().with_block_size(64 * 1024);
        let archive = compressor.compress_to_archive(&data1);
        assert_eq!(archive.blocks.len(), 1);

        let appended = archive.append(&data2, 3);
        assert_eq!(appended.blocks.len(), 2);
        assert_eq!(appended.original_size, 128 * 1024);
    }
}
