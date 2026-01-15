//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

pub fn train_dictionary(samples: &[Vec<u8>], dict_size: usize) -> Option<Vec<u8>> {
    if samples.is_empty() {
        return None;
    }

   
    let total_size: usize = samples.iter().map(|s| s.len()).sum();
    let mut sample_data = Vec::with_capacity(total_size);
    let mut sample_sizes: Vec<usize> = Vec::with_capacity(samples.len());

    for sample in samples {
        sample_data.extend_from_slice(sample);
        sample_sizes.push(sample.len());
    }

   
    match zstd::dict::from_continuous(&sample_data, &sample_sizes, dict_size) {
        Ok(trained) => Some(trained),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_train_dictionary() {
        let samples: Vec<Vec<u8>> = (0..100)
            .map(|i| format!("log entry {} with some common text pattern", i).into_bytes())
            .collect();

        let dict = train_dictionary(&samples, 1024);
        assert!(dict.is_some());
        assert!(dict.unwrap().len() > 0);
    }

    #[test]
    fn test_train_dictionary_empty() {
        let samples: Vec<Vec<u8>> = vec![];
        let dict = train_dictionary(&samples, 1024);
        assert!(dict.is_none());
    }
}
