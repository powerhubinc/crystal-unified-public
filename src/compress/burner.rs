//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use byteorder::{LittleEndian, WriteBytesExt};
use crate::constants::*;
use crate::options::CompressionOptions;
use crate::hash::{BloomFilter, TrigramBloom};
use crate::block::{BlockData, ParseBuffer};
use crate::dictionary::{train_dictionary, ReusableCCtx};
use super::helpers::compress_no_dict;

pub struct BurnerV10 {
    pub options: CompressionOptions,
    current_block: BlockData,
    parse_buf: ParseBuffer,
    compressed_blocks: Vec<Vec<u8>>,
    decompressed_sizes: Vec<u64>,
    block_blooms: Vec<BloomFilter>,
    trigram_blooms: Vec<TrigramBloom>,
    jl_sketches: Vec<Vec<u8>>,
    row_count: u64,
    trailing_newline: bool,
    dictionary: Option<Vec<u8>>,
    cctx: Option<ReusableCCtx>,
    sample_buffer: Vec<Vec<u8>>,
    sample_bytes_collected: usize,
    dictionary_trained: bool,
}

impl BurnerV10 {
    pub fn new() -> Self {
        Self::with_options(CompressionOptions::default())
    }

    pub fn with_options(options: CompressionOptions) -> Self {
        Self {
            current_block: BlockData::with_fast_mode(options.use_trigrams, options.fast_mode),
            parse_buf: ParseBuffer::new(),
            compressed_blocks: Vec::new(),
            decompressed_sizes: Vec::new(),
            block_blooms: Vec::new(),
            trigram_blooms: Vec::new(),
            jl_sketches: Vec::new(),
            row_count: 0,
            trailing_newline: true,
            dictionary: None,
            cctx: None,
            sample_buffer: Vec::new(),
            sample_bytes_collected: 0,
            dictionary_trained: !options.use_dictionary,
            options,
        }
    }

    pub fn set_trailing_newline(&mut self, has_trailing: bool) {
        self.trailing_newline = has_trailing;
    }

    pub fn ingest_line(&mut self, line: &[u8]) {
        self.current_block.add_line(line, &mut self.parse_buf);
        self.row_count += 1;

        if self.current_block.row_count() as u64 >= self.options.block_size {
            self.flush_block();
        }
    }

    fn flush_block(&mut self) {
        if self.current_block.row_count() == 0 {
            return;
        }

       
        let raw_data = if self.options.fast_mode {
            self.current_block.serialize_fast()
        } else {
            self.current_block.serialize()
        };
        let decompressed_size = raw_data.len() as u64;

       
        if self.options.use_dictionary && !self.dictionary_trained {
            if self.sample_bytes_collected < self.options.dictionary_sample_bytes {
                self.sample_buffer.push(raw_data.clone());
                self.sample_bytes_collected += raw_data.len();
            }

            if self.sample_bytes_collected >= self.options.dictionary_sample_bytes {
                self.train_and_init_dictionary();
            }
        }

       
        let compressed = if let Some(ref mut cctx) = self.cctx {
            cctx.compress(&raw_data)
        } else {
            compress_no_dict(&raw_data, self.options.compression_level)
        };

        self.compressed_blocks.push(compressed);
        self.decompressed_sizes.push(decompressed_size);
        self.block_blooms.push(std::mem::take(&mut self.current_block.bloom));

        if self.options.use_trigrams {
            if let Some(tb) = std::mem::take(&mut self.current_block.trigram_bloom) {
                self.trigram_blooms.push(tb);
            }
        }

        self.current_block.clear(self.options.use_trigrams);
    }

    fn train_and_init_dictionary(&mut self) {
        if self.dictionary_trained {
            return;
        }

        if let Some(dict) = train_dictionary(&self.sample_buffer, self.options.dictionary_size) {
            self.dictionary = Some(dict.clone());
            self.cctx = Some(
                ReusableCCtx::new(self.options.compression_level)
                    .with_dictionary(&dict, self.options.compression_level)
            );
        } else {
            self.cctx = Some(ReusableCCtx::new(self.options.compression_level));
        }

        self.dictionary_trained = true;
        self.sample_buffer.clear();
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        self.flush_block();

        if self.options.use_dictionary && !self.dictionary_trained && !self.sample_buffer.is_empty() {
            self.train_and_init_dictionary();
        }

        let block_count = self.compressed_blocks.len();

        let dict_compressed = self.dictionary.as_ref().map(|d| {
            zstd::stream::encode_all(&d[..], 3).unwrap_or_else(|_| d.clone())
        });
        let dict_size = dict_compressed.as_ref().map(|d| d.len()).unwrap_or(0);

        let mut block_offsets = Vec::with_capacity(block_count);
        let mut block_lengths = Vec::with_capacity(block_count);
        let mut current_offset = 0u64;

        for block in &self.compressed_blocks {
            block_offsets.push(current_offset);
            block_lengths.push(block.len() as u64);
            current_offset += block.len() as u64;
        }

        let index_size = block_count * BLOCK_INDEX_ENTRY_SIZE;
        let bloom_size = block_count * BLOOM_SIZE_BYTES;
        let trigram_size = if self.options.use_trigrams {
            block_count * TRIGRAM_BLOOM_SIZE_BYTES
        } else {
            0
        };
        let entry_size = jl_sketch_entry_size(self.options.sketch_dim, self.options.sketch_bits);
        let sketch_size = if self.options.use_jl_sketch && !self.jl_sketches.is_empty() {
            block_count * entry_size
        } else {
            0
        };
        let data_size: usize = self.compressed_blocks.iter().map(|b| b.len()).sum();

        let total_size = HEADER_SIZE + 8 + dict_size + index_size + bloom_size + trigram_size + sketch_size + data_size;
        let mut output = Vec::with_capacity(total_size);

       
        output.extend_from_slice(MAGIC);
        output.write_u32::<LittleEndian>(VERSION).unwrap();
        output.write_u64::<LittleEndian>(self.options.block_size).unwrap();
        output.write_u64::<LittleEndian>(self.row_count).unwrap();
        output.write_u64::<LittleEndian>(block_count as u64).unwrap();

        let mut flags = if self.trailing_newline { FLAG_TRAILING_NEWLINE } else { 0 };
        if self.dictionary.is_some() {
            flags |= FLAG_HAS_DICTIONARY;
        }
        if self.options.use_trigrams {
            flags |= FLAG_HAS_TRIGRAMS;
        }
        if self.options.fast_mode {
            flags |= FLAG_FAST_MODE;
        }
        if self.options.use_jl_sketch && !self.jl_sketches.is_empty() {
            flags |= FLAG_HAS_JL_SKETCH;
        }
        output.write_u64::<LittleEndian>(flags).unwrap();

        output.write_i32::<LittleEndian>(self.options.compression_level).unwrap();
        output.write_u64::<LittleEndian>(dict_size as u64).unwrap();

        let orig_dict_size = self.dictionary.as_ref().map(|d| d.len()).unwrap_or(0);
        output.write_u64::<LittleEndian>(orig_dict_size as u64).unwrap();

        if self.options.use_jl_sketch && !self.jl_sketches.is_empty() {
            output.push(self.options.sketch_dim as u8);
            output.push(self.options.sketch_bits);
            output.extend_from_slice(&0u32.to_le_bytes()); // no IDF table
        }
        while output.len() < HEADER_SIZE {
            output.push(0);
        }

        if let Some(ref dict) = dict_compressed {
            output.extend_from_slice(dict);
        }

       
        for i in 0..block_count {
            output.write_u64::<LittleEndian>(block_offsets[i]).unwrap();
            output.write_u64::<LittleEndian>(block_lengths[i]).unwrap();
            output.write_u64::<LittleEndian>(self.decompressed_sizes[i]).unwrap();
        }

       
        for bloom in &self.block_blooms {
            for &v in bloom {
                output.write_u64::<LittleEndian>(v).unwrap();
            }
        }

       
        if self.options.use_trigrams {
            for trigram_bloom in &self.trigram_blooms {
                for &v in trigram_bloom {
                    output.write_u64::<LittleEndian>(v).unwrap();
                }
            }
        }

        if self.options.use_jl_sketch {
            for sketch_bytes in &self.jl_sketches {
                output.extend_from_slice(sketch_bytes);
            }
        }

        for block in &self.compressed_blocks {
            output.extend_from_slice(block);
        }

        output
    }
}

impl Default for BurnerV10 {
    fn default() -> Self {
        Self::new()
    }
}
