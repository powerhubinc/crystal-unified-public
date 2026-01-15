//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use byteorder::{LittleEndian, WriteBytesExt};
use crate::constants::{MAX_TEMPLATE_PARTS, MAX_VAR_RANGES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VarType {
    Raw = 0,
    Integer = 1,
    Float = 2,
    Timestamp = 3,
    IpAddress = 4,
    Uuid = 5,
    Hex = 6,
    Path = 7,
}

pub struct ParseBuffer {
    pub part_ranges: [(u16, u16); MAX_TEMPLATE_PARTS],
    pub var_ranges: [(u16, u16); MAX_VAR_RANGES],
    pub var_types: [VarType; MAX_VAR_RANGES],
    pub part_count: usize,
    pub var_count: usize,
    pub tmpl_buf: Vec<u8>,
    pub overflow: bool,
}

impl ParseBuffer {
    pub fn new() -> Self {
        Self {
            part_ranges: [(0, 0); MAX_TEMPLATE_PARTS],
            var_ranges: [(0, 0); MAX_VAR_RANGES],
            var_types: [VarType::Raw; MAX_VAR_RANGES],
            part_count: 0,
            var_count: 0,
            tmpl_buf: Vec::with_capacity(256),
            overflow: false,
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.part_count = 0;
        self.var_count = 0;
        self.overflow = false;
    }

    #[inline(always)]
    fn add_part(&mut self, start: usize, end: usize) -> bool {
        if start < end {
            if self.part_count >= MAX_TEMPLATE_PARTS {
                self.overflow = true;
                return false;
            }
            self.part_ranges[self.part_count] = (start as u16, end as u16);
            self.part_count += 1;
        }
        true
    }

    #[inline(always)]
    fn add_var_typed(&mut self, start: usize, end: usize, var_type: VarType) -> bool {
        if self.var_count >= MAX_VAR_RANGES || self.part_count >= MAX_TEMPLATE_PARTS {
            self.overflow = true;
            return false;
        }
        self.var_ranges[self.var_count] = (start as u16, end as u16);
        self.var_types[self.var_count] = var_type;
        self.var_count += 1;
       
        self.part_ranges[self.part_count] = (0, 0);
        self.part_count += 1;
        true
    }

    #[inline(always)]
    fn add_var(&mut self, start: usize, end: usize) -> bool {
        self.add_var_typed(start, end, VarType::Raw)
    }

    #[inline]
    fn try_match_uuid(&self, line: &[u8], pos: usize) -> Option<usize> {
        if pos + 36 > line.len() {
            return None;
        }

        let pattern = [8, 4, 4, 4, 12];
        let mut i = pos;

        for (idx, &group_len) in pattern.iter().enumerate() {
            for _ in 0..group_len {
                if i >= line.len() || !line[i].is_ascii_hexdigit() {
                    return None;
                }
                i += 1;
            }
            if idx < 4 {
                if i >= line.len() || line[i] != b'-' {
                    return None;
                }
                i += 1;
            }
        }

       
        if i < line.len() && (line[i].is_ascii_alphanumeric() || line[i] == b'-') {
            return None;
        }

        Some(i - pos)
    }

    #[inline]
    fn try_match_ip(&self, line: &[u8], pos: usize) -> Option<usize> {
        let mut i = pos;
        let mut octets = 0;

        while octets < 4 && i < line.len() {
           
            let start = i;
            while i < line.len() && line[i].is_ascii_digit() && i - start < 3 {
                i += 1;
            }
            if i == start {
                return None;
            }

           
            let mut val: u16 = 0;
            for j in start..i {
                val = val * 10 + (line[j] - b'0') as u16;
            }
            if val > 255 {
                return None;
            }

            octets += 1;
            if octets < 4 {
                if i >= line.len() || line[i] != b'.' {
                    return None;
                }
                i += 1;
            }
        }

        if octets != 4 {
            return None;
        }

       
        if i < line.len() && (line[i].is_ascii_digit() || line[i] == b'.') {
            return None;
        }

        Some(i - pos)
    }

    #[inline]
    fn try_match_hex(&self, line: &[u8], pos: usize) -> Option<usize> {
        // 0x prefix
        if pos + 3 <= line.len() && line[pos] == b'0' && (line[pos + 1] == b'x' || line[pos + 1] == b'X') {
            let mut i = pos + 2;
            while i < line.len() && line[i].is_ascii_hexdigit() {
                i += 1;
            }
            if i > pos + 2 {
               
                if i >= line.len() || !line[i].is_ascii_alphanumeric() {
                    return Some(i - pos);
                }
            }
        }

       
        if pos < line.len() && line[pos].is_ascii_hexdigit() {
            let mut i = pos;
            let mut has_letter = false;
            while i < line.len() && line[i].is_ascii_hexdigit() {
                if line[i].is_ascii_alphabetic() {
                    has_letter = true;
                }
                i += 1;
            }
           
            if i - pos >= 8 && has_letter {
                if i >= line.len() || !line[i].is_ascii_alphanumeric() {
                    return Some(i - pos);
                }
            }
        }

        None
    }

    #[inline]
    fn try_match_timestamp(&self, line: &[u8], pos: usize) -> Option<usize> {
       
        if pos + 13 <= line.len() && line[pos + 6] == b' ' {
            let date_ok = line[pos..pos+6].iter().all(|b| b.is_ascii_digit());
            let time_ok = line[pos+7..pos+13].iter().all(|b| b.is_ascii_digit());
            if date_ok && time_ok {
               
                if pos + 13 >= line.len() || !line[pos + 13].is_ascii_digit() {
                    return Some(13);
                }
            }
        }

       
        if pos + 19 <= line.len() &&
           line[pos + 4] == b'-' && line[pos + 7] == b'-' &&
           line[pos + 10] == b' ' && line[pos + 13] == b':' && line[pos + 16] == b':' {
            let digits_ok = [0,1,2,3,5,6,8,9,11,12,14,15,17,18]
                .iter().all(|&i| line[pos + i].is_ascii_digit());
            if digits_ok {
               
                if pos + 19 >= line.len() || !line[pos + 19].is_ascii_digit() {
                    return Some(19);
                }
            }
        }

        None
    }

    #[inline]
    fn try_match_number(&self, line: &[u8], pos: usize) -> Option<usize> {
        let mut i = pos;

       
        if i < line.len() && line[i] == b'-' {
            if i > 0 && line[i - 1].is_ascii_alphanumeric() {
                return None;
            }
            i += 1;
        }

       
        if i >= line.len() || !line[i].is_ascii_digit() {
            return None;
        }

       
        while i < line.len() && line[i].is_ascii_digit() {
            i += 1;
        }

       
        if i < line.len() && line[i] == b'.' {
            let dot_pos = i;
            i += 1;
            let frac_start = i;
            while i < line.len() && line[i].is_ascii_digit() {
                i += 1;
            }
           
            if i == frac_start {
                i = dot_pos;
            }
        }

       
        if i < line.len() && (line[i] == b'e' || line[i] == b'E') {
            let exp_pos = i;
            i += 1;
            if i < line.len() && (line[i] == b'+' || line[i] == b'-') {
                i += 1;
            }
            let exp_start = i;
            while i < line.len() && line[i].is_ascii_digit() {
                i += 1;
            }
           
            if i == exp_start {
                i = exp_pos;
            }
        }

        let consumed = i - pos;
       
        if consumed >= 1 {
            if i >= line.len() || !line[i].is_ascii_alphanumeric() {
                return Some(consumed);
            }
        }

        None
    }

    #[inline]
    fn try_match_path(&self, line: &[u8], pos: usize) -> Option<usize> {
       
        if pos + 2 <= line.len() && line[pos] == b'/' {
            let next = line[pos + 1];
            if next.is_ascii_alphanumeric() || next == b'.' || next == b'_' {
                let mut i = pos + 1;
                while i < line.len() {
                    let c = line[i];
                    if c.is_ascii_alphanumeric() || c == b'/' || c == b'.' || c == b'_' || c == b'-' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                if i - pos >= 4 { 
                    return Some(i - pos);
                }
            }
        }

       
        if pos + 3 <= line.len() && line[pos].is_ascii_alphabetic() && line[pos + 1] == b':' && line[pos + 2] == b'\\' {
            let mut i = pos + 3;
            while i < line.len() {
                let c = line[i];
                if c.is_ascii_alphanumeric() || c == b'\\' || c == b'/' || c == b'.' || c == b'_' || c == b'-' {
                    i += 1;
                } else {
                    break;
                }
            }
            if i - pos >= 5 { 
                return Some(i - pos);
            }
        }

        None
    }

    #[inline]
    pub fn parse_line(&mut self, line: &[u8]) {
        self.clear();
        let len = line.len();

       
        if len > 65535 {
            self.overflow = true;
            return;
        }

        let mut i = 0usize;
        let mut part_start = 0usize;

        while i < len {
           
            let match_result = self.try_match_timestamp(line, i).map(|len| (len, VarType::Timestamp))
                .or_else(|| self.try_match_uuid(line, i).map(|len| (len, VarType::Uuid)))
                .or_else(|| self.try_match_ip(line, i).map(|len| (len, VarType::IpAddress)))
                .or_else(|| self.try_match_hex(line, i).map(|len| (len, VarType::Hex)))
                .or_else(|| self.try_match_number(line, i).map(|len| {
                   
                    let var_bytes = &line[i..i+len];
                    let is_float = var_bytes.contains(&b'.') || var_bytes.contains(&b'e') || var_bytes.contains(&b'E');
                    (len, if is_float { VarType::Float } else { VarType::Integer })
                }))
                .or_else(|| self.try_match_path(line, i).map(|len| (len, VarType::Path)));

            if let Some((consumed, var_type)) = match_result {
               
                if !self.add_part(part_start, i) {
                    return;
                }

               
                if !self.add_var_typed(i, i + consumed, var_type) {
                    return;
                }

                i += consumed;
                part_start = i;
            } else {
                i += 1;
            }
        }

       
        self.add_part(part_start, len);
    }

    #[inline]
    pub fn build_template(&mut self, line: &[u8]) -> &[u8] {
        self.tmpl_buf.clear();
        self.tmpl_buf.write_u16::<LittleEndian>(self.part_count as u16).unwrap();

        for i in 0..self.part_count {
            let (start, end) = self.part_ranges[i];
            let part_len = (end - start) as u16;
            self.tmpl_buf.write_u16::<LittleEndian>(part_len).unwrap();
            if part_len > 0 {
                self.tmpl_buf.extend_from_slice(&line[start as usize..end as usize]);
            }
        }

        &self.tmpl_buf
    }
}

impl Default for ParseBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numbers() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"error code 123 at line 456");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 2);
    }

    #[test]
    fn test_parse_ip() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"connect from 192.168.1.100 to server");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"connect from 192.168.1.100 to server"[start as usize..end as usize], b"192.168.1.100");
    }

    #[test]
    fn test_parse_uuid() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"request 550e8400-e29b-41d4-a716-446655440000 completed");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"request 550e8400-e29b-41d4-a716-446655440000 completed"[start as usize..end as usize],
                   b"550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_parse_hex() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"address 0x1a2b3c4d allocated");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"address 0x1a2b3c4d allocated"[start as usize..end as usize], b"0x1a2b3c4d");
    }

    #[test]
    fn test_parse_path() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"reading /var/log/syslog file");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"reading /var/log/syslog file"[start as usize..end as usize], b"/var/log/syslog");
    }

    #[test]
    fn test_parse_mixed() {
        let mut buf = ParseBuffer::new();
        let line = b"2023-01-15 10:30:45 INFO [192.168.1.1] Request 550e8400-e29b-41d4-a716-446655440000 processed in 123ms";
        buf.parse_line(line);
        assert!(!buf.overflow);
       
       
        assert!(buf.var_count >= 3, "Expected >= 3 variables, got {}", buf.var_count);
    }

    #[test]
    fn test_parse_float() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"temperature is 23.5 degrees");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"temperature is 23.5 degrees"[start as usize..end as usize], b"23.5");
    }

    #[test]
    fn test_parse_negative() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"offset is -100 bytes");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"offset is -100 bytes"[start as usize..end as usize], b"-100");
    }

    #[test]
    fn test_no_variables() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"hello world");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 0);
        assert_eq!(buf.part_count, 1);
    }

    #[test]
    fn test_long_hex() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"hash: deadbeef12345678 computed");
        assert!(!buf.overflow);
        assert_eq!(buf.var_count, 1);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"hash: deadbeef12345678 computed"[start as usize..end as usize], b"deadbeef12345678");
    }

    #[test]
    fn test_parse_timestamp_hdfs() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"081109 203518 143 INFO dfs.DataNode");
        assert!(!buf.overflow);
        assert!(buf.var_count >= 2);
        assert_eq!(buf.var_types[0], VarType::Timestamp);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"081109 203518 143 INFO dfs.DataNode"[start as usize..end as usize], b"081109 203518");
    }

    #[test]
    fn test_parse_timestamp_iso() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"2023-12-26 10:30:45 INFO server started");
        assert!(!buf.overflow);
        assert_eq!(buf.var_types[0], VarType::Timestamp);
        let (start, end) = buf.var_ranges[0];
        assert_eq!(&b"2023-12-26 10:30:45 INFO server started"[start as usize..end as usize], b"2023-12-26 10:30:45");
    }

    #[test]
    fn test_var_types() {
        let mut buf = ParseBuffer::new();
        buf.parse_line(b"192.168.1.1 sent 123.45 bytes at /var/log");
        assert!(!buf.overflow);
        assert_eq!(buf.var_types[0], VarType::IpAddress);
        assert_eq!(buf.var_types[1], VarType::Float);
        assert_eq!(buf.var_types[2], VarType::Path);
    }
}
