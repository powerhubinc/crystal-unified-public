//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

// ============================================================================
// File Format Constants
// ============================================================================

pub const MAGIC: &[u8; 4] = b"CUZ1";

pub const RAW_MAGIC: &[u8; 4] = b"CUZR";

pub const BLOCK_RAW_MAGIC: &[u8; 4] = b"CUZB";

pub const VERSION: u32 = 100;

pub const RAW_VERSION: u32 = 100;

pub const BLOCK_RAW_VERSION: u32 = 100;

pub const HEADER_SIZE: usize = 96;

pub const BLOCK_RAW_HEADER_SIZE: usize = 64;

pub const DEFAULT_BINARY_BLOCK_SIZE: usize = 64 * 1024;

pub const RAW_HEADER_SIZE: usize = 56;

pub const BLOCK_INDEX_ENTRY_SIZE: usize = 24;

// ============================================================================
// Default Configuration
// ============================================================================

pub const DEFAULT_BLOCK_SIZE: u64 = 16384;

pub const DEFAULT_DICTIONARY_SIZE: usize = 64 * 1024;

pub const DEFAULT_DICTIONARY_SAMPLES: usize = 10 * 1024 * 1024;

pub const DEFAULT_COMPRESSION_LEVEL: i32 = 1;

// ============================================================================
// Flags
// ============================================================================

pub const FLAG_TRAILING_NEWLINE: u64 = 1;

pub const FLAG_HAS_DICTIONARY: u64 = 2;

pub const FLAG_HAS_TRIGRAMS: u64 = 4;

pub const FLAG_HAS_TRANSFORM: u64 = 8;

pub const FLAG_RAW_MODE: u64 = 16;

pub const FLAG_FAST_MODE: u64 = 32;

pub const FLAG_STREAMING_MODE: u64 = 64;

// ============================================================================
// Transform Type IDs
// ============================================================================

pub const TRANSFORM_NONE: u8 = 0;

pub const TRANSFORM_DNA_2BIT: u8 = 1;

pub const TRANSFORM_NUMERIC_DELTA: u8 = 2;

pub const TRANSFORM_BINARY_DELTA: u8 = 3;

pub const TRANSFORM_NIBBLE_SPLIT: u8 = 4;

pub const TRANSFORM_STRUCTURED: u8 = 5;

pub const TRANSFORM_DNA_FASTA: u8 = 6;

// ============================================================================
// Detection Constants
// ============================================================================

pub const DETECT_THRESHOLD: f64 = 0.8;

pub const SIGNAL_FREQUENCY: f64 = 2.922282;

pub const DEFAULT_DEPTH: usize = 3;

pub const MIN_DEPTH: usize = 1;

pub const MAX_DEPTH: usize = 180;

pub const DETECT_SAMPLE_SIZE: usize = 4 * 1024;

pub const DETECTION_SAMPLE_SIZE: usize = 4 * 1024;

pub const SMALL_FILE_THRESHOLD: usize = 32 * 1024; // 32KB

pub const TINY_FILE_THRESHOLD: usize = 256; // 256 bytes

pub const MIN_COMPRESSION_BENEFIT: f64 = 0.95;

// ============================================================================
// Template Constants
// ============================================================================

pub const LITERAL_TEMPLATE_ID: u16 = 65535;

pub const MAX_TEMPLATE_PARTS: usize = 32;

pub const MAX_VAR_RANGES: usize = 16;

// ============================================================================
// Bloom Filter Constants
// ============================================================================

pub const BLOOM_SIZE_U64: usize = 1024;

pub const BLOOM_SIZE_BYTES: usize = BLOOM_SIZE_U64 * 8;

pub const BLOOM_SIZE_BITS: usize = BLOOM_SIZE_U64 * 64;

pub const TRIGRAM_BLOOM_SIZE_U64: usize = 1024;

pub const TRIGRAM_BLOOM_SIZE_BYTES: usize = TRIGRAM_BLOOM_SIZE_U64 * 8;

pub const TRIGRAM_BLOOM_SIZE_BITS: usize = TRIGRAM_BLOOM_SIZE_U64 * 64;
