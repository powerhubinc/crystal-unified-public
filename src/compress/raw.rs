//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use byteorder::{LittleEndian, WriteBytesExt};
use zstd::zstd_safe::CDict;

use crate::constants::*;
use crate::options::CompressionOptions;
use crate::detection::CompressionFeatures;
use crate::transforms;
use super::helpers::{compress_no_dict, compress_with_dict};

pub struct RawCompressorV10 {
    options: CompressionOptions,
}

impl RawCompressorV10 {
    pub fn new() -> Self {
        Self::with_options(CompressionOptions::adaptive())
    }

    pub fn with_options(options: CompressionOptions) -> Self {
        Self { options }
    }

    pub fn compress(&self, data: &[u8], filename_hint: Option<&str>) -> Vec<u8> {
        if data.is_empty() {
            return self.write_empty();
        }

       
        let features = CompressionFeatures::extract(data, filename_hint);

       
        let transform_type = if self.options.auto_transform {
            features.data_type.recommended_transform()
        } else {
            self.options.transform_type
        };

       
        let level = if self.options.auto_level {
            features.recommended_level()
        } else {
            self.options.compression_level
        };

       
        let transformed = transforms::apply_transform(data, transform_type);

       
        let dictionary = if self.options.use_dictionary && transformed.len() > 1024 {
            let sample_size = transformed.len().min(self.options.dictionary_sample_bytes);
            let sample = &transformed[..sample_size];
            let sample_sizes = [sample_size];
            zstd::dict::from_continuous(sample, &sample_sizes, self.options.dictionary_size)
                .ok()
                .filter(|d| !d.is_empty())
        } else {
            None
        };

       
        let transformed_size = transformed.len();
        let compressed = if let Some(ref dict_data) = dictionary {
            let cdict = CDict::create(dict_data, level);
            compress_with_dict(&transformed, &cdict, level)
        } else {
            compress_no_dict(&transformed, level)
        };

       
        self.write_output(data.len(), transformed_size, &compressed, transform_type, level, dictionary)
    }

    pub fn compress_with_transform(&self, data: &[u8], transform_type: u8) -> Vec<u8> {
        if data.is_empty() {
            return self.write_empty();
        }

        let level = self.options.compression_level;

       
        let transformed = transforms::apply_transform(data, transform_type);

       
        let dictionary = if self.options.use_dictionary && transformed.len() > 1024 {
            let sample_size = transformed.len().min(self.options.dictionary_sample_bytes);
            let sample = &transformed[..sample_size];
            let sample_sizes = [sample_size];
            zstd::dict::from_continuous(sample, &sample_sizes, self.options.dictionary_size)
                .ok()
                .filter(|d| !d.is_empty())
        } else {
            None
        };

       
        let transformed_size = transformed.len();
        let compressed = if let Some(ref dict_data) = dictionary {
            let cdict = CDict::create(dict_data, level);
            compress_with_dict(&transformed, &cdict, level)
        } else {
            compress_no_dict(&transformed, level)
        };

        self.write_output(data.len(), transformed_size, &compressed, transform_type, level, dictionary)
    }

    fn write_empty(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(RAW_HEADER_SIZE);
        output.extend_from_slice(RAW_MAGIC);
        output.write_u32::<LittleEndian>(RAW_VERSION).unwrap();
        output.write_u64::<LittleEndian>(FLAG_RAW_MODE).unwrap();
        output.push(TRANSFORM_NONE);
        output.push(1);
        output.extend_from_slice(&[0u8; 6]);
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output.write_u64::<LittleEndian>(0).unwrap();
        output
    }

    fn write_output(
        &self,
        original_size: usize,
        transformed_size: usize,
        compressed: &[u8],
        transform_type: u8,
        level: i32,
        dictionary: Option<Vec<u8>>,
    ) -> Vec<u8> {
        let dict_compressed = dictionary.as_ref().map(|d| {
            zstd::stream::encode_all(&d[..], 3).unwrap_or_else(|_| d.clone())
        });
        let dict_size = dict_compressed.as_ref().map(|d| d.len()).unwrap_or(0);
        let orig_dict_size = dictionary.as_ref().map(|d| d.len()).unwrap_or(0);

        let mut flags = FLAG_RAW_MODE;
        if dictionary.is_some() {
            flags |= FLAG_HAS_DICTIONARY;
        }
        if transform_type != TRANSFORM_NONE {
            flags |= FLAG_HAS_TRANSFORM;
        }

        let mut output = Vec::with_capacity(RAW_HEADER_SIZE + dict_size + compressed.len());

       
        output.extend_from_slice(RAW_MAGIC);
        output.write_u32::<LittleEndian>(RAW_VERSION).unwrap();
        output.write_u64::<LittleEndian>(flags).unwrap();
        output.push(transform_type);
        output.push(level.clamp(1, 22) as u8);
        output.extend_from_slice(&[0u8; 6]);
        output.write_u64::<LittleEndian>(original_size as u64).unwrap();
        output.write_u64::<LittleEndian>(transformed_size as u64).unwrap();
        output.write_u64::<LittleEndian>(dict_size as u64).unwrap();
        output.write_u64::<LittleEndian>(orig_dict_size as u64).unwrap();

       
        if let Some(ref dict) = dict_compressed {
            output.extend_from_slice(dict);
        }

       
        output.extend_from_slice(compressed);

        output
    }
}

impl Default for RawCompressorV10 {
    fn default() -> Self {
        Self::new()
    }
}
