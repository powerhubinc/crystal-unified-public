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
use zstd::zstd_safe::DDict;

use crate::constants::*;
use crate::error::{Result, CrystalError};
use crate::hash::{
    BloomFilter, TrigramBloom,
    bloom_empty, bloom_query, bloom_might_contain,
    trigram_bloom_empty, trigram_query_pattern, trigram_might_contain,
};
use crate::encoding::{contains_bytes, group_varint_decode, read_varint_fast};
use crate::sketch::{
    QuantizedVector, VectorQuantizer, SimilarityMatch,
    jl_sketch_from_embedding,
};
use super::helpers::{decompress_no_dict, decompress_with_ddict};

struct TemplateV10<'a> {
    parts: Vec<&'a [u8]>,
}

#[allow(dead_code)]
pub struct CrystalReaderV10<'a> {
    data: &'a [u8],
    block_size: u64,
    row_count: u64,
    block_count: u64,
    flags: u64,
    compression_level: i32,
    dict_offset: usize,
    dict_size: usize,
    orig_dict_size: usize,
    index_offset: usize,
    bloom_offset: usize,
    trigram_offset: usize,
    sketch_offset: usize,
    data_offset: usize,
    ddict: Option<DDict<'static>>,
    fast_mode: bool,
    streaming_mode: bool,
    has_jl_sketch: bool,
    sketch_dim: usize,
    sketch_bits: u8,
    sketch_entry_size: usize,
}

impl<'a> CrystalReaderV10<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(CrystalError::DecompressionFailed("Data too short".into()));
        }
        if &data[0..4] != MAGIC {
            return Err(CrystalError::InvalidMagic {
                expected: MAGIC.to_vec(),
                got: data[0..4].to_vec(),
            });
        }

        let mut cursor = Cursor::new(&data[4..]);
        let version = cursor.read_u32::<LittleEndian>()?;
        if version != VERSION {
            return Err(CrystalError::DecompressionFailed(
                format!("Expected v{}, got v{}", VERSION, version)
            ));
        }

        let block_size = cursor.read_u64::<LittleEndian>()?;
        let row_count = cursor.read_u64::<LittleEndian>()?;
        let block_count = cursor.read_u64::<LittleEndian>()?;
        let flags = cursor.read_u64::<LittleEndian>()?;
        let compression_level = cursor.read_i32::<LittleEndian>()?;
        let dict_size = cursor.read_u64::<LittleEndian>()? as usize;
        let orig_dict_size = cursor.read_u64::<LittleEndian>()? as usize;

        let has_dictionary = (flags & FLAG_HAS_DICTIONARY) != 0;
        let has_trigrams = (flags & FLAG_HAS_TRIGRAMS) != 0;
        let fast_mode = (flags & FLAG_FAST_MODE) != 0;
        let streaming_mode = (flags & FLAG_STREAMING_MODE) != 0;
        let has_jl_sketch = (flags & FLAG_HAS_JL_SKETCH) != 0;

        let (sketch_dim, sketch_bits, idf_section_size) = if has_jl_sketch {
            let dim = data[HEADER_SKETCH_DIM_OFFSET] as usize;
            let bits = data[HEADER_SKETCH_BITS_OFFSET];
            let dim = if dim == 0 { JL_SKETCH_DEFAULT_DIM } else { dim as usize };
            let bits = if bits == 0 { JL_SKETCH_QUANT_BITS } else { bits };
            let idf_size = if HEADER_IDF_SIZE_OFFSET + 4 <= HEADER_SIZE {
                u32::from_le_bytes(
                    data[HEADER_IDF_SIZE_OFFSET..HEADER_IDF_SIZE_OFFSET + 4]
                        .try_into()
                        .unwrap_or([0; 4]),
                ) as usize
            } else {
                0
            };
            (dim, bits, idf_size)
        } else {
            (JL_SKETCH_DEFAULT_DIM, JL_SKETCH_QUANT_BITS, 0)
        };
        let sketch_entry_size = jl_sketch_entry_size(sketch_dim, sketch_bits);

        let dict_offset = HEADER_SIZE;
        let index_offset = dict_offset + dict_size;
        let bloom_offset = index_offset + (block_count as usize * BLOCK_INDEX_ENTRY_SIZE);
        let trigram_offset = bloom_offset + (block_count as usize * BLOOM_SIZE_BYTES);
        let sketch_offset = if has_trigrams {
            trigram_offset + (block_count as usize * TRIGRAM_BLOOM_SIZE_BYTES)
        } else {
            trigram_offset
        };
        let data_offset = if has_jl_sketch {
            sketch_offset + (block_count as usize * sketch_entry_size) + idf_section_size
        } else {
            sketch_offset
        };

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

        Ok(Self {
            data,
            block_size,
            row_count,
            block_count,
            flags,
            compression_level,
            dict_offset,
            dict_size,
            orig_dict_size,
            index_offset,
            bloom_offset,
            trigram_offset,
            sketch_offset,
            data_offset,
            ddict,
            fast_mode,
            streaming_mode,
            has_jl_sketch,
            sketch_dim,
            sketch_bits,
            sketch_entry_size,
        })
    }

    pub fn block_count(&self) -> u64 { self.block_count }
    pub fn row_count(&self) -> u64 { self.row_count }
    pub fn block_size(&self) -> u64 { self.block_size }
    pub fn has_dictionary(&self) -> bool { self.ddict.is_some() }
    pub fn has_trigrams(&self) -> bool { (self.flags & FLAG_HAS_TRIGRAMS) != 0 }
    pub fn has_trailing_newline(&self) -> bool { (self.flags & FLAG_TRAILING_NEWLINE) != 0 }
    pub fn is_fast_mode(&self) -> bool { self.fast_mode }
    pub fn compression_level(&self) -> i32 { self.compression_level }
    pub fn is_streaming_mode(&self) -> bool { self.streaming_mode }
    pub fn flags(&self) -> u64 { self.flags }
    pub fn has_jl_sketch(&self) -> bool { self.has_jl_sketch }

    pub fn get_raw_block(&self, block_idx: u64) -> Option<(&[u8], usize)> {
        if block_idx >= self.block_count {
            return None;
        }
        let (start, compressed_len, decompressed_size) = self.get_block_range(block_idx);
        if start + compressed_len > self.data.len() {
            return None;
        }
        Some((&self.data[start..start + compressed_len], decompressed_size))
    }

    pub fn get_block_bloom(&self, block_idx: u64) -> BloomFilter {
        self.get_bloom(block_idx)
    }

    pub fn get_block_trigram(&self, block_idx: u64) -> Option<TrigramBloom> {
        self.get_trigram_bloom(block_idx)
    }

    fn get_block_range(&self, block_idx: u64) -> (usize, usize, usize) {
        let idx = block_idx as usize;
        let pos = self.index_offset + idx * BLOCK_INDEX_ENTRY_SIZE;
        let offset = u64::from_le_bytes(self.data[pos..pos + 8].try_into().unwrap()) as usize;
        let compressed_len = u64::from_le_bytes(self.data[pos + 8..pos + 16].try_into().unwrap()) as usize;
        let decompressed_size = u64::from_le_bytes(self.data[pos + 16..pos + 24].try_into().unwrap()) as usize;
        (self.data_offset + offset, compressed_len, decompressed_size)
    }

    fn get_streaming_line_range(&self, block_idx: u64) -> (u64, u64, u64) {
        let idx = block_idx as usize;
        let pos = self.index_offset + idx * BLOCK_INDEX_ENTRY_SIZE;
        let start_line = u64::from_le_bytes(self.data[pos..pos + 8].try_into().unwrap());
        let line_count = u64::from_le_bytes(self.data[pos + 8..pos + 16].try_into().unwrap());
        let byte_offset = u64::from_le_bytes(self.data[pos + 16..pos + 24].try_into().unwrap());
        (start_line, line_count, byte_offset)
    }

    fn get_bloom(&self, block_idx: u64) -> BloomFilter {
        let pos = self.bloom_offset + (block_idx as usize * BLOOM_SIZE_BYTES);
        let mut bloom: BloomFilter = bloom_empty();
        for i in 0..BLOOM_SIZE_U64 {
            bloom[i] = u64::from_le_bytes(self.data[pos + i * 8..pos + (i + 1) * 8].try_into().unwrap());
        }
        bloom
    }

    fn get_trigram_bloom(&self, block_idx: u64) -> Option<TrigramBloom> {
        if !self.has_trigrams() {
            return None;
        }
        let pos = self.trigram_offset + (block_idx as usize * TRIGRAM_BLOOM_SIZE_BYTES);
        let mut bloom: TrigramBloom = trigram_bloom_empty();
        for i in 0..TRIGRAM_BLOOM_SIZE_U64 {
            bloom[i] = u64::from_le_bytes(self.data[pos + i * 8..pos + (i + 1) * 8].try_into().unwrap());
        }
        Some(bloom)
    }

    pub fn filter_candidates(&self, term: &[u8]) -> Vec<u64> {
        let term_bloom = bloom_query(term);
        let mut candidates = Vec::new();

        for block_idx in 0..self.block_count {
            let block_bloom = self.get_bloom(block_idx);
            if bloom_might_contain(&block_bloom, &term_bloom) {
                candidates.push(block_idx);
            }
        }

        candidates
    }

    pub fn filter_candidates_trigram(&self, pattern: &[u8]) -> Vec<u64> {
        if !self.has_trigrams() {
            // Without trigrams, word bloom filters only work for exact whole-word matches.
            // They cannot reliably find substrings due to how words are hashed.
            // For accuracy, search all blocks when trigrams are unavailable.
            return (0..self.block_count).collect();
        }

        let query_trigram = trigram_query_pattern(pattern);
        let mut candidates = Vec::new();

        for block_idx in 0..self.block_count {
            if let Some(block_trigram) = self.get_trigram_bloom(block_idx) {
                if trigram_might_contain(&block_trigram, &query_trigram) {
                    candidates.push(block_idx);
                }
            }
        }

        candidates
    }

    fn decompress_block(&self, block_idx: u64) -> Result<Vec<Vec<u8>>> {
        let (start, compressed_len, decompressed_size) = self.get_block_range(block_idx);
        if start + compressed_len > self.data.len() {
            return Err(CrystalError::DecompressionFailed("Block out of bounds".into()));
        }

        let compressed = &self.data[start..start + compressed_len];

       
        let decompressed = if let Some(ref ddict) = self.ddict {
            decompress_with_ddict(compressed, ddict, decompressed_size)?
        } else {
            decompress_no_dict(compressed, decompressed_size)?
        };

       
        if self.fast_mode {
            return Ok(decompressed
                .split(|&b| b == b'\n')
                .filter(|line| !line.is_empty())
                .map(|line| line.to_vec())
                .collect());
        }

        self.parse_block_rows(&decompressed)
    }

    fn parse_block_rows(&self, block_data: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut cursor = Cursor::new(block_data);

        let template_count = cursor.read_u32::<LittleEndian>()? as usize;
        let mut templates = Vec::with_capacity(template_count);
        let mut pos = 4usize;

        for _ in 0..template_count {
            if pos + 2 > block_data.len() { break; }
            let parts_count = u16::from_le_bytes(block_data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            let mut parts = Vec::with_capacity(parts_count);
            for _ in 0..parts_count {
                if pos + 2 > block_data.len() { break; }
                let p_len = u16::from_le_bytes(block_data[pos..pos + 2].try_into().unwrap()) as usize;
                pos += 2;
                if pos + p_len > block_data.len() { break; }
                parts.push(&block_data[pos..pos + p_len]);
                pos += p_len;
            }
            templates.push(TemplateV10 { parts });
        }

        if pos + 4 > block_data.len() {
            return Ok(Vec::new());
        }
        let row_count = u32::from_le_bytes(block_data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        let mut row_lengths = Vec::with_capacity(row_count);
        let full_groups = row_count / 4;
        let remainder = row_count % 4;

        for _ in 0..full_groups {
            if pos >= block_data.len() { break; }
            let control = block_data[pos];
            pos += 1;
            let (vals, bytes_read) = group_varint_decode(control, &block_data[pos..]);
            row_lengths.extend(vals.iter().map(|&v| v as usize));
            pos += bytes_read;
        }

        for _ in 0..remainder {
            if pos >= block_data.len() { break; }
            let len = read_varint_fast(block_data, &mut pos) as usize;
            row_lengths.push(len);
        }

        let mut rows = Vec::with_capacity(row_count);
        let mut recon_buf = Vec::with_capacity(512);

        for &row_len in &row_lengths {
            if pos + row_len > block_data.len() { break; }
            let row_data = &block_data[pos..pos + row_len];
            pos += row_len;

            if row_data.len() < 4 { continue; }
            let tmpl_id = u16::from_le_bytes(row_data[0..2].try_into().unwrap()) as usize;
            let var_count = u16::from_le_bytes(row_data[2..4].try_into().unwrap()) as usize;

            if tmpl_id == LITERAL_TEMPLATE_ID as usize {
                let mut rpos = 4usize;
                let line_len = read_varint_fast(row_data, &mut rpos) as usize;
                if rpos + line_len <= row_data.len() {
                    rows.push(row_data[rpos..rpos + line_len].to_vec());
                }
            } else if let Some(template) = templates.get(tmpl_id) {
                recon_buf.clear();
                let mut rpos = 4usize;
                let mut var_idx = 0;

                for part in &template.parts {
                    if part.is_empty() && var_idx < var_count {
                        let v_len = read_varint_fast(row_data, &mut rpos) as usize;
                        if rpos + v_len <= row_data.len() {
                            recon_buf.extend_from_slice(&row_data[rpos..rpos + v_len]);
                            rpos += v_len;
                        }
                        var_idx += 1;
                    } else {
                        recon_buf.extend_from_slice(part);
                    }
                }

                rows.push(recon_buf.clone());
            }
        }

        Ok(rows)
    }

    pub fn count_matches(&self, term: &[u8]) -> usize {
       
        let candidates = if term.len() >= 3 {
            self.filter_candidates_trigram(term)
        } else {
            self.filter_candidates(term)
        };

        if candidates.is_empty() {
            return 0;
        }

        candidates
            .par_iter()
            .map(|&block_idx| {
                match self.decompress_block(block_idx) {
                    Ok(rows) => {
                        rows.iter()
                            .filter(|row| contains_bytes(row, term))
                            .count()
                    }
                    Err(_) => 0,
                }
            })
            .sum()
    }

    pub fn search_fast(&self, term: &[u8]) -> Vec<Vec<u8>> {
       
        let candidates = if term.len() >= 3 {
            self.filter_candidates_trigram(term)
        } else {
            self.filter_candidates(term)
        };

        if candidates.is_empty() {
            return Vec::new();
        }

       
        if self.streaming_mode {
            return self.search_streaming(term, &candidates)
                .into_iter()
                .map(|s| s.into_bytes())
                .collect();
        }

        let results: Vec<Vec<Vec<u8>>> = candidates
            .par_iter()
            .filter_map(|&block_idx| {
                match self.decompress_block(block_idx) {
                    Ok(rows) => {
                        let matches: Vec<Vec<u8>> = rows
                            .into_iter()
                            .filter(|row| contains_bytes(row, term))
                            .collect();
                        if matches.is_empty() { None } else { Some(matches) }
                    }
                    Err(_) => None,
                }
            })
            .collect();

        results.into_iter().flatten().collect()
    }

    pub fn search_parallel(&self, term: &[u8]) -> Vec<String> {
       
        let candidates = if term.len() >= 3 {
            self.filter_candidates_trigram(term)
        } else {
            self.filter_candidates(term)
        };

        if candidates.is_empty() {
            return Vec::new();
        }

       
        if self.streaming_mode {
            return self.search_streaming(term, &candidates);
        }

        let results: Vec<Vec<String>> = candidates
            .par_iter()
            .filter_map(|&block_idx| {
                match self.decompress_block(block_idx) {
                    Ok(rows) => {
                        let matches: Vec<String> = rows
                            .into_iter()
                            .filter_map(|row| {
                                if contains_bytes(&row, term) {
                                    Some(String::from_utf8_lossy(&row).into_owned())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if matches.is_empty() { None } else { Some(matches) }
                    }
                    Err(_) => None,
                }
            })
            .collect();

        results.into_iter().flatten().collect()
    }

    fn search_streaming(&self, term: &[u8], candidates: &[u64]) -> Vec<String> {
       
        let decompressed = self.reconstruct_streaming();
        if decompressed.is_empty() {
            return Vec::new();
        }

       
        let mut line_starts: Vec<usize> = vec![0];
        for (i, &b) in decompressed.iter().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }

       
        let mut results = Vec::new();
        for &block_idx in candidates {
            let (start_line, line_count, _byte_offset) = self.get_streaming_line_range(block_idx);
            let start = start_line as usize;
            let end = (start_line + line_count) as usize;

            for line_idx in start..end.min(line_starts.len().saturating_sub(1)) {
                let line_start = line_starts[line_idx];
                let line_end = if line_idx + 1 < line_starts.len() {
                    line_starts[line_idx + 1].saturating_sub(1)
                } else {
                    decompressed.len()
                };

                let line = &decompressed[line_start..line_end];
                if contains_bytes(line, term) {
                    results.push(String::from_utf8_lossy(line).into_owned());
                }
            }
        }

        results
    }

    pub fn search_trigram(&self, pattern: &[u8]) -> Vec<String> {
        let candidates = self.filter_candidates_trigram(pattern);

        if candidates.is_empty() {
            return Vec::new();
        }

       
        if self.streaming_mode {
            return self.search_streaming(pattern, &candidates);
        }

        let results: Vec<Vec<String>> = candidates
            .par_iter()
            .filter_map(|&block_idx| {
                match self.decompress_block(block_idx) {
                    Ok(rows) => {
                        let matches: Vec<String> = rows
                            .into_iter()
                            .filter_map(|row| {
                                if contains_bytes(&row, pattern) {
                                    Some(String::from_utf8_lossy(&row).into_owned())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if matches.is_empty() { None } else { Some(matches) }
                    }
                    Err(_) => None,
                }
            })
            .collect();

        results.into_iter().flatten().collect()
    }

    pub fn reconstruct(&self) -> Vec<u8> {
        if self.block_count == 0 {
            return Vec::new();
        }

       
        if self.streaming_mode {
            return self.reconstruct_streaming();
        }

       
        let block_indices: Vec<u64> = (0..self.block_count).collect();
        let block_bytes: Vec<Vec<u8>> = block_indices
            .par_iter()
            .map(|&block_idx| {
                self.decompress_block_bytes(block_idx).unwrap_or_default()
            })
            .collect();

       
        let total_size: usize = block_bytes.iter().map(|b| b.len()).sum();
        let mut output = Vec::with_capacity(total_size);

        for bytes in block_bytes {
            output.extend_from_slice(&bytes);
        }

        let has_trailing = (self.flags & FLAG_TRAILING_NEWLINE) != 0;
        if !has_trailing && output.last() == Some(&b'\n') {
            output.pop();
        }

        output
    }

    fn reconstruct_streaming(&self) -> Vec<u8> {
       
        let size_offset = self.data_offset;
        if size_offset + 16 > self.data.len() {
            return Vec::new();
        }

        let decompressed_size = u64::from_le_bytes(
            self.data[size_offset..size_offset + 8].try_into().unwrap()
        ) as usize;

        let compressed_size = u64::from_le_bytes(
            self.data[size_offset + 8..size_offset + 16].try_into().unwrap()
        ) as usize;

        let compressed_start = size_offset + 16;
        if compressed_start + compressed_size > self.data.len() {
            return Vec::new();
        }

        let compressed = &self.data[compressed_start..compressed_start + compressed_size];

       
        let mut output = vec![0u8; decompressed_size];
        match zstd::bulk::decompress_to_buffer(compressed, &mut output) {
            Ok(actual_size) => {
                output.truncate(actual_size);
            }
            Err(_) => {
               
                output = match zstd::stream::decode_all(compressed) {
                    Ok(data) => data,
                    Err(_) => return Vec::new(),
                };
            }
        }

        let has_trailing = (self.flags & FLAG_TRAILING_NEWLINE) != 0;
        if !has_trailing && output.last() == Some(&b'\n') {
            output.pop();
        }

        output
    }

    pub fn decompress_block_bytes(&self, block_idx: u64) -> Result<Vec<u8>> {
        let (start, compressed_len, decompressed_size) = self.get_block_range(block_idx);
        if start + compressed_len > self.data.len() {
            return Err(CrystalError::DecompressionFailed("Block out of bounds".into()));
        }

        let compressed = &self.data[start..start + compressed_len];

        let decompressed = if let Some(ref ddict) = self.ddict {
            decompress_with_ddict(compressed, ddict, decompressed_size)?
        } else {
            decompress_no_dict(compressed, decompressed_size)?
        };

       
        if self.fast_mode {
            return Ok(decompressed);
        }

       
        self.parse_block_to_bytes(&decompressed)
    }

    fn parse_block_to_bytes(&self, block_data: &[u8]) -> Result<Vec<u8>> {
        if block_data.len() < 4 {
            return Ok(Vec::new());
        }

        let template_count = u32::from_le_bytes(block_data[0..4].try_into().unwrap()) as usize;
        let mut pos = 4usize;

       
        let mut templates: Vec<Vec<&[u8]>> = Vec::with_capacity(template_count);
        for _ in 0..template_count {
            if pos + 2 > block_data.len() { break; }
            let parts_count = u16::from_le_bytes(block_data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            let mut parts = Vec::with_capacity(parts_count);
            for _ in 0..parts_count {
                if pos + 2 > block_data.len() { break; }
                let p_len = u16::from_le_bytes(block_data[pos..pos + 2].try_into().unwrap()) as usize;
                pos += 2;
                if pos + p_len > block_data.len() { break; }
                parts.push(&block_data[pos..pos + p_len]);
                pos += p_len;
            }
            templates.push(parts);
        }

        if pos + 4 > block_data.len() {
            return Ok(Vec::new());
        }
        let row_count = u32::from_le_bytes(block_data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

       
        let mut row_lengths = Vec::with_capacity(row_count);
        let full_groups = row_count / 4;
        let remainder = row_count % 4;

        for _ in 0..full_groups {
            if pos >= block_data.len() { break; }
            let control = block_data[pos];
            pos += 1;
            let (vals, bytes_read) = group_varint_decode(control, &block_data[pos..]);
            row_lengths.extend(vals.iter().map(|&v| v as usize));
            pos += bytes_read;
        }

        for _ in 0..remainder {
            if pos >= block_data.len() { break; }
            let len = read_varint_fast(block_data, &mut pos) as usize;
            row_lengths.push(len);
        }

       
        let estimated_size: usize = row_lengths.iter().sum::<usize>() + row_count;
        let mut output = Vec::with_capacity(estimated_size);

       
        for &row_len in &row_lengths {
            if pos + row_len > block_data.len() { break; }
            let row_data = &block_data[pos..pos + row_len];
            pos += row_len;

            if row_data.len() < 4 { continue; }
            let tmpl_id = u16::from_le_bytes(row_data[0..2].try_into().unwrap()) as usize;
            let var_count = u16::from_le_bytes(row_data[2..4].try_into().unwrap()) as usize;

            if tmpl_id == LITERAL_TEMPLATE_ID as usize {
               
                let mut rpos = 4usize;
                let line_len = read_varint_fast(row_data, &mut rpos) as usize;
                if rpos + line_len <= row_data.len() {
                    output.extend_from_slice(&row_data[rpos..rpos + line_len]);
                    output.push(b'\n');
                }
            } else if let Some(parts) = templates.get(tmpl_id) {
               
                let mut rpos = 4usize;
                let mut var_idx = 0;

                for part in parts {
                    if part.is_empty() && var_idx < var_count {
                        let v_len = read_varint_fast(row_data, &mut rpos) as usize;
                        if rpos + v_len <= row_data.len() {
                            output.extend_from_slice(&row_data[rpos..rpos + v_len]);
                            rpos += v_len;
                        }
                        var_idx += 1;
                    } else {
                        output.extend_from_slice(part);
                    }
                }
                output.push(b'\n');
            }
        }

        Ok(output)
    }

    // ========================================================================
    // JL Sketch / Vector Similarity Search
    // ========================================================================

    pub fn sketch_dim(&self) -> usize { self.sketch_dim }
    pub fn sketch_bits(&self) -> u8 { self.sketch_bits }

    pub fn get_block_sketch(&self, block_idx: u64) -> Option<QuantizedVector> {
        if !self.has_jl_sketch || block_idx >= self.block_count {
            return None;
        }
        let pos = self.sketch_offset + (block_idx as usize * self.sketch_entry_size);
        if pos + self.sketch_entry_size > self.data.len() {
            return None;
        }
        QuantizedVector::from_bytes_with_dim(
            &self.data[pos..pos + self.sketch_entry_size],
            self.sketch_bits,
            self.sketch_dim,
        )
    }

    fn make_query_sketch(&self, embedding: &[f32]) -> Option<(VectorQuantizer, QuantizedVector)> {
        if embedding.is_empty() {
            return None;
        }
        let sketch = jl_sketch_from_embedding(embedding, self.sketch_dim);
        let norm: f32 = sketch.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm < 1e-10 {
            return None;
        }
        let quantizer = VectorQuantizer::with_dim(self.sketch_bits, self.sketch_dim);
        let query_qv = quantizer.encode(&sketch);
        Some((quantizer, query_qv))
    }

    pub fn search_similar(&self, query: &[f32], threshold: f32) -> Vec<SimilarityMatch> {
        if !self.has_jl_sketch || self.block_count == 0 {
            return Vec::new();
        }

        let (quantizer, query_qv) = match self.make_query_sketch(query) {
            Some(v) => v,
            None => return Vec::new(),
        };

        let mut matches: Vec<SimilarityMatch> = if self.block_count > 100 {
            (0..self.block_count)
                .into_par_iter()
                .filter_map(|block_idx| {
                    let block_qv = self.get_block_sketch(block_idx)?;
                    let score = quantizer.quantized_cosine_similarity(&query_qv, &block_qv);
                    if score >= threshold {
                        Some(SimilarityMatch { block_idx, score })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            (0..self.block_count)
                .filter_map(|block_idx| {
                    let block_qv = self.get_block_sketch(block_idx)?;
                    let score = quantizer.quantized_cosine_similarity(&query_qv, &block_qv);
                    if score >= threshold {
                        Some(SimilarityMatch { block_idx, score })
                    } else {
                        None
                    }
                })
                .collect()
        };

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    pub fn search_similar_top_k(&self, query: &[f32], k: usize) -> Vec<SimilarityMatch> {
        if !self.has_jl_sketch || self.block_count == 0 {
            return Vec::new();
        }

        let (quantizer, query_qv) = match self.make_query_sketch(query) {
            Some(v) => v,
            None => return Vec::new(),
        };

        let mut all: Vec<SimilarityMatch> = if self.block_count > 100 {
            (0..self.block_count)
                .into_par_iter()
                .filter_map(|block_idx| {
                    let block_qv = self.get_block_sketch(block_idx)?;
                    let score = quantizer.quantized_cosine_similarity(&query_qv, &block_qv);
                    Some(SimilarityMatch { block_idx, score })
                })
                .collect()
        } else {
            (0..self.block_count)
                .filter_map(|block_idx| {
                    let block_qv = self.get_block_sketch(block_idx)?;
                    let score = quantizer.quantized_cosine_similarity(&query_qv, &block_qv);
                    Some(SimilarityMatch { block_idx, score })
                })
                .collect()
        };

        all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(k);
        all
    }

    pub fn search_similar_with_results(&self, query: &[f32], threshold: f32) -> Vec<(f32, Vec<String>)> {
        let matches = self.search_similar(query, threshold);
        if matches.is_empty() {
            return Vec::new();
        }

        matches
            .into_iter()
            .filter_map(|m| {
                match self.decompress_block(m.block_idx) {
                    Ok(rows) => {
                        let lines: Vec<String> = rows
                            .into_iter()
                            .map(|row| String::from_utf8_lossy(&row).into_owned())
                            .collect();
                        Some((m.score, lines))
                    }
                    Err(_) => None,
                }
            })
            .collect()
    }

    pub fn get_row(&self, index: u64) -> Option<Vec<u8>> {
        if index >= self.row_count {
            return None;
        }

        let block_idx = index / self.block_size;
        let row_in_block = (index % self.block_size) as usize;

        let rows = self.decompress_block(block_idx).ok()?;
        rows.get(row_in_block).cloned()
    }
}
