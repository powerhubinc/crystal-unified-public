//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use crate::options::CompressionOptions;
use crate::compress::{BurnerV10, ParallelBurnerV10, RawCompressorV10, StreamingBurnerV10};
use crate::decompress::{CrystalReaderV10, RawDecompressorV10};
use crate::error::Result;

pub fn compress(data: &[u8]) -> Vec<u8> {
    compress_with_options(data, CompressionOptions::default())
}

pub fn compress_parallel(data: &[u8]) -> Vec<u8> {
    compress_parallel_with_options(data, CompressionOptions::default())
}

pub fn compress_with_options(data: &[u8], options: CompressionOptions) -> Vec<u8> {
   
    if options.raw_mode {
        let compressor = RawCompressorV10::with_options(options);
        return compressor.compress(data, None);
    }

   
    if data.is_empty() {
        let mut burner = BurnerV10::with_options(options);
        return burner.serialize();
    }

    let trailing_newline = data.last() == Some(&b'\n');
    let content = if trailing_newline { &data[..data.len() - 1] } else { data };

    let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();

    let mut burner = BurnerV10::with_options(options);
    burner.set_trailing_newline(trailing_newline);
    for line in &lines {
        burner.ingest_line(line);
    }
    burner.serialize()
}

pub fn compress_parallel_with_options(data: &[u8], options: CompressionOptions) -> Vec<u8> {
    if data.is_empty() {
        let mut burner = BurnerV10::with_options(options.clone());
        return burner.serialize();
    }

    let trailing_newline = data.last() == Some(&b'\n');
    let content = if trailing_newline { &data[..data.len() - 1] } else { data };

    let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();

    let mut burner = ParallelBurnerV10::with_options(options);
    burner.set_trailing_newline(trailing_newline);
    burner.compress_parallel(&lines)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
   
    if RawDecompressorV10::is_raw_format(data) {
        return RawDecompressorV10::decompress(data);
    }

   
    let reader = CrystalReaderV10::new(data)?;
    Ok(reader.reconstruct())
}

pub fn compress_streaming(data: &[u8]) -> Vec<u8> {
    compress_streaming_with_options(data, CompressionOptions::default())
}

pub fn compress_streaming_with_options(data: &[u8], options: CompressionOptions) -> Vec<u8> {
    if data.is_empty() {
        let mut burner = StreamingBurnerV10::with_options(options);
        return burner.serialize();
    }

    let trailing_newline = data.last() == Some(&b'\n');
    let content = if trailing_newline { &data[..data.len() - 1] } else { data };

    let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();

    let mut burner = StreamingBurnerV10::with_options(options);
    burner.set_trailing_newline(trailing_newline);
    for line in &lines {
        burner.ingest_line(line);
    }
    burner.serialize()
}
