//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

#[inline]
pub fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }

    let first = needle[0];
    let mut pos = 0;

    while pos + needle.len() <= haystack.len() {
       
        if let Some(idx) = memchr::memchr(first, &haystack[pos..]) {
            let start = pos + idx;
            if start + needle.len() > haystack.len() {
                return false;
            }
           
            if &haystack[start..start + needle.len()] == needle {
                return true;
            }
            pos = start + 1;
        } else {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_bytes_found() {
        assert!(contains_bytes(b"hello world", b"world"));
        assert!(contains_bytes(b"hello world", b"hello"));
        assert!(contains_bytes(b"hello world", b"lo wo"));
    }

    #[test]
    fn test_contains_bytes_not_found() {
        assert!(!contains_bytes(b"hello world", b"xyz"));
        assert!(!contains_bytes(b"hello", b"hello world"));
    }

    #[test]
    fn test_contains_bytes_empty() {
        assert!(contains_bytes(b"hello", b""));
        assert!(!contains_bytes(b"", b"hello"));
    }
}
