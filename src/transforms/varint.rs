//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md

#[inline]
pub fn encode_varint(output: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        output.push((value as u8) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

#[inline]
pub fn decode_varint(data: &[u8], pos: &mut usize) -> u64 {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        if *pos >= data.len() {
            break;
        }
        let b = data[*pos];
        *pos += 1;
        result |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_small() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, 127);
        assert_eq!(buf.len(), 1);
        let mut pos = 0;
        let decoded = decode_varint(&buf, &mut pos);
        assert_eq!(decoded, 127);
    }

    #[test]
    fn test_varint_large() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, 16384);
        assert_eq!(buf.len(), 3);
        let mut pos = 0;
        let decoded = decode_varint(&buf, &mut pos);
        assert_eq!(decoded, 16384);
    }

    #[test]
    fn test_varint_max() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, u64::MAX);
        let mut pos = 0;
        let decoded = decode_varint(&buf, &mut pos);
        assert_eq!(decoded, u64::MAX);
    }
}
