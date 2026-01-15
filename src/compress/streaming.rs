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
use crate::hash::{
    BloomFilter, TrigramBloom,
    bloom_empty, bloom_add_line,
    trigram_bloom_empty, trigram_bloom_add_line,
};

pub struct StreamingBurnerV10 {
    options: CompressionOptions,
    all_lines: Vec<u8>,
    block_line_counts: Vec<u64>,
    block_blooms: Vec<BloomFilter>,
    trigram_blooms: Vec<TrigramBloom>,
    current_bloom: BloomFilter,
    current_trigram: Option<TrigramBloom>,
    current_block_lines: u64,
    total_lines: u64,
    trailing_newline: bool,
}

impl StreamingBurnerV10 {
    pub fn new() -> Self {
        Self::with_options(CompressionOptions::default())
    }

    pub fn with_options(options: CompressionOptions) -> Self {
        Self {
            current_bloom: bloom_empty(),
            current_trigram: if options.use_trigrams { Some(trigram_bloom_empty()) } else { None },
            all_lines: Vec::with_capacity(1024 * 1024), // 1MB initial
            block_line_counts: Vec::new(),
            block_blooms: Vec::new(),
            trigram_blooms: Vec::new(),
            current_block_lines: 0,
            total_lines: 0,
            trailing_newline: true,
            options,
        }
    }

    pub fn set_trailing_newline(&mut self, has_trailing: bool) {
        self.trailing_newline = has_trailing;
    }

    pub fn ingest_line(&mut self, line: &[u8]) {
       
        bloom_add_line(&mut self.current_bloom, line);

        if let Some(ref mut tb) = self.current_trigram {
            trigram_bloom_add_line(tb, line);
        }

       
        self.all_lines.extend_from_slice(line);
        self.all_lines.push(b'\n');

        self.current_block_lines += 1;
        self.total_lines += 1;

       
        if self.current_block_lines >= self.options.block_size {
            self.flush_block();
        }
    }

    fn flush_block(&mut self) {
        if self.current_block_lines == 0 {
            return;
        }

       
        self.block_blooms.push(std::mem::replace(&mut self.current_bloom, bloom_empty()));

        if let Some(tb) = std::mem::take(&mut self.current_trigram) {
            self.trigram_blooms.push(tb);
            self.current_trigram = Some(trigram_bloom_empty());
        }

       
        self.block_line_counts.push(self.current_block_lines);
        self.current_block_lines = 0;
    }

    pub fn serialize(&mut self) -> Vec<u8> {
       
        self.flush_block();

        if self.total_lines == 0 {
            return self.write_empty();
        }

        let block_count = self.block_blooms.len();

       
        let compressed_data = zstd::stream::encode_all(
            &self.all_lines[..],
            self.options.compression_level
        ).unwrap_or_else(|_| self.all_lines.clone());

       
       
        let mut line_offsets: Vec<(u64, u64, u64)> = Vec::with_capacity(block_count);
        let mut current_line: u64 = 0;
        let mut current_offset: u64 = 0;

       
        let mut line_byte_offsets: Vec<u64> = vec![0];
        for (i, &b) in self.all_lines.iter().enumerate() {
            if b == b'\n' {
                line_byte_offsets.push((i + 1) as u64);
            }
        }

        for &line_count in &self.block_line_counts {
            let start_offset = if current_line == 0 { 0 } else {
                line_byte_offsets.get(current_line as usize).copied().unwrap_or(0)
            };
            line_offsets.push((current_line, line_count, start_offset));
            current_line += line_count;
        }

       
       
        let index_size = block_count * 24;
        let bloom_size = block_count * BLOOM_SIZE_BYTES;
        let trigram_size = if self.options.use_trigrams {
            block_count * TRIGRAM_BLOOM_SIZE_BYTES
        } else {
            0
        };

       
        let decompressed_size = self.all_lines.len() as u64;
        let total_size = HEADER_SIZE + index_size + bloom_size + trigram_size + 16 + compressed_data.len();
        let mut output = Vec::with_capacity(total_size);

       
        output.extend_from_slice(MAGIC);
        output.write_u32::<LittleEndian>(VERSION).unwrap();
        output.write_u64::<LittleEndian>(self.options.block_size).unwrap();
        output.write_u64::<LittleEndian>(self.total_lines).unwrap();
        output.write_u64::<LittleEndian>(block_count as u64).unwrap();

        let mut flags = if self.trailing_newline { FLAG_TRAILING_NEWLINE } else { 0 };
        flags |= FLAG_FAST_MODE;     
        flags |= FLAG_STREAMING_MODE;
        if self.options.use_trigrams {
            flags |= FLAG_HAS_TRIGRAMS;
        }
        output.write_u64::<LittleEndian>(flags).unwrap();

        output.write_i32::<LittleEndian>(self.options.compression_level).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();

       
        while output.len() < HEADER_SIZE {
            output.push(0);
        }

       
        for (start_line, line_count, byte_offset) in &line_offsets {
            output.write_u64::<LittleEndian>(*start_line).unwrap();
            output.write_u64::<LittleEndian>(*line_count).unwrap();
            output.write_u64::<LittleEndian>(*byte_offset).unwrap();
        }

       
        for bloom in &self.block_blooms {
            for &v in bloom {
                output.write_u64::<LittleEndian>(v).unwrap();
            }
        }

       
        if self.options.use_trigrams {
            for tb in &self.trigram_blooms {
                for &v in tb {
                    output.write_u64::<LittleEndian>(v).unwrap();
                }
            }
        }

       
        output.write_u64::<LittleEndian>(decompressed_size).unwrap();
        output.write_u64::<LittleEndian>(compressed_data.len() as u64).unwrap();
        output.extend_from_slice(&compressed_data);

        output
    }

    fn write_empty(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(HEADER_SIZE);
        output.extend_from_slice(MAGIC);
        output.write_u32::<LittleEndian>(VERSION).unwrap();
        output.write_u64::<LittleEndian>(self.options.block_size).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        let flags = FLAG_FAST_MODE | FLAG_STREAMING_MODE;
        output.write_u64::<LittleEndian>(flags).unwrap();
        output.write_i32::<LittleEndian>(self.options.compression_level).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        while output.len() < HEADER_SIZE {
            output.push(0);
        }
        output
    }
}

impl Default for StreamingBurnerV10 {
    fn default() -> Self {
        Self::new()
    }
}
