//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

pub mod constants;
pub mod error;
pub mod options;

pub mod detection;
pub mod transforms;
pub mod hash;
pub mod encoding;
pub mod block;
pub mod dictionary;
pub mod compress;
pub mod decompress;

mod api;

pub use error::{CrystalError, Result};
pub use options::CompressionOptions;
pub use detection::{DataType, CompressionFeatures, SmartDetectionResult, smart_detect, would_bloat};
pub use compress::{BurnerV10, ParallelBurnerV10, RawCompressorV10, StreamingBurnerV10, compress_ultra_fast};
pub use compress::{BlockRawCompressor, BlockRawArchive, CompressedBlock};
pub use decompress::{CrystalReaderV10, RawDecompressorV10, BlockRawReader};

pub use api::{compress, decompress, compress_with_options, compress_parallel, compress_parallel_with_options, compress_streaming, compress_streaming_with_options};

pub use constants::{TRANSFORM_NONE, TRANSFORM_DNA_2BIT, TRANSFORM_NUMERIC_DELTA, TRANSFORM_BINARY_DELTA, TRANSFORM_NIBBLE_SPLIT, TRANSFORM_STRUCTURED};

pub use constants::{
    DETECT_THRESHOLD, SIGNAL_FREQUENCY, DEFAULT_DEPTH, MIN_DEPTH, MAX_DEPTH,
    DETECT_SAMPLE_SIZE, DETECTION_SAMPLE_SIZE,
    SMALL_FILE_THRESHOLD, TINY_FILE_THRESHOLD, MIN_COMPRESSION_BENEFIT,
    BLOCK_RAW_MAGIC, BLOCK_RAW_VERSION, BLOCK_RAW_HEADER_SIZE, DEFAULT_BINARY_BLOCK_SIZE,
};
