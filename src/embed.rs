//! Embed external vectors into Crystal archives
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::constants::*;
use crate::error::{Result, CrystalError};
use crate::decompress::CrystalReaderV10;
use crate::sketch::{VectorQuantizer, jl_sketch_from_embedding};

/// Add a vector index to an existing Crystal archive.
///
/// Each block gets one embedding vector, quantized to `bits` per dim.
/// The number of embeddings must equal the archive's block count.
/// Embeddings can be any dimensionality -- they will be projected to `dim` if needed.
///
/// Returns a new archive with the vector index embedded.
pub fn embed_archive(
    archive: &[u8],
    embeddings: &[Vec<f32>],
    dim: usize,
    bits: u8,
) -> Result<Vec<u8>> {
    let reader = CrystalReaderV10::new(archive)?;

    if reader.has_jl_sketch() {
        return Err(CrystalError::InvalidFormat("Archive already has a vector index".into()));
    }

    let block_count = reader.block_count() as usize;
    if embeddings.len() != block_count {
        return Err(CrystalError::InvalidFormat(
            format!("Expected {} embeddings (one per block), got {}", block_count, embeddings.len())
        ));
    }

    let tq = VectorQuantizer::with_dim(bits, dim);
    let entry_size = jl_sketch_entry_size(dim, bits);

    // Quantize all embeddings
    let mut sketch_section = Vec::with_capacity(block_count * entry_size);
    for emb in embeddings {
        let sketch = jl_sketch_from_embedding(emb, dim);
        let qv = tq.encode(&sketch);
        sketch_section.extend_from_slice(&qv.to_bytes());
    }

    // Parse header fields we need
    let mut cursor = Cursor::new(&archive[4..]);
    let _version = cursor.read_u32::<LittleEndian>()?;
    let _block_size = cursor.read_u64::<LittleEndian>()?;
    let _row_count = cursor.read_u64::<LittleEndian>()?;
    let _block_count_header = cursor.read_u64::<LittleEndian>()?;
    let flags = cursor.read_u64::<LittleEndian>()?;
    let _compression_level = cursor.read_i32::<LittleEndian>()?;
    let dict_size = cursor.read_u64::<LittleEndian>()? as usize;

    let index_offset = HEADER_SIZE + dict_size;
    let bloom_offset = index_offset + block_count * BLOCK_INDEX_ENTRY_SIZE;
    let trigram_offset = bloom_offset + block_count * BLOOM_SIZE_BYTES;
    let old_data_offset = if (flags & FLAG_HAS_TRIGRAMS) != 0 {
        trigram_offset + block_count * TRIGRAM_BLOOM_SIZE_BYTES
    } else {
        trigram_offset
    };

    // Build new archive: header + metadata sections + sketch section + compressed data
    let mut output = Vec::with_capacity(archive.len() + sketch_section.len() + 8);

    // Copy header
    output.extend_from_slice(&archive[..HEADER_SIZE]);

    // Set FLAG_HAS_JL_SKETCH in flags (bytes 32-39 of header)
    let new_flags = flags | FLAG_HAS_JL_SKETCH;
    output[32..40].copy_from_slice(&new_flags.to_le_bytes());

    // Set sketch dim and bits at bytes 60-61
    output[HEADER_SKETCH_DIM_OFFSET] = dim as u8;
    output[HEADER_SKETCH_BITS_OFFSET] = bits;

    // Zero out IDF size at bytes 62-65 (no IDF table)
    output[HEADER_IDF_SIZE_OFFSET..HEADER_IDF_SIZE_OFFSET + 4].copy_from_slice(&0u32.to_le_bytes());

    // Copy dict + block index + blooms + trigrams (everything up to old data_offset)
    output.extend_from_slice(&archive[HEADER_SIZE..old_data_offset]);

    // Insert sketch section
    output.extend_from_slice(&sketch_section);

    // Copy compressed block data
    output.extend_from_slice(&archive[old_data_offset..]);

    Ok(output)
}
