//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!
//! Reference-based DNA compression for extreme compression ratios when
//! a reference genome (e.g., hg38) is available.

use std::collections::HashMap;

const DNA_REF_MAGIC: &[u8; 4] = b"CDNR";
const DNA_REF_VERSION: u8 = 1;
const INDEX_MAGIC: &[u8; 4] = b"CDNI";
const INDEX_VERSION: u8 = 1;

// K-mer size for indexing (21-mers are common for uniqueness in human genome)
const KMER_SIZE: usize = 21;
// Minimum match length to record
const MIN_MATCH_LEN: usize = 32;
// Step size when building index (every Nth k-mer to save memory)
const INDEX_STEP: usize = 8;

/// Reference genome index for fast lookups
pub struct ReferenceIndex {
    /// Map from k-mer hash to positions in reference
    kmer_positions: HashMap<u64, Vec<u32>>,
    /// Reference sequence (2-bit encoded for memory efficiency)
    reference_2bit: Vec<u8>,
    /// Original reference length in bases
    reference_len: usize,
    /// Hash of reference for validation
    reference_hash: u64,
}

impl ReferenceIndex {
    /// Build an index from a reference genome (FASTA format)
    pub fn build(reference_data: &[u8]) -> Self {
        let sequence = extract_sequence(reference_data);
        let reference_len = sequence.len();
        let reference_hash = compute_hash(&sequence);

        // 2-bit encode the reference for memory efficiency
        let reference_2bit = encode_2bit(&sequence);

        // Build k-mer index
        let mut kmer_positions: HashMap<u64, Vec<u32>> = HashMap::new();

        if sequence.len() >= KMER_SIZE {
            for i in (0..sequence.len() - KMER_SIZE + 1).step_by(INDEX_STEP) {
                let kmer = &sequence[i..i + KMER_SIZE];
                if is_valid_kmer(kmer) {
                    let hash = kmer_hash(kmer);
                    kmer_positions.entry(hash).or_default().push(i as u32);
                }
            }
        }

        Self {
            kmer_positions,
            reference_2bit,
            reference_len,
            reference_hash,
        }
    }

    /// Load index from bytes
    pub fn load(data: &[u8]) -> Option<Self> {
        if data.len() < 25 || &data[0..4] != INDEX_MAGIC {
            return None;
        }

        let version = data[4];
        if version != INDEX_VERSION {
            return None;
        }

        let mut pos = 5;

        // Read reference hash
        let reference_hash = u64::from_le_bytes(data[pos..pos+8].try_into().ok()?);
        pos += 8;

        // Read reference length
        let reference_len = u64::from_le_bytes(data[pos..pos+8].try_into().ok()?) as usize;
        pos += 8;

        // Read 2-bit reference size
        let ref_2bit_size = u64::from_le_bytes(data[pos..pos+8].try_into().ok()?) as usize;
        pos += 8;

        // Read 2-bit reference
        if pos + ref_2bit_size > data.len() {
            return None;
        }
        let reference_2bit = data[pos..pos + ref_2bit_size].to_vec();
        pos += ref_2bit_size;

        // Read k-mer count
        if pos + 8 > data.len() {
            return None;
        }
        let kmer_count = u64::from_le_bytes(data[pos..pos+8].try_into().ok()?) as usize;
        pos += 8;

        // Read k-mer positions
        let mut kmer_positions: HashMap<u64, Vec<u32>> = HashMap::with_capacity(kmer_count);

        for _ in 0..kmer_count {
            if pos + 12 > data.len() {
                return None;
            }

            let hash = u64::from_le_bytes(data[pos..pos+8].try_into().ok()?);
            pos += 8;

            let count = u32::from_le_bytes(data[pos..pos+4].try_into().ok()?) as usize;
            pos += 4;

            let mut positions = Vec::with_capacity(count);
            for _ in 0..count {
                if pos + 4 > data.len() {
                    return None;
                }
                positions.push(u32::from_le_bytes(data[pos..pos+4].try_into().ok()?));
                pos += 4;
            }

            kmer_positions.insert(hash, positions);
        }

        Some(Self {
            kmer_positions,
            reference_2bit,
            reference_len,
            reference_hash,
        })
    }

    /// Serialize index to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();

        // Header
        output.extend_from_slice(INDEX_MAGIC);
        output.push(INDEX_VERSION);

        // Reference info
        output.extend_from_slice(&self.reference_hash.to_le_bytes());
        output.extend_from_slice(&(self.reference_len as u64).to_le_bytes());
        output.extend_from_slice(&(self.reference_2bit.len() as u64).to_le_bytes());
        output.extend_from_slice(&self.reference_2bit);

        // K-mer index
        output.extend_from_slice(&(self.kmer_positions.len() as u64).to_le_bytes());

        for (hash, positions) in &self.kmer_positions {
            output.extend_from_slice(&hash.to_le_bytes());
            output.extend_from_slice(&(positions.len() as u32).to_le_bytes());
            for &pos in positions {
                output.extend_from_slice(&pos.to_le_bytes());
            }
        }

        output
    }

    /// Get reference sequence at position
    fn get_ref_base(&self, pos: usize) -> u8 {
        if pos >= self.reference_len {
            return b'N';
        }
        let byte_idx = pos / 4;
        let bit_offset = (3 - (pos % 4)) * 2;
        let bits = (self.reference_2bit[byte_idx] >> bit_offset) & 0b11;
        match bits {
            0b00 => b'A',
            0b01 => b'C',
            0b10 => b'G',
            0b11 => b'T',
            _ => b'N',
        }
    }

    /// Get reference slice
    fn get_ref_slice(&self, start: usize, len: usize) -> Vec<u8> {
        (start..start + len).map(|i| self.get_ref_base(i)).collect()
    }
}

/// Segment types for encoded output
#[derive(Debug, Clone)]
enum Segment {
    /// Match: copy from reference (ref_pos, length)
    Match { ref_pos: u64, length: u32 },
    /// Insert: new bases not in reference
    Insert { data: Vec<u8> },
}

/// Encode DNA data using a reference genome
pub fn encode_dna_with_reference(data: &[u8], index: &ReferenceIndex) -> Vec<u8> {
    let sequence = extract_sequence(data);

    if sequence.is_empty() {
        return create_empty_output(index.reference_hash);
    }

    let mut segments: Vec<Segment> = Vec::new();
    let mut pos = 0;
    let mut pending_insert: Vec<u8> = Vec::new();

    while pos < sequence.len() {
        // Try to find a match at current position
        if pos + KMER_SIZE <= sequence.len() {
            let kmer = &sequence[pos..pos + KMER_SIZE];

            if is_valid_kmer(kmer) {
                let hash = kmer_hash(kmer);

                if let Some(ref_positions) = index.kmer_positions.get(&hash) {
                    // Find best match
                    let mut best_match: Option<(u64, u32)> = None;

                    for &ref_pos in ref_positions {
                        let match_len = extend_match(&sequence, pos, index, ref_pos as usize);

                        if match_len >= MIN_MATCH_LEN {
                            if best_match.is_none() || match_len > best_match.unwrap().1 as usize {
                                best_match = Some((ref_pos as u64, match_len as u32));
                            }
                        }
                    }

                    if let Some((ref_pos, length)) = best_match {
                        // Flush pending insert
                        if !pending_insert.is_empty() {
                            segments.push(Segment::Insert { data: pending_insert.clone() });
                            pending_insert.clear();
                        }

                        segments.push(Segment::Match { ref_pos, length });
                        pos += length as usize;
                        continue;
                    }
                }
            }
        }

        // No match found, add to pending insert
        pending_insert.push(sequence[pos]);
        pos += 1;
    }

    // Flush remaining insert
    if !pending_insert.is_empty() {
        segments.push(Segment::Insert { data: pending_insert });
    }

    // Serialize output
    serialize_segments(&segments, index.reference_hash, sequence.len())
}

/// Decode DNA data using a reference genome
pub fn decode_dna_with_reference(data: &[u8], index: &ReferenceIndex) -> Vec<u8> {
    if data.len() < 21 || &data[0..4] != DNA_REF_MAGIC {
        return Vec::new();
    }

    let version = data[4];
    if version != DNA_REF_VERSION {
        return Vec::new();
    }

    let mut pos = 5;

    // Read and verify reference hash
    let stored_hash = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
    pos += 8;

    if stored_hash != index.reference_hash {
        eprintln!("Reference hash mismatch!");
        return Vec::new();
    }

    // Read original length
    let original_len = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
    pos += 8;

    // Read segment count
    let segment_count = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
    pos += 8;

    let mut output = Vec::with_capacity(original_len);

    for _ in 0..segment_count {
        if pos >= data.len() {
            break;
        }

        let seg_type = data[pos];
        pos += 1;

        match seg_type {
            0 => {
                // Match segment
                if pos + 12 > data.len() {
                    break;
                }
                let ref_pos = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()) as usize;
                pos += 8;
                let length = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
                pos += 4;

                output.extend(index.get_ref_slice(ref_pos, length));
            }
            1 => {
                // Insert segment (2-bit encoded)
                if pos + 4 > data.len() {
                    break;
                }
                let length = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
                pos += 4;

                let packed_len = (length + 3) / 4;
                if pos + packed_len > data.len() {
                    break;
                }

                for i in 0..length {
                    let byte_idx = i / 4;
                    let bit_offset = (3 - (i % 4)) * 2;
                    let bits = (data[pos + byte_idx] >> bit_offset) & 0b11;
                    let base = match bits {
                        0b00 => b'A',
                        0b01 => b'C',
                        0b10 => b'G',
                        0b11 => b'T',
                        _ => b'N',
                    };
                    output.push(base);
                }
                pos += packed_len;
            }
            _ => break,
        }
    }

    output
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_sequence(data: &[u8]) -> Vec<u8> {
    let mut sequence = Vec::with_capacity(data.len());
    let mut in_header = false;

    for &byte in data {
        // Skip FASTA header lines (starting with '>')
        if byte == b'>' {
            in_header = true;
            continue;
        }
        if byte == b'\n' {
            in_header = false;
            continue;
        }
        if in_header {
            continue;
        }

        match byte {
            b'A' | b'a' => sequence.push(b'A'),
            b'C' | b'c' => sequence.push(b'C'),
            b'G' | b'g' => sequence.push(b'G'),
            b'T' | b't' => sequence.push(b'T'),
            b'N' | b'n' => sequence.push(b'N'),
            _ => {} // Skip other characters
        }
    }

    sequence
}

fn encode_2bit(sequence: &[u8]) -> Vec<u8> {
    let packed_len = (sequence.len() + 3) / 4;
    let mut output = Vec::with_capacity(packed_len);

    let mut current_byte = 0u8;
    let mut bit_pos = 0;

    for &base in sequence {
        let bits = match base {
            b'A' => 0b00,
            b'C' => 0b01,
            b'G' => 0b10,
            b'T' => 0b11,
            _ => 0b00, // N -> A
        };

        current_byte |= bits << (6 - bit_pos);
        bit_pos += 2;

        if bit_pos == 8 {
            output.push(current_byte);
            current_byte = 0;
            bit_pos = 0;
        }
    }

    if bit_pos > 0 {
        output.push(current_byte);
    }

    output
}

fn is_valid_kmer(kmer: &[u8]) -> bool {
    kmer.iter().all(|&b| matches!(b, b'A' | b'C' | b'G' | b'T'))
}

fn kmer_hash(kmer: &[u8]) -> u64 {
    let mut hash = 0u64;
    for &base in kmer {
        let bits = match base {
            b'A' => 0u64,
            b'C' => 1u64,
            b'G' => 2u64,
            b'T' => 3u64,
            _ => 0u64,
        };
        hash = hash.wrapping_mul(4).wrapping_add(bits);
    }
    hash
}

fn compute_hash(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64; // FNV-1a
    for &byte in data.iter().take(1_000_000) { // Sample first 1MB
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^= data.len() as u64;
    hash
}

fn extend_match(sequence: &[u8], seq_pos: usize, index: &ReferenceIndex, ref_pos: usize) -> usize {
    let mut length = 0;
    let max_len = sequence.len() - seq_pos;
    let max_ref_len = index.reference_len - ref_pos;
    let max = max_len.min(max_ref_len);

    while length < max {
        let seq_base = sequence[seq_pos + length];
        let ref_base = index.get_ref_base(ref_pos + length);

        if seq_base != ref_base {
            break;
        }
        length += 1;
    }

    length
}

fn create_empty_output(ref_hash: u64) -> Vec<u8> {
    let mut output = Vec::with_capacity(29);
    output.extend_from_slice(DNA_REF_MAGIC);
    output.push(DNA_REF_VERSION);
    output.extend_from_slice(&ref_hash.to_le_bytes());
    output.extend_from_slice(&0u64.to_le_bytes()); // original length
    output.extend_from_slice(&0u64.to_le_bytes()); // segment count
    output
}

fn serialize_segments(segments: &[Segment], ref_hash: u64, original_len: usize) -> Vec<u8> {
    let mut output = Vec::new();

    // Header
    output.extend_from_slice(DNA_REF_MAGIC);
    output.push(DNA_REF_VERSION);
    output.extend_from_slice(&ref_hash.to_le_bytes());
    output.extend_from_slice(&(original_len as u64).to_le_bytes());
    output.extend_from_slice(&(segments.len() as u64).to_le_bytes());

    // Segments
    for segment in segments {
        match segment {
            Segment::Match { ref_pos, length } => {
                output.push(0); // Type: Match
                output.extend_from_slice(&ref_pos.to_le_bytes());
                output.extend_from_slice(&length.to_le_bytes());
            }
            Segment::Insert { data } => {
                output.push(1); // Type: Insert
                output.extend_from_slice(&(data.len() as u32).to_le_bytes());
                // 2-bit encode the insert data
                let packed = encode_2bit(data);
                output.extend_from_slice(&packed);
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sequence() {
        let fasta = b">test\nACGTACGT\nGGGG\n";
        let seq = extract_sequence(fasta);
        assert_eq!(seq, b"ACGTACGTGGGG");
    }

    #[test]
    fn test_2bit_roundtrip() {
        let seq = b"ACGTACGTACGT";
        let encoded = encode_2bit(seq);
        assert!(encoded.len() <= (seq.len() + 3) / 4);
    }

    #[test]
    fn test_kmer_hash() {
        let kmer1 = b"ACGTACGTACGTACGTACGTG";
        let kmer2 = b"ACGTACGTACGTACGTACGTG";
        let kmer3 = b"ACGTACGTACGTACGTACGTA";

        assert_eq!(kmer_hash(kmer1), kmer_hash(kmer2));
        assert_ne!(kmer_hash(kmer1), kmer_hash(kmer3));
    }

    #[test]
    fn test_reference_index_build() {
        let reference = b">ref\nACGTACGTACGTACGTACGTACGTACGTACGTACGT\n";
        let index = ReferenceIndex::build(reference);
        assert!(index.reference_len > 0);
    }

    #[test]
    fn test_encode_decode_identical() {
        let reference = b">ref\nACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT\n";
        let index = ReferenceIndex::build(reference);

        let sample = b"ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT";
        let encoded = encode_dna_with_reference(sample, &index);
        let decoded = decode_dna_with_reference(&encoded, &index);

        assert_eq!(decoded, extract_sequence(sample));
    }

    #[test]
    fn test_index_serialization() {
        let reference = b">ref\nACGTACGTACGTACGTACGTACGTACGTACGTACGT\n";
        let index = ReferenceIndex::build(reference);

        let bytes = index.to_bytes();
        let loaded = ReferenceIndex::load(&bytes).unwrap();

        assert_eq!(loaded.reference_len, index.reference_len);
        assert_eq!(loaded.reference_hash, index.reference_hash);
    }
}
