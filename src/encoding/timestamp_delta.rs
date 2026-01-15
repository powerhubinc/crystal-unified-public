//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use super::{write_varint, read_varint_fast};
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;

const FULL_TIMESTAMP: u8 = 0x00;
const DELTA_SMALL_MAX: u8 = 0x7F;
const DELTA_MEDIUM: u8 = 0x80;
const DELTA_LARGE: u8 = 0x81;
const DELTA_NEGATIVE: u8 = 0x82;

pub struct TimestampEncoder {
    last_ts: i64,
    initialized: bool,
}

impl TimestampEncoder {
    pub fn new() -> Self {
        Self {
            last_ts: 0,
            initialized: false,
        }
    }

    #[inline]
    pub fn encode(&mut self, ts: i64, output: &mut Vec<u8>) -> usize {
        let start = output.len();

        if !self.initialized {
           
            output.push(FULL_TIMESTAMP);
            output.write_i64::<LittleEndian>(ts).unwrap();
            self.last_ts = ts;
            self.initialized = true;
        } else {
            let delta = ts - self.last_ts;

            if delta >= 1 && delta <= 127 {
               
                output.push(delta as u8);
            } else if delta >= 0 && delta <= 65535 {
               
                output.push(DELTA_MEDIUM);
                output.write_u16::<LittleEndian>(delta as u16).unwrap();
            } else if delta >= 0 && delta <= i32::MAX as i64 {
               
                output.push(DELTA_LARGE);
                output.write_u32::<LittleEndian>(delta as u32).unwrap();
            } else {
               
                output.push(DELTA_NEGATIVE);
                output.write_i32::<LittleEndian>(delta as i32).unwrap();
            }
            self.last_ts = ts;
        }

        output.len() - start
    }

    pub fn reset(&mut self) {
        self.last_ts = 0;
        self.initialized = false;
    }
}

pub struct TimestampDecoder {
    last_ts: i64,
}

impl TimestampDecoder {
    pub fn new() -> Self {
        Self { last_ts: 0 }
    }

    #[inline]
    pub fn decode(&mut self, input: &[u8]) -> Option<(i64, usize)> {
        if input.is_empty() {
            return None;
        }

        let marker = input[0];

        if marker == FULL_TIMESTAMP {
           
            if input.len() < 9 {
                return None;
            }
            let mut cursor = Cursor::new(&input[1..9]);
            let ts = cursor.read_i64::<LittleEndian>().ok()?;
            self.last_ts = ts;
            Some((ts, 9))
        } else if marker <= DELTA_SMALL_MAX {
           
            let delta = marker as i64;
            self.last_ts += delta;
            Some((self.last_ts, 1))
        } else if marker == DELTA_MEDIUM {
           
            if input.len() < 3 {
                return None;
            }
            let mut cursor = Cursor::new(&input[1..3]);
            let delta = cursor.read_u16::<LittleEndian>().ok()? as i64;
            self.last_ts += delta;
            Some((self.last_ts, 3))
        } else if marker == DELTA_LARGE {
           
            if input.len() < 5 {
                return None;
            }
            let mut cursor = Cursor::new(&input[1..5]);
            let delta = cursor.read_u32::<LittleEndian>().ok()? as i64;
            self.last_ts += delta;
            Some((self.last_ts, 5))
        } else if marker == DELTA_NEGATIVE {
           
            if input.len() < 5 {
                return None;
            }
            let mut cursor = Cursor::new(&input[1..5]);
            let delta = cursor.read_i32::<LittleEndian>().ok()? as i64;
            self.last_ts += delta;
            Some((self.last_ts, 5))
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.last_ts = 0;
    }
}

impl Default for TimestampEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TimestampDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn parse_timestamp(input: &[u8]) -> Option<(i64, usize)> {
   
    if input.len() >= 13 && input[6] == b' ' {
        if let (Some(date), Some(time)) = (
            parse_yymmdd(&input[0..6]),
            parse_hhmmss(&input[7..13])
        ) {
            return Some((date + time, 13));
        }
    }

   
    if input.len() >= 19 && input[4] == b'-' && input[7] == b'-' &&
       input[10] == b' ' && input[13] == b':' && input[16] == b':' {
        if let Some(ts) = parse_iso_timestamp(&input[0..19]) {
            return Some((ts, 19));
        }
    }

   
    let mut i = 0;
    while i < input.len() && input[i].is_ascii_digit() {
        i += 1;
    }
    if i >= 10 && i <= 13 {
       
        let s = std::str::from_utf8(&input[0..i]).ok()?;
        let val: i64 = s.parse().ok()?;
       
        let ms = if val < 1_000_000_000_000 { val * 1000 } else { val };
        return Some((ms, i));
    }

    None
}

#[inline]
fn parse_yymmdd(s: &[u8]) -> Option<i64> {
    if s.len() != 6 || !s.iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let yy = (s[0] - b'0') as i64 * 10 + (s[1] - b'0') as i64;
    let mm = (s[2] - b'0') as i64 * 10 + (s[3] - b'0') as i64;
    let dd = (s[4] - b'0') as i64 * 10 + (s[5] - b'0') as i64;

   
    let year = if yy < 70 { 2000 + yy } else { 1900 + yy };

   
    let days = (year - 1970) * 365 + (year - 1969) / 4 + day_of_year(mm, dd);
    Some(days * 86400 * 1000)
}

#[inline]
fn parse_hhmmss(s: &[u8]) -> Option<i64> {
    if s.len() != 6 || !s.iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let hh = (s[0] - b'0') as i64 * 10 + (s[1] - b'0') as i64;
    let mm = (s[2] - b'0') as i64 * 10 + (s[3] - b'0') as i64;
    let ss = (s[4] - b'0') as i64 * 10 + (s[5] - b'0') as i64;
    Some((hh * 3600 + mm * 60 + ss) * 1000)
}

#[inline]
fn day_of_year(month: i64, day: i64) -> i64 {
    const DAYS: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    if month >= 1 && month <= 12 {
        DAYS[(month - 1) as usize] + day - 1
    } else {
        0
    }
}

#[inline]
fn parse_iso_timestamp(s: &[u8]) -> Option<i64> {
   
    let year: i64 = std::str::from_utf8(&s[0..4]).ok()?.parse().ok()?;
    let month: i64 = std::str::from_utf8(&s[5..7]).ok()?.parse().ok()?;
    let day: i64 = std::str::from_utf8(&s[8..10]).ok()?.parse().ok()?;
    let hour: i64 = std::str::from_utf8(&s[11..13]).ok()?.parse().ok()?;
    let min: i64 = std::str::from_utf8(&s[14..16]).ok()?.parse().ok()?;
    let sec: i64 = std::str::from_utf8(&s[17..19]).ok()?.parse().ok()?;

    let days = (year - 1970) * 365 + (year - 1969) / 4 + day_of_year(month, day);
    Some((days * 86400 + hour * 3600 + min * 60 + sec) * 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_encoding() {
        let mut encoder = TimestampEncoder::new();
        let mut decoder = TimestampDecoder::new();
        let mut buf = Vec::new();

       
        let base = 1703548800000i64;
        encoder.encode(base, &mut buf);
        assert_eq!(buf.len(), 9); // 1 marker + 8 bytes

       
        encoder.encode(base + 50, &mut buf);
        assert_eq!(buf.len(), 10); // +1 byte for small delta

       
        let (ts1, len1) = decoder.decode(&buf).unwrap();
        assert_eq!(ts1, base);
        assert_eq!(len1, 9);

        let (ts2, len2) = decoder.decode(&buf[9..]).unwrap();
        assert_eq!(ts2, base + 50);
        assert_eq!(len2, 1);
    }

    #[test]
    fn test_medium_delta() {
        let mut encoder = TimestampEncoder::new();
        let mut buf = Vec::new();

        let base = 1703548800000i64;
        encoder.encode(base, &mut buf);
        encoder.encode(base + 30000, &mut buf); // 30 second gap

       
        assert_eq!(buf.len(), 9 + 3);
    }

    #[test]
    fn test_timestamp_parsing() {
       
        let (ts, len) = parse_timestamp(b"081109 203518 INFO").unwrap();
        assert_eq!(len, 13);
        assert!(ts > 0);

       
        let (ts2, len2) = parse_timestamp(b"2023-12-26 10:30:45 INFO").unwrap();
        assert_eq!(len2, 19);
        assert!(ts2 > 0);
    }

    #[test]
    fn test_log_sequence() {
        let mut encoder = TimestampEncoder::new();
        let mut buf = Vec::new();

       
        let base = 1703548800000i64;
        for i in 0..100 {
            encoder.encode(base + i * 10, &mut buf); // 10ms apart
        }

       
        assert_eq!(buf.len(), 9 + 99);
    }
}
