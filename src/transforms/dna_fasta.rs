//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!


const FASTA_MAGIC: &[u8; 4] = b"FDNA";
const VERSION: u8 = 2;

pub fn encode_dna_fasta(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(data.len() / 3);

   
    output.extend_from_slice(FASTA_MAGIC);
    output.push(VERSION);

   
    let lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();

   
    let mut headers: Vec<&[u8]> = Vec::new();
    let mut seq_lines: Vec<&[u8]> = Vec::new();
    let mut seq_line_lengths: Vec<usize> = Vec::new();

    for line in &lines {
        if line.starts_with(b">") {
            headers.push(line);
        } else if !line.is_empty() {
            seq_lines.push(line);
            seq_line_lengths.push(line.len());
        }
    }

   
    write_varint(&mut output, headers.len() as u64);
    for header in &headers {
        write_varint(&mut output, header.len() as u64);
        output.extend_from_slice(header);
    }

   
    let uniform_len = detect_uniform_line_length(&seq_line_lengths);
    write_varint(&mut output, uniform_len as u64);

   
    write_varint(&mut output, seq_line_lengths.len() as u64);

   
    if uniform_len == 0 {
        for &len in &seq_line_lengths {
            write_varint(&mut output, len as u64);
        }
    }

   
    let mut sequence = Vec::new();
    let mut n_positions: Vec<bool> = Vec::new();

    for line in &seq_lines {
        for &b in *line {
            if is_base(b) {
                let is_n = matches!(b, b'N' | b'n');
                n_positions.push(is_n);
               
                if is_n {
                    sequence.push(b'A');
                } else {
                    sequence.push(b);
                }
            }
        }
    }

    let total_bases = sequence.len() as u64;

   
    output.extend_from_slice(&total_bases.to_le_bytes());

   
    let n_rle = rle_encode_bitmask(&n_positions);
    write_varint(&mut output, n_rle.len() as u64);
    output.extend_from_slice(&n_rle);

   
    let packed = pack_2bit(&sequence);
    output.extend_from_slice(&packed);

   
    let has_trailing_newline = data.last() == Some(&b'\n');
    output.push(if has_trailing_newline { 1 } else { 0 });

    output
}

pub fn decode_dna_fasta(data: &[u8]) -> Vec<u8> {
    if data.len() < 6 {
        return Vec::new();
    }

   
    if &data[0..4] != FASTA_MAGIC {
        return Vec::new();
    }

    let version = data[4];
    let mut pos = 5;

   
    let (header_count, bytes_read) = read_varint(&data[pos..]);
    pos += bytes_read;

   
    let mut headers: Vec<Vec<u8>> = Vec::new();
    for _ in 0..header_count {
        let (header_len, bytes_read) = read_varint(&data[pos..]);
        pos += bytes_read;
        headers.push(data[pos..pos + header_len as usize].to_vec());
        pos += header_len as usize;
    }

   
    let (uniform_len, bytes_read) = read_varint(&data[pos..]);
    pos += bytes_read;

   
    let (line_count, bytes_read) = read_varint(&data[pos..]);
    pos += bytes_read;

   
    let line_lengths: Vec<usize> = if uniform_len == 0 {
        let mut lengths = Vec::with_capacity(line_count as usize);
        for _ in 0..line_count {
            let (len, bytes_read) = read_varint(&data[pos..]);
            pos += bytes_read;
            lengths.push(len as usize);
        }
        lengths
    } else {
        vec![uniform_len as usize; line_count as usize]
    };

   
    let total_bases = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
    pos += 8;

   
    let n_positions = if version >= 2 {
        let (n_rle_len, bytes_read) = read_varint(&data[pos..]);
        pos += bytes_read;
        let n_rle = &data[pos..pos + n_rle_len as usize];
        pos += n_rle_len as usize;
        rle_decode_bitmask(n_rle, total_bases)
    } else {
        vec![false; total_bases]
    };

   
    let packed_size = (total_bases + 3) / 4;
    let packed_data = &data[pos..pos + packed_size];
    pos += packed_size;

   
    let has_trailing_newline = data.get(pos) == Some(&1);

   
    let mut sequence = unpack_2bit(packed_data, total_bases);

   
    for (i, &is_n) in n_positions.iter().enumerate() {
        if is_n && i < sequence.len() {
            sequence[i] = b'N';
        }
    }

   
    let mut output = Vec::with_capacity(data.len() * 3);
    let mut seq_pos = 0;

   
    let lines_per_header = if headers.is_empty() {
        line_count as usize
    } else {
        (line_count as usize + headers.len() - 1) / headers.len()
    };

    for (h_idx, header) in headers.iter().enumerate() {
       
        output.extend_from_slice(header);
        output.push(b'\n');

       
        let start_line = h_idx * lines_per_header;
        let end_line = if h_idx == headers.len() - 1 {
            line_count as usize
        } else {
            (h_idx + 1) * lines_per_header
        };

        for l_idx in start_line..end_line {
            if l_idx < line_lengths.len() {
                let line_len = line_lengths[l_idx];
                let end_pos = (seq_pos + line_len).min(sequence.len());
                output.extend_from_slice(&sequence[seq_pos..end_pos]);
                seq_pos = end_pos;

               
                if l_idx < line_lengths.len() - 1 || has_trailing_newline {
                    output.push(b'\n');
                }
            }
        }
    }

   
    if headers.is_empty() && !line_lengths.is_empty() {
        for (l_idx, &line_len) in line_lengths.iter().enumerate() {
            let end_pos = (seq_pos + line_len).min(sequence.len());
            output.extend_from_slice(&sequence[seq_pos..end_pos]);
            seq_pos = end_pos;

            if l_idx < line_lengths.len() - 1 || has_trailing_newline {
                output.push(b'\n');
            }
        }
    }

    output
}

fn rle_encode_bitmask(bits: &[bool]) -> Vec<u8> {
    if bits.is_empty() {
        return Vec::new();
    }

    let mut output = Vec::new();
    let mut current_bit = bits[0];
    let mut run_length: u64 = 1;

   
    output.push(if current_bit { 1 } else { 0 });

    for &bit in bits.iter().skip(1) {
        if bit == current_bit {
            run_length += 1;
        } else {
           
            write_varint(&mut output, run_length);
            current_bit = bit;
            run_length = 1;
        }
    }

   
    write_varint(&mut output, run_length);

    output
}

fn rle_decode_bitmask(data: &[u8], expected_len: usize) -> Vec<bool> {
    if data.is_empty() {
        return vec![false; expected_len];
    }

    let mut output = Vec::with_capacity(expected_len);
    let mut current_bit = data[0] == 1;
    let mut pos = 1;

    while pos < data.len() && output.len() < expected_len {
        let (run_length, bytes_read) = read_varint(&data[pos..]);
        pos += bytes_read;

        for _ in 0..run_length {
            if output.len() >= expected_len {
                break;
            }
            output.push(current_bit);
        }
        current_bit = !current_bit;
    }

   
    while output.len() < expected_len {
        output.push(false);
    }

    output
}

fn detect_uniform_line_length(lengths: &[usize]) -> usize {
    if lengths.is_empty() {
        return 0;
    }

    if lengths.len() == 1 {
        return lengths[0];
    }

    let first_len = lengths[0];
    let all_same = lengths[..lengths.len() - 1].iter().all(|&l| l == first_len);

    if all_same && lengths.last().map_or(true, |&l| l <= first_len) {
        first_len
    } else {
        0
    }
}

#[inline]
fn is_base(b: u8) -> bool {
    matches!(b, b'A' | b'a' | b'C' | b'c' | b'G' | b'g' | b'T' | b't' | b'N' | b'n')
}

fn pack_2bit(sequence: &[u8]) -> Vec<u8> {
    let packed_size = (sequence.len() + 3) / 4;
    let mut packed = Vec::with_capacity(packed_size);

    let mut current_byte: u8 = 0;
    let mut bit_pos = 0;

    for &base in sequence {
        let bits = match base {
            b'A' | b'a' => 0b00,
            b'C' | b'c' => 0b01,
            b'G' | b'g' => 0b10,
            b'T' | b't' => 0b11,
            _ => 0b00,
        };

        current_byte |= bits << (6 - bit_pos);
        bit_pos += 2;

        if bit_pos == 8 {
            packed.push(current_byte);
            current_byte = 0;
            bit_pos = 0;
        }
    }

    if bit_pos > 0 {
        packed.push(current_byte);
    }

    packed
}

fn unpack_2bit(packed: &[u8], total_bases: usize) -> Vec<u8> {
    let mut sequence = Vec::with_capacity(total_bases);
    let mut bases_unpacked = 0;

    for &byte in packed {
        for shift in [6, 4, 2, 0] {
            if bases_unpacked >= total_bases {
                break;
            }
            let bits = (byte >> shift) & 0b11;
            let base = match bits {
                0b00 => b'A',
                0b01 => b'C',
                0b10 => b'G',
                0b11 => b'T',
                _ => unreachable!(),
            };
            sequence.push(base);
            bases_unpacked += 1;
        }
    }

    sequence
}

fn write_varint(output: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(data: &[u8]) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in data {
        bytes_read += 1;
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    (value, bytes_read)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fasta_roundtrip() {
        let fasta = b">seq1\nACGTACGT\nACGT\n";
        let encoded = encode_dna_fasta(fasta);
        let decoded = decode_dna_fasta(&encoded);
        assert_eq!(decoded, fasta.to_vec());
    }

    #[test]
    fn test_fasta_with_n() {
        let fasta = b">seq1\nNNNNACGT\nACGTNNNN\n";
        let encoded = encode_dna_fasta(fasta);
        let decoded = decode_dna_fasta(&encoded);
        assert_eq!(decoded, fasta.to_vec(), "N bases should be preserved");
    }

    #[test]
    fn test_long_n_run() {
       
        let n_run: Vec<u8> = std::iter::repeat(b'N').take(1000).collect();
        let bases = b"ACGTACGT";
        let fasta = format!(">chr1\n{}{}\n", String::from_utf8_lossy(&n_run), String::from_utf8_lossy(bases));
        let encoded = encode_dna_fasta(fasta.as_bytes());
        let decoded = decode_dna_fasta(&encoded);
        assert_eq!(decoded, fasta.as_bytes().to_vec());
    }

    #[test]
    fn test_multi_header_fasta() {
        let fasta = b">seq1\nACGTACGT\n>seq2\nGGGGAAAA\n";
        let encoded = encode_dna_fasta(fasta);
        let decoded = decode_dna_fasta(&encoded);
        assert_eq!(decoded, fasta.to_vec());
    }

    #[test]
    fn test_no_trailing_newline() {
        let fasta = b">seq1\nACGTACGT";
        let encoded = encode_dna_fasta(fasta);
        let decoded = decode_dna_fasta(&encoded);
        assert_eq!(decoded, fasta.to_vec());
    }

    #[test]
    fn test_uniform_line_detection() {
        let lengths = vec![60, 60, 60, 45];
        assert_eq!(detect_uniform_line_length(&lengths), 60);

        let variable = vec![60, 70, 60];
        assert_eq!(detect_uniform_line_length(&variable), 0);
    }

    #[test]
    fn test_rle_roundtrip() {
       
        let mut bits = vec![true; 10000];
        bits.extend(vec![false; 5000]);
        bits.extend(vec![true; 100]);

        let encoded = rle_encode_bitmask(&bits);
        let decoded = rle_decode_bitmask(&encoded, bits.len());

        assert_eq!(decoded, bits);
       
        assert!(encoded.len() < 20, "RLE should be compact for long runs");
    }

    #[test]
    fn test_compression_ratio() {
        let bases: Vec<u8> = (0..1000).map(|i| match i % 4 {
            0 => b'A', 1 => b'C', 2 => b'G', _ => b'T'
        }).collect();
        let fasta = format!(">test\n{}\n", String::from_utf8_lossy(&bases));
        let encoded = encode_dna_fasta(fasta.as_bytes());

        assert!(encoded.len() < fasta.len() / 2);
    }
}
