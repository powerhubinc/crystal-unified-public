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
use zstd::zstd_safe::CDict;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::constants::*;
use crate::options::CompressionOptions;
use crate::hash::{BloomFilter, TrigramBloom};
use crate::block::{BlockData, ParseBuffer};
use crate::dictionary::train_dictionary;
use super::helpers::{compress_no_dict, compress_with_dict};

type BlockResult = (Vec<u8>, BloomFilter, Option<TrigramBloom>, Option<Vec<u8>>, usize);

pub struct ParallelBurnerV10 {
    options: CompressionOptions,
    trailing_newline: bool,
}

impl ParallelBurnerV10 {
    pub fn new() -> Self {
        Self::with_options(CompressionOptions::default())
    }

    pub fn with_options(options: CompressionOptions) -> Self {
        Self {
            options,
            trailing_newline: true,
        }
    }

    pub fn set_trailing_newline(&mut self, has_trailing: bool) {
        self.trailing_newline = has_trailing;
    }

    pub fn compress_parallel(&self, lines: &[&[u8]]) -> Vec<u8> {
        let block_size = self.options.block_size as usize;
        let total_lines = lines.len();

        if total_lines == 0 {
            return self.write_empty_file();
        }

        let fast_mode = self.options.fast_mode;
        let dictionary = if self.options.use_dictionary {
            let sample_blocks = std::cmp::min(
                (self.options.dictionary_sample_bytes / 50000).max(1),
                total_lines / block_size + 1
            );

            let samples: Vec<Vec<u8>> = lines
                .chunks(block_size)
                .take(sample_blocks)
                .map(|chunk| {
                    let mut block = BlockData::with_fast_mode(false, fast_mode);
                    let mut parse_buf = ParseBuffer::new();
                    for line in chunk {
                        block.add_line(line, &mut parse_buf);
                    }
                    if fast_mode { block.serialize_fast() } else { block.serialize() }
                })
                .collect();

            train_dictionary(&samples, self.options.dictionary_size)
        } else {
            None
        };

        let encoder_dict: Option<CDict<'static>> = dictionary.as_ref().map(|d| {
            CDict::create(d, self.options.compression_level)
        });

        let use_trigrams = self.options.use_trigrams;
        let level = self.options.compression_level;

        let compressed_results: Vec<BlockResult> = lines
            .par_chunks(block_size)
            .map(|chunk| {
                let mut block = BlockData::with_fast_mode(use_trigrams, fast_mode);
                let mut parse_buf = ParseBuffer::new();

                if fast_mode {
                    for line in chunk {
                        block.add_line_fast(line);
                    }
                    block.finalize_blooms_parallel();
                } else {
                    for line in chunk {
                        block.add_line(line, &mut parse_buf);
                    }
                }

                let raw_data = if fast_mode { block.serialize_fast() } else { block.serialize() };
                let decompressed_size = raw_data.len();

                let compressed = if let Some(ref dict) = encoder_dict {
                    compress_with_dict(&raw_data, dict, level)
                } else {
                    compress_no_dict(&raw_data, level)
                };

                let sketch_bytes: Option<Vec<u8>> = None;

                (compressed, block.bloom, block.trigram_bloom, sketch_bytes, decompressed_size)
            })
            .collect();

        self.write_final_file(compressed_results, total_lines as u64, dictionary, fast_mode)
    }

    fn write_empty_file(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(HEADER_SIZE);
        output.extend_from_slice(MAGIC);
        output.write_u32::<LittleEndian>(VERSION).unwrap();
        output.write_u64::<LittleEndian>(self.options.block_size).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        let mut flags = if self.trailing_newline { FLAG_TRAILING_NEWLINE } else { 0 };
        if self.options.fast_mode {
            flags |= FLAG_FAST_MODE;
        }
        output.write_u64::<LittleEndian>(flags).unwrap();
        output.write_i32::<LittleEndian>(self.options.compression_level).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        while output.len() < HEADER_SIZE {
            output.push(0);
        }
        output
    }

    fn write_final_file(
        &self,
        blocks: Vec<BlockResult>,
        row_count: u64,
        dictionary: Option<Vec<u8>>,
        fast_mode: bool,
    ) -> Vec<u8> {
        self.write_final_file_inner(blocks, row_count, dictionary, fast_mode, self.trailing_newline)
    }

    fn write_final_file_inner(
        &self,
        blocks: Vec<BlockResult>,
        row_count: u64,
        dictionary: Option<Vec<u8>>,
        fast_mode: bool,
        trailing_newline: bool,
    ) -> Vec<u8> {
        let block_count = blocks.len();

        let dict_compressed = dictionary.as_ref().map(|d| {
            zstd::stream::encode_all(&d[..], 3).unwrap_or_else(|_| d.clone())
        });
        let dict_size = dict_compressed.as_ref().map(|d| d.len()).unwrap_or(0);
        let orig_dict_size = dictionary.as_ref().map(|d| d.len()).unwrap_or(0);

        let has_jl_sketch = blocks.first().map(|b| b.3.is_some()).unwrap_or(false);

        let mut block_offsets = Vec::with_capacity(block_count);
        let mut block_lengths = Vec::with_capacity(block_count);
        let mut decompressed_sizes = Vec::with_capacity(block_count);
        let mut current_offset = 0u64;

        for (data, _, _, _, decompressed_size) in &blocks {
            block_offsets.push(current_offset);
            block_lengths.push(data.len() as u64);
            decompressed_sizes.push(*decompressed_size as u64);
            current_offset += data.len() as u64;
        }

        let index_size = block_count * BLOCK_INDEX_ENTRY_SIZE;
        let bloom_size = block_count * BLOOM_SIZE_BYTES;
        let trigram_size = if self.options.use_trigrams {
            block_count * TRIGRAM_BLOOM_SIZE_BYTES
        } else {
            0
        };
        let entry_size = jl_sketch_entry_size(self.options.sketch_dim, self.options.sketch_bits);
        let sketch_size = if has_jl_sketch {
            block_count * entry_size
        } else {
            0
        };
        let data_size: usize = blocks.iter().map(|(d, _, _, _, _)| d.len()).sum();

        let mut output = Vec::with_capacity(
            HEADER_SIZE + dict_size + index_size + bloom_size + trigram_size + sketch_size + data_size,
        );

        // Header
        output.extend_from_slice(MAGIC);
        output.write_u32::<LittleEndian>(VERSION).unwrap();
        output.write_u64::<LittleEndian>(self.options.block_size).unwrap();
        output.write_u64::<LittleEndian>(row_count).unwrap();
        output.write_u64::<LittleEndian>(block_count as u64).unwrap();

        let mut flags = if trailing_newline { FLAG_TRAILING_NEWLINE } else { 0 };
        if dictionary.is_some() {
            flags |= FLAG_HAS_DICTIONARY;
        }
        if self.options.use_trigrams {
            flags |= FLAG_HAS_TRIGRAMS;
        }
        if fast_mode {
            flags |= FLAG_FAST_MODE;
        }
        if has_jl_sketch {
            flags |= FLAG_HAS_JL_SKETCH;
        }
        output.write_u64::<LittleEndian>(flags).unwrap();
        output.write_i32::<LittleEndian>(self.options.compression_level).unwrap();
        output.write_u64::<LittleEndian>(dict_size as u64).unwrap();
        output.write_u64::<LittleEndian>(orig_dict_size as u64).unwrap();

        // Bytes 60-65: sketch params (in header padding area)
        if has_jl_sketch {
            output.push(self.options.sketch_dim as u8);
            output.push(self.options.sketch_bits);
            output.extend_from_slice(&0u32.to_le_bytes()); // no IDF table
        }
        while output.len() < HEADER_SIZE {
            output.push(0);
        }

        // Dictionary
        if let Some(ref dict) = dict_compressed {
            output.extend_from_slice(dict);
        }

        // Block index
        for i in 0..block_count {
            output.write_u64::<LittleEndian>(block_offsets[i]).unwrap();
            output.write_u64::<LittleEndian>(block_lengths[i]).unwrap();
            output.write_u64::<LittleEndian>(decompressed_sizes[i]).unwrap();
        }

        // Bloom filters
        for (_, bloom, _, _, _) in &blocks {
            for &v in bloom {
                output.write_u64::<LittleEndian>(v).unwrap();
            }
        }

        // Trigram blooms
        if self.options.use_trigrams {
            for (_, _, trigram_bloom, _, _) in &blocks {
                if let Some(ref tb) = trigram_bloom {
                    for &v in tb {
                        output.write_u64::<LittleEndian>(v).unwrap();
                    }
                }
            }
        }

        // JL sketches (Quantized sketches)
        if has_jl_sketch {
            for (_, _, _, sketch_bytes, _) in &blocks {
                if let Some(ref sb) = sketch_bytes {
                    output.extend_from_slice(sb);
                }
            }
        }

        // Compressed block data
        for (data, _, _, _, _) in blocks {
            output.extend_from_slice(&data);
        }

        output
    }

    pub fn write_final_file_with_trailing(
        &self,
        blocks: Vec<BlockResult>,
        row_count: u64,
        dictionary: Option<Vec<u8>>,
        fast_mode: bool,
        trailing_newline: bool,
    ) -> Vec<u8> {
        self.write_final_file_inner(blocks, row_count, dictionary, fast_mode, trailing_newline)
    }
}

impl Default for ParallelBurnerV10 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compress_ultra_fast(data: &[u8], options: CompressionOptions) -> Vec<u8> {
    if data.is_empty() {
        let burner = ParallelBurnerV10::with_options(options);
        return burner.compress_parallel(&[]);
    }

    let trailing_newline = data.last() == Some(&b'\n');
    let content = if trailing_newline { &data[..data.len() - 1] } else { data };

    let num_threads = rayon::current_num_threads();
    let target_chunk_size = (content.len() / num_threads).max(2 * 1024 * 1024);

    let mut chunk_boundaries = vec![0usize];
    let mut pos = target_chunk_size;
    while pos < content.len() {
        if let Some(nl_offset) = content[pos..].iter().position(|&b| b == b'\n') {
            chunk_boundaries.push(pos + nl_offset + 1);
            pos = pos + nl_offset + 1 + target_chunk_size;
        } else {
            break;
        }
    }
    chunk_boundaries.push(content.len());

    let block_size = options.block_size as usize;
    let use_trigrams = options.use_trigrams;
    let level = options.compression_level;
    let fast_mode = options.fast_mode;
    let total_lines = AtomicU64::new(0);

    let chunk_results: Vec<Vec<BlockResult>> =
        chunk_boundaries.windows(2)
            .collect::<Vec<_>>()
            .par_iter()
            .map(|bounds| {
                let start = bounds[0];
                let end = bounds[1];
                let chunk_data = &content[start..end];

                let mut blocks: Vec<BlockResult> = Vec::new();
                let mut current_block = BlockData::with_fast_mode(use_trigrams, fast_mode);
                let mut parse_buf = ParseBuffer::new();
                let mut line_count = 0u64;

                let mut line_start = 0;
                for (i, &byte) in chunk_data.iter().enumerate() {
                    if byte == b'\n' {
                        let line = &chunk_data[line_start..i];
                        current_block.add_line(line, &mut parse_buf);
                        line_count += 1;
                        line_start = i + 1;

                        if current_block.row_count() >= block_size {
                            let raw = if fast_mode {
                                current_block.serialize_fast()
                            } else {
                                current_block.serialize()
                            };
                            let decompressed_size = raw.len();
                            let compressed = compress_no_dict(&raw, level);

                            let sketch_bytes: Option<Vec<u8>> = None;

                            blocks.push((compressed, current_block.bloom, current_block.trigram_bloom, sketch_bytes, decompressed_size));
                            current_block = BlockData::with_fast_mode(use_trigrams, fast_mode);
                        }
                    }
                }

                if line_start < chunk_data.len() {
                    let line = &chunk_data[line_start..];
                    current_block.add_line(line, &mut parse_buf);
                    line_count += 1;
                }

                if current_block.row_count() > 0 {
                    let raw = if fast_mode {
                        current_block.serialize_fast()
                    } else {
                        current_block.serialize()
                    };
                    let decompressed_size = raw.len();
                    let compressed = compress_no_dict(&raw, level);

                    let sketch_bytes: Option<Vec<u8>> = None;

                    blocks.push((compressed, current_block.bloom, current_block.trigram_bloom, sketch_bytes, decompressed_size));
                }

                total_lines.fetch_add(line_count, Ordering::Relaxed);
                blocks
            })
            .collect();

    let all_blocks: Vec<BlockResult> = chunk_results.into_iter().flatten().collect();
    let row_count = total_lines.load(Ordering::Relaxed);

    let mut fast_options = options.clone();
    fast_options.use_dictionary = false;
    let burner = ParallelBurnerV10::with_options(fast_options);
    burner.write_final_file_with_trailing(all_blocks, row_count, None, fast_mode, trailing_newline)
}
