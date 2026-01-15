//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

pub fn group_varint_encode(values: &[u32; 4]) -> (u8, [u8; 16], usize) {
    let mut output = [0u8; 16];
    let mut control = 0u8;
    let mut pos = 0usize;

    for (i, &val) in values.iter().enumerate() {
        let bytes_needed = match val {
            0..=0xFF => 1,
            0x100..=0xFFFF => 2,
            0x1_0000..=0xFF_FFFF => 3,
            _ => 4,
        };
        control |= ((bytes_needed - 1) as u8) << (i * 2);
        let le_bytes = val.to_le_bytes();
        output[pos..pos + bytes_needed].copy_from_slice(&le_bytes[..bytes_needed]);
        pos += bytes_needed;
    }

    (control, output, pos)
}

pub fn group_varint_decode(control: u8, data: &[u8]) -> ([u32; 4], usize) {
    let mut values = [0u32; 4];
    let mut pos = 0usize;

    for i in 0..4 {
        let bytes_needed = (((control >> (i * 2)) & 3) + 1) as usize;
        let mut buf = [0u8; 4];
        buf[..bytes_needed].copy_from_slice(&data[pos..pos + bytes_needed]);
        values[i] = u32::from_le_bytes(buf);
        pos += bytes_needed;
    }

    (values, pos)
}

#[inline]
pub fn write_varint(w: &mut Vec<u8>, mut val: u64) -> std::io::Result<()> {
    while val >= 0x80 {
        w.push((val as u8) | 0x80);
        val >>= 7;
    }
    w.push(val as u8);
    Ok(())
}

#[inline]
pub fn read_varint_fast(data: &[u8], pos: &mut usize) -> u64 {
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
    fn test_group_varint_roundtrip() {
        let values = [100u32, 1000, 100000, 1000000];
        let (control, data, len) = group_varint_encode(&values);
        let (decoded, bytes_read) = group_varint_decode(control, &data);
        assert_eq!(values, decoded);
        assert_eq!(len, bytes_read);
    }

    #[test]
    fn test_varint_roundtrip() {
        let mut buf = Vec::new();
        write_varint(&mut buf, 12345).unwrap();
        let mut pos = 0;
        let decoded = read_varint_fast(&buf, &mut pos);
        assert_eq!(decoded, 12345);
    }
}

#[inline]
pub unsafe fn read_varint_unchecked(data: &[u8], pos: &mut usize) -> u64 {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let b = *data.get_unchecked(*pos);
        *pos += 1;
        result |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    result
}
