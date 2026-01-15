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
use crate::hash::{
    BloomFilter, TrigramBloom, WyHashMap,
    bloom_empty, bloom_add_line, wyhash,
    trigram_bloom_empty, trigram_bloom_add_line,
    bloom_add_lines_parallel, trigram_bloom_add_lines_parallel,
};
use crate::encoding::{group_varint_encode, write_varint};
use super::ParseBuffer;

pub struct BlockData {
    pub rows: Vec<Vec<u8>>,
    pub bloom: BloomFilter,
    pub trigram_bloom: Option<TrigramBloom>,
    pub template_data: Vec<Vec<u8>>,
    pub template_map: WyHashMap<u64, u16>,
    pub row_buf: Vec<u8>,
    pub original_lines: Vec<Vec<u8>>,
    pub fast_mode: bool,
}

impl BlockData {
    pub fn new(use_trigrams: bool) -> Self {
        Self::with_fast_mode(use_trigrams, false)
    }

    pub fn with_fast_mode(use_trigrams: bool, fast_mode: bool) -> Self {
        Self {
            rows: Vec::with_capacity(16384),
            bloom: bloom_empty(),
            trigram_bloom: if use_trigrams { Some(trigram_bloom_empty()) } else { None },
            template_data: Vec::with_capacity(if fast_mode { 0 } else { 1024 }),
            template_map: WyHashMap::default(),
            row_buf: Vec::with_capacity(if fast_mode { 0 } else { 256 }),
            original_lines: Vec::with_capacity(if fast_mode { 16384 } else { 0 }),
            fast_mode,
        }
    }

    pub fn clear(&mut self, use_trigrams: bool) {
        self.rows.clear();
        self.original_lines.clear();
       
        if self.bloom.len() != BLOOM_SIZE_U64 {
            self.bloom = bloom_empty();
        } else {
            for v in self.bloom.iter_mut() {
                *v = 0;
            }
        }
        if use_trigrams {
            if let Some(ref mut tb) = self.trigram_bloom {
                if tb.len() == TRIGRAM_BLOOM_SIZE_U64 {
                    for v in tb.iter_mut() {
                        *v = 0;
                    }
                } else {
                    self.trigram_bloom = Some(trigram_bloom_empty());
                }
            } else {
                self.trigram_bloom = Some(trigram_bloom_empty());
            }
        }
        self.template_data.clear();
        self.template_map.clear();
    }
    #[inline]
    pub fn take_blooms_and_reset(&mut self, use_trigrams: bool) -> (BloomFilter, Option<TrigramBloom>) {
       
        let bloom = std::mem::replace(&mut self.bloom, bloom_empty());
        let trigram = self.trigram_bloom.take();

       
        self.rows.clear();
        self.original_lines.clear();
        self.template_data.clear();
        self.template_map.clear();

       
        self.bloom = bloom_empty();
        if use_trigrams {
            self.trigram_bloom = Some(trigram_bloom_empty());
        }

        (bloom, trigram)
    }

    #[inline]
    pub fn finalize_blooms_parallel(&mut self) {
        if !self.fast_mode || self.original_lines.len() < 100 {
            return;
        }
       
        self.bloom = bloom_add_lines_parallel(&self.original_lines);
        if self.trigram_bloom.is_some() {
            self.trigram_bloom = Some(trigram_bloom_add_lines_parallel(&self.original_lines));
        }
    }
    #[inline]
    pub fn add_line_fast(&mut self, line: &[u8]) {
       
        self.original_lines.push(line.to_vec());
    }
        /// Add a line to the block
    #[inline]
    pub fn add_line(&mut self, line: &[u8], parse_buf: &mut ParseBuffer) {
       
        bloom_add_line(&mut self.bloom, line);

        if let Some(ref mut tb) = self.trigram_bloom {
            trigram_bloom_add_line(tb, line);
        }

       
        if self.fast_mode {
            self.original_lines.push(line.to_vec());
            return;
        }

       
        parse_buf.parse_line(line);

        let tmpl_id = if parse_buf.overflow {
            LITERAL_TEMPLATE_ID
        } else {
            let tmpl = parse_buf.build_template(line);
            let tmpl_hash = wyhash(tmpl, 0);

            if let Some(&id) = self.template_map.get(&tmpl_hash) {
                id
            } else if self.template_data.len() >= (LITERAL_TEMPLATE_ID as usize) {
                LITERAL_TEMPLATE_ID
            } else {
                let id = self.template_data.len() as u16;
                self.template_map.insert(tmpl_hash, id);
                self.template_data.push(tmpl.to_vec());
                id
            }
        };

        self.row_buf.clear();
        self.row_buf.write_u16::<LittleEndian>(tmpl_id).unwrap();

        if tmpl_id == LITERAL_TEMPLATE_ID {
            self.row_buf.write_u16::<LittleEndian>(0).unwrap();
            write_varint(&mut self.row_buf, line.len() as u64).unwrap();
            self.row_buf.extend_from_slice(line);
        } else {
            self.row_buf.write_u16::<LittleEndian>(parse_buf.var_count as u16).unwrap();
            for i in 0..parse_buf.var_count {
                let (start, end) = parse_buf.var_ranges[i];
                let var = &line[start as usize..end as usize];
                write_varint(&mut self.row_buf, var.len() as u64).unwrap();
                self.row_buf.extend_from_slice(var);
            }
        }

       
        self.rows.push(std::mem::replace(&mut self.row_buf, Vec::with_capacity(256)));
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut output = Vec::new();

       
        output.write_u32::<LittleEndian>(self.template_data.len() as u32).unwrap();
        for tmpl in &self.template_data {
            output.extend_from_slice(tmpl);
        }

       
        output.write_u32::<LittleEndian>(self.rows.len() as u32).unwrap();

       
        let row_count = self.rows.len();
        let full_groups = row_count / 4;
        let _remainder = row_count % 4;

        for g in 0..full_groups {
            let vals = [
                self.rows[g * 4].len() as u32,
                self.rows[g * 4 + 1].len() as u32,
                self.rows[g * 4 + 2].len() as u32,
                self.rows[g * 4 + 3].len() as u32,
            ];
            let (control, data, len) = group_varint_encode(&vals);
            output.push(control);
            output.extend_from_slice(&data[..len]);
        }

        for i in (full_groups * 4)..row_count {
            write_varint(&mut output, self.rows[i].len() as u64).unwrap();
        }

       
        for row in &self.rows {
            output.extend_from_slice(row);
        }

        output
    }

    pub fn serialize_fast(&self) -> Vec<u8> {
       
        let total_size: usize = self.original_lines.iter().map(|l| l.len() + 1).sum();
        let mut output = Vec::with_capacity(total_size);

       
        for line in &self.original_lines {
            output.extend_from_slice(line);
            output.push(b'\n');
        }

        output
    }

    #[inline]
    pub fn row_count(&self) -> usize {
        if self.fast_mode {
            self.original_lines.len()
        } else {
            self.rows.len()
        }
    }
}
