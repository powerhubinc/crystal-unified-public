//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

mod dna_2bit;
mod dna_fasta;
mod numeric_delta;
mod binary_delta;
mod nibble_split;
mod structured;
mod varint;

pub use dna_2bit::{
    encode_dna_2bit, decode_dna_2bit,
    codon_index, compute_dna_mod3_score, would_benefit_mod3 as dna_would_benefit_mod3,
};
pub use dna_fasta::{encode_dna_fasta, decode_dna_fasta};
pub use numeric_delta::{encode_numeric_delta, decode_numeric_delta};
pub use binary_delta::{encode_binary_delta, decode_binary_delta};
pub use nibble_split::{encode_nibble_split, decode_nibble_split, would_benefit as nibble_would_benefit};
pub use structured::{encode_structured, decode_structured, would_benefit as structured_would_benefit};
pub use varint::{encode_varint, decode_varint};

use crate::constants::{
    TRANSFORM_DNA_2BIT, TRANSFORM_DNA_FASTA, TRANSFORM_NUMERIC_DELTA,
    TRANSFORM_BINARY_DELTA, TRANSFORM_NIBBLE_SPLIT, TRANSFORM_STRUCTURED,
};

pub fn apply_transform(data: &[u8], transform_type: u8) -> Vec<u8> {
    match transform_type {
        TRANSFORM_DNA_2BIT => encode_dna_2bit(data),
        TRANSFORM_DNA_FASTA => encode_dna_fasta(data),
        TRANSFORM_NUMERIC_DELTA => encode_numeric_delta(data),
        TRANSFORM_BINARY_DELTA => encode_binary_delta(data),
        TRANSFORM_NIBBLE_SPLIT => encode_nibble_split(data),
        TRANSFORM_STRUCTURED => encode_structured(data),
        _ => data.to_vec(),
    }
}

pub fn reverse_transform(data: &[u8], transform_type: u8) -> Vec<u8> {
    match transform_type {
        TRANSFORM_DNA_2BIT => decode_dna_2bit(data),
        TRANSFORM_DNA_FASTA => decode_dna_fasta(data),
        TRANSFORM_NUMERIC_DELTA => decode_numeric_delta(data),
        TRANSFORM_BINARY_DELTA => decode_binary_delta(data),
        TRANSFORM_NIBBLE_SPLIT => decode_nibble_split(data),
        TRANSFORM_STRUCTURED => decode_structured(data),
        _ => data.to_vec(),
    }
}
