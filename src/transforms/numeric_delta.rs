//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

use super::varint::{encode_varint, decode_varint};

const MAGIC: [u8; 4] = *b"ND10";
const HEADER_SIZE: usize = 24;

fn detect_width(data: &[u8]) -> u8 {
    if data.len() < 8 {
        return 1;
    }

   
   
    if data.len() % 2 == 0 {
       
        let mut looks_like_16bit = true;
        let samples = (data.len() / 2).min(1000);
        for i in 0..samples {
            let val = u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
           
           
            if val > 4095 && i < 100 {
               
                looks_like_16bit = false;
                break;
            }
        }
        if looks_like_16bit {
            return 2;
        }
    }

   
    if data.len() % 4 == 0 {
        return 4;
    }

   
    1
}

pub fn encode_numeric_delta(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        let mut output = Vec::with_capacity(HEADER_SIZE);
        output.extend_from_slice(&MAGIC);
        output.extend_from_slice(&[1, 0, 0, 0]);
        output.extend_from_slice(&0u64.to_le_bytes());
        output.extend_from_slice(&0i64.to_le_bytes());
        return output;
    }

    let width = detect_width(data);

    match width {
        2 => encode_16bit(data),
        4 => encode_32bit(data),
        _ => encode_8bit(data),
    }
}

fn encode_8bit(data: &[u8]) -> Vec<u8> {
    let count = data.len();
    let mut output = Vec::with_capacity(HEADER_SIZE + count);

   
    output.extend_from_slice(&MAGIC);
    output.push(1); // width
    output.push(0);
    output.extend_from_slice(&[0, 0]);
    output.extend_from_slice(&(count as u64).to_le_bytes());
    output.extend_from_slice(&(data[0] as i64).to_le_bytes());

   
    let mut prev = data[0] as i64;
    for &byte in &data[1..] {
        let val = byte as i64;
        let delta = val - prev;
        let zigzag = ((delta << 1) ^ (delta >> 63)) as u64;
        encode_varint(&mut output, zigzag);
        prev = val;
    }

    output
}

fn encode_16bit(data: &[u8]) -> Vec<u8> {
    let count = data.len() / 2;
    let remainder = data.len() % 2;

    let mut output = Vec::with_capacity(HEADER_SIZE + count * 2);

   
    output.extend_from_slice(&MAGIC);
    output.push(2); // width
    output.push(remainder as u8);
    output.extend_from_slice(&[0, 0]);
    output.extend_from_slice(&(count as u64).to_le_bytes());

    if count == 0 {
        output.extend_from_slice(&0i64.to_le_bytes());
       
        output.extend_from_slice(&data[..remainder]);
        return output;
    }

    let first = i16::from_le_bytes([data[0], data[1]]) as i64;
    output.extend_from_slice(&first.to_le_bytes());

   
    let mut prev = first;
    for i in 1..count {
        let val = i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]) as i64;
        let delta = val - prev;
        let zigzag = ((delta << 1) ^ (delta >> 63)) as u64;
        encode_varint(&mut output, zigzag);
        prev = val;
    }

   
    if remainder > 0 {
        output.extend_from_slice(&data[count * 2..]);
    }

    output
}

fn encode_32bit(data: &[u8]) -> Vec<u8> {
    let count = data.len() / 4;
    let remainder = data.len() % 4;

    let mut output = Vec::with_capacity(HEADER_SIZE + count * 2);

   
    output.extend_from_slice(&MAGIC);
    output.push(4); // width
    output.push(remainder as u8);
    output.extend_from_slice(&[0, 0]);
    output.extend_from_slice(&(count as u64).to_le_bytes());

    if count == 0 {
        output.extend_from_slice(&0i64.to_le_bytes());
        output.extend_from_slice(&data[..remainder]);
        return output;
    }

    let first = i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as i64;
    output.extend_from_slice(&first.to_le_bytes());

   
    let mut prev = first;
    for i in 1..count {
        let off = i * 4;
        let val = i32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as i64;
        let delta = val - prev;
        let zigzag = ((delta << 1) ^ (delta >> 63)) as u64;
        encode_varint(&mut output, zigzag);
        prev = val;
    }

   
    if remainder > 0 {
        output.extend_from_slice(&data[count * 4..]);
    }

    output
}

pub fn decode_numeric_delta(data: &[u8]) -> Vec<u8> {
    if data.len() < HEADER_SIZE {
        return Vec::new();
    }

   
    if &data[0..4] != &MAGIC {
       
        return Vec::new();
    }

    let width = data[4];
    let remainder = data[5] as usize;
    let count = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
    let first = i64::from_le_bytes(data[16..24].try_into().unwrap());

    match width {
        2 => decode_16bit(data, count, first, remainder),
        4 => decode_32bit(data, count, first, remainder),
        _ => decode_8bit(data, count, first),
    }
}

fn decode_8bit(data: &[u8], count: usize, first: i64) -> Vec<u8> {
    if count == 0 {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(count);
    output.push(first as u8);

    let mut pos = HEADER_SIZE;
    let mut prev = first;

    for _ in 1..count {
        if pos >= data.len() {
            break;
        }
        let zigzag = decode_varint(data, &mut pos);
        let delta = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
        let val = prev + delta;
        output.push(val as u8);
        prev = val;
    }

    output
}

fn decode_16bit(data: &[u8], count: usize, first: i64, remainder: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(count * 2 + remainder);

    if count == 0 {
       
        if remainder > 0 && data.len() > HEADER_SIZE {
            output.extend_from_slice(&data[HEADER_SIZE..HEADER_SIZE + remainder.min(data.len() - HEADER_SIZE)]);
        }
        return output;
    }

   
    output.extend_from_slice(&(first as i16).to_le_bytes());

    let mut pos = HEADER_SIZE;
    let mut prev = first;

    for _ in 1..count {
        if pos >= data.len() {
            break;
        }
        let zigzag = decode_varint(data, &mut pos);
        let delta = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
        let val = prev + delta;
        output.extend_from_slice(&(val as i16).to_le_bytes());
        prev = val;
    }

   
    if remainder > 0 && pos < data.len() {
        let remaining = (data.len() - pos).min(remainder);
        output.extend_from_slice(&data[pos..pos + remaining]);
    }

    output
}

fn decode_32bit(data: &[u8], count: usize, first: i64, remainder: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(count * 4 + remainder);

    if count == 0 {
        if remainder > 0 && data.len() > HEADER_SIZE {
            output.extend_from_slice(&data[HEADER_SIZE..HEADER_SIZE + remainder.min(data.len() - HEADER_SIZE)]);
        }
        return output;
    }

   
    output.extend_from_slice(&(first as i32).to_le_bytes());

    let mut pos = HEADER_SIZE;
    let mut prev = first;

    for _ in 1..count {
        if pos >= data.len() {
            break;
        }
        let zigzag = decode_varint(data, &mut pos);
        let delta = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
        let val = prev + delta;
        output.extend_from_slice(&(val as i32).to_le_bytes());
        prev = val;
    }

   
    if remainder > 0 && pos < data.len() {
        let remaining = (data.len() - pos).min(remainder);
        output.extend_from_slice(&data[pos..pos + remaining]);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_8bit_roundtrip() {
        let data: Vec<u8> = (0..255).collect();
        let encoded = encode_numeric_delta(&data);
        let decoded = decode_numeric_delta(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_numeric_16bit_roundtrip() {
       
        let mut data = Vec::new();
        for i in 0..1000u16 {
            data.extend_from_slice(&i.to_le_bytes());
        }
        let encoded = encode_numeric_delta(&data);
        let decoded = decode_numeric_delta(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_numeric_16bit_with_remainder() {
       
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let encoded = encode_numeric_delta(&data);
        let decoded = decode_numeric_delta(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_numeric_32bit_roundtrip() {
        let mut data = Vec::new();
        for i in 0..100i32 {
            data.extend_from_slice(&(i * 1000).to_le_bytes());
        }
        let encoded = encode_numeric_delta(&data);
        let decoded = decode_numeric_delta(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_numeric_empty() {
        let data: Vec<u8> = vec![];
        let encoded = encode_numeric_delta(&data);
        let decoded = decode_numeric_delta(&encoded);
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_numeric_sequential_16bit_compression() {
       
        let mut data = Vec::new();
        for i in 0..10000u16 {
            data.extend_from_slice(&i.to_le_bytes());
        }
        let encoded = encode_numeric_delta(&data);
       
        assert!(encoded.len() < data.len());
        let decoded = decode_numeric_delta(&encoded);
        assert_eq!(decoded, data);
    }
}
