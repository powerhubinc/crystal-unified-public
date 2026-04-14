//! Crystal Unified Compression Library
//!
//! Copyright (c) 2026 Powerhub Inc. All rights reserved.
//! Patent Pending.
//!
//! Licensed under the Business Source License 1.1 (BSL-1.1).
//! See: https://github.com/powerhubinc/crystal-unified-public/blob/main/LICENSES/LICENSE.md
//!

use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crystal_unified::{compress_with_options, compress_parallel_with_options, compress_ultra_fast, decompress, CompressionOptions};
use crystal_unified::{TRANSFORM_NONE, TRANSFORM_DNA_2BIT, TRANSFORM_NUMERIC_DELTA, TRANSFORM_BINARY_DELTA, TRANSFORM_NIBBLE_SPLIT, TRANSFORM_STRUCTURED};
use crystal_unified::{smart_detect, would_bloat};
use crystal_unified::{SMALL_FILE_THRESHOLD, TINY_FILE_THRESHOLD};
use crystal_unified::CrystalReaderV10;
use crystal_unified::{ReferenceIndex, encode_dna_with_reference, decode_dna_with_reference};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Crystal Unified v1.0 - High-Performance Searchable Compression");
        eprintln!();
        eprintln!("Usage: cuz <command> [options]");
        eprintln!();
        eprintln!("Compression:");
        eprintln!("  c, compress   <input> [output] [-l level] [-t transform]");
        eprintln!("  d, decompress <input> [output]");
        eprintln!("  auto          <input> [output]   Smart auto-detect & compress");
        eprintln!("  a, analyze    <input>            Detect data type");
        eprintln!();
        eprintln!("Search & Append:");
        eprintln!("  s, search     <file.cuz> <term>  Search in compressed file");
        eprintln!("  append        <file.cuz> <data>  Append new lines to archive");
        eprintln!();
        eprintln!("Multi-file archive:");
        eprintln!("  ar, archive   <output.cuz> <file1> [file2...]");
        eprintln!("  ls, list      <archive.cuz>");
        eprintln!("  x, extract    <archive.cuz> [output_dir]");
        eprintln!();
        eprintln!("Firmware/binary (.cuzb):");
        eprintln!("  fw, firmware  <input> [output] [-l level] [-b block_size]");
        eprintln!("  append-raw    <file.cuzb> <data>");
        eprintln!("  patch         <file.cuzb> <block#> <data>");
        eprintln!("  delta         <old> <new> -o <patch.cuzd>");
        eprintln!("  apply         <file> <patch.cuzd> [-o output]");
        eprintln!();
        eprintln!("DNA reference compression:");
        eprintln!("  dna-index     <reference.fa> <output.cdni>  Build reference index");
        eprintln!("  dna-compress  <input.fa> -r <ref.cdni> [-o output]");
        eprintln!("  dna-decompress <input.cdnr> -r <ref.cdni> [-o output]");
        eprintln!();
        eprintln!();
        eprintln!("Similarity search:");
        eprintln!("  similar       <file.cuz>  (check if archive has vector index)");
        eprintln!();
        eprintln!("Transforms: none, dna, numeric, binary, nibble, struct");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "c" | "compress" => compress_cmd(&args[2..]),
        "similar" | "sim" => similar_cmd(&args[2..]),
        "d" | "decompress" => decompress_cmd(&args[2..]),
        "a" | "analyze" => analyze_cmd(&args[2..]),
        "auto" => auto_compress_cmd(&args[2..]),
        "s" | "search" => search_cmd(&args[2..]),
        "append" => append_cmd(&args[2..]),
        "ar" | "archive" => archive_cmd(&args[2..]),
        "ls" | "list" => list_archive_cmd(&args[2..]),
        "x" | "extract" => extract_cmd(&args[2..]),
        "fw" | "firmware" => firmware_cmd(&args[2..]),
        "append-raw" => append_raw_cmd(&args[2..]),
        "patch" => patch_cmd(&args[2..]),
        "delta" => delta_cmd(&args[2..]),
        "apply" => apply_cmd(&args[2..]),
        "dna-index" => dna_index_cmd(&args[2..]),
        "dna-compress" => dna_compress_cmd(&args[2..]),
        "dna-decompress" => dna_decompress_cmd(&args[2..]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn compress_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz compress <input> [output] [-l level] [-t transform] [-j] [-s]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut level = 1;
    let mut output_path = format!("{}.cuz", input_path);
    let mut transform: Option<u8> = None;
    let mut dict_size: Option<usize> = None;
    let mut parallel = false;
    let mut block_size: Option<u64> = None;
    let mut streaming = false;
    let mut fast_mode = false;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-l" && i + 1 < args.len() {
            level = args[i + 1].parse().unwrap_or(1);
            i += 2;
        } else if args[i] == "-t" && i + 1 < args.len() {
            transform = Some(match args[i + 1].as_str() {
                "none" => TRANSFORM_NONE,
                "dna" => TRANSFORM_DNA_2BIT,
                "numeric" => TRANSFORM_NUMERIC_DELTA,
                "binary" => TRANSFORM_BINARY_DELTA,
                "nibble" => TRANSFORM_NIBBLE_SPLIT,
                "struct" => TRANSFORM_STRUCTURED,
                _ => {
                    eprintln!("Unknown transform: {}", args[i + 1]);
                    std::process::exit(1);
                }
            });
            i += 2;
        } else if args[i] == "-d" && i + 1 < args.len() {
            dict_size = Some(args[i + 1].parse::<usize>().unwrap_or(128) * 1024);
            i += 2;
        } else if args[i] == "-o" && i + 1 < args.len() {
            output_path = args[i + 1].clone();
            i += 2;
        } else if args[i] == "-j" {
            parallel = true;
            i += 1;
        } else if args[i] == "-b" && i + 1 < args.len() {
            block_size = Some(args[i + 1].parse::<u64>().unwrap_or(16384));
            i += 2;
        } else if args[i] == "--fast" {
            fast_mode = true;
            i += 1;
        } else if args[i] == "-s" {
            streaming = true;
            i += 1;
        } else {
            output_path = args[i].clone();
            i += 1;
        }
    }

    let data = fs::read(input_path).expect("Failed to read input file");
    let mut options = CompressionOptions::level(level);
    if fast_mode {
        options = options.with_fast_mode(true);
    }

    if let Some(t) = transform {
        options = options.with_transform(t).with_raw_mode(true);
    }
    if let Some(d) = dict_size {
        options = options.with_dictionary(true).with_dictionary_size(d);
    }
    if let Some(b) = block_size {
        options = options.with_block_size(b);
    }
    let start = Instant::now();
    let compressed = if streaming && transform.is_none() {
       
        crystal_unified::compress_streaming_with_options(&data, options)
    } else if parallel && transform.is_none() {
       
        compress_parallel_with_options(&data, options)
    } else {
        compress_with_options(&data, options)
    };
    let elapsed = start.elapsed();

    fs::write(&output_path, &compressed).expect("Failed to write output file");

    let ratio = compressed.len() as f64 / data.len() as f64 * 100.0;
    let speed = data.len() as f64 / 1_000_000.0 / elapsed.as_secs_f64();
    println!("{} -> {} ({:.2}%, {:.1} MB/s)", input_path, output_path, ratio, speed);
}

fn decompress_cmd(args: &[String]) {
    use std::io::{BufWriter, Write};
    use std::sync::mpsc;
    use std::thread;
    use rayon::prelude::*;
    use crystal_unified::BlockRawReader;

    if args.is_empty() {
        eprintln!("Usage: cuz decompress <input> [output]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut output_path = input_path.trim_end_matches(".cuz").trim_end_matches(".cuzb").to_string();

   
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            output_path = args[i + 1].clone();
            i += 2;
        } else if !args[i].starts_with('-') {
            output_path = args[i].clone();
            i += 1;
        } else {
            i += 1;
        }
    }

    let data = fs::read(input_path).expect("Failed to read input file");
    let start = Instant::now();

   
    if BlockRawReader::is_block_raw_format(&data) {
        let reader = BlockRawReader::new(&data).expect("Failed to parse CUZB archive");
        let decompressed = reader.decompress_all().expect("Failed to decompress");
        let size = decompressed.len();
        fs::write(&output_path, &decompressed).expect("Failed to write");
        let elapsed = start.elapsed();
        let speed = size as f64 / 1_000_000.0 / elapsed.as_secs_f64();
        println!("{} -> {} ({} bytes, {:.1} MB/s)", input_path, output_path, size, speed);
        return;
    }

   
    let reader = match CrystalReaderV10::new(&data) {
        Ok(r) => r,
        Err(_) => {
           
            let decompressed = decompress(&data).expect("Failed to decompress");
            fs::write(&output_path, &decompressed).expect("Failed to write");
            println!("Decompressed {} -> {} ({} bytes)", input_path, output_path, decompressed.len());
            return;
        }
    };

   
    if reader.is_streaming_mode() {
        let decompressed = reader.reconstruct();
        let size = decompressed.len();
        fs::write(&output_path, &decompressed).expect("Failed to write");
        let elapsed = start.elapsed();
        let speed = size as f64 / 1_000_000.0 / elapsed.as_secs_f64();
        println!("{} -> {} ({} bytes, {:.1} MB/s)", input_path, output_path, size, speed);
        return;
    }

    let block_count = reader.block_count() as usize;
    let has_trailing_newline = reader.has_trailing_newline();

    let (tx, rx) = mpsc::sync_channel::<(usize, Vec<u8>)>(16);
    let out_path = output_path.clone();
    let writer_handle = thread::spawn(move || {
        let file = fs::File::create(&out_path).expect("Failed to create output file");
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);
        let mut total_written = 0usize;
        let mut next_block = 0usize;
        let mut pending: std::collections::BTreeMap<usize, Vec<u8>> = std::collections::BTreeMap::new();

        while let Ok((block_idx, data)) = rx.recv() {
            pending.insert(block_idx, data);

           
            while let Some(chunk) = pending.remove(&next_block) {
                writer.write_all(&chunk).expect("Failed to write");
                total_written += chunk.len();
                next_block += 1;
            }
        }

        writer.flush().expect("Failed to flush");
        total_written
    });

   
    let tx_clone = tx.clone();
    let decompressed_size: usize = (0..block_count)
        .into_par_iter()
        .map(|block_idx| {
            let block_data = reader.decompress_block_bytes(block_idx as u64).unwrap_or_default();
            let size = block_data.len();
            tx_clone.send((block_idx, block_data)).expect("Failed to send block");
            size
        })
        .sum();

    drop(tx_clone);
    drop(tx);

    let written = writer_handle.join().expect("Writer thread panicked");

    // Fix trailing newline: if original file did not have trailing newline, remove it
    let final_size = if !has_trailing_newline && written > 0 {
        let file = fs::OpenOptions::new()
            .write(true)
            .open(&output_path)
            .expect("Failed to open output file for truncation");
        file.set_len((written - 1) as u64).expect("Failed to truncate file");
        written - 1
    } else {
        written
    };

    let total_elapsed = start.elapsed();
    let total_speed = decompressed_size as f64 / 1_000_000.0 / total_elapsed.as_secs_f64();
    println!("{} -> {} ({} bytes, {:.1} MB/s)", input_path, output_path, final_size, total_speed);
}

fn analyze_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz analyze <input>");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let data = fs::read(input_path).expect("Failed to read input file");
    let detection = smart_detect(&data);
    println!("{}: {:?} -> {} ({})", input_path, detection.data_type, transform_name(detection.transform), detection.reason);
}

fn transform_name(id: u8) -> &'static str {
    match id {
        TRANSFORM_NONE => "None (direct)",
        TRANSFORM_DNA_2BIT => "DNA 2-bit",
        TRANSFORM_NUMERIC_DELTA => "Numeric Delta",
        TRANSFORM_BINARY_DELTA => "Binary Delta",
        TRANSFORM_NIBBLE_SPLIT => "Nibble Split",
        TRANSFORM_STRUCTURED => "Structured",
        _ => "Unknown",
    }
}

fn auto_compress_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz auto <input> [output] [-j] [--text]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut output_path = format!("{}.cuz", input_path);
    let mut parallel = false;
    let mut force_text = false;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-j" {
            parallel = true;
            i += 1;
        } else if args[i] == "--text" || args[i] == "-t" {
            force_text = true;
            i += 1;
        } else {
            output_path = args[i].clone();
            i += 1;
        }
    }

    let data = fs::read(input_path).expect("Failed to read input file");
    let file_size = data.len();

   
    let mut detection = smart_detect(&data);

    if force_text {
        detection.transform = TRANSFORM_NONE;
    }

   
    if file_size <= TINY_FILE_THRESHOLD {
        fs::write(&output_path, &data).expect("Failed to write output file");
        println!("{} -> {} (stored, too small)", input_path, output_path);
        return;
    }

    let options = detection.to_options();
    let start = Instant::now();
    // Always use searchable compression (CUZ1 format) - never use compress_ultra_fast (CUZR)
    // which doesn't include search indexes
    let compressed = if parallel {
        compress_parallel_with_options(&data, options)
    } else {
        compress_with_options(&data, options)
    };
    let elapsed = start.elapsed();

   
    if would_bloat(file_size, compressed.len()) {
        fs::write(&output_path, &data).expect("Failed to write output file");
        println!("{} -> {} (stored, incompressible)", input_path, output_path);
        return;
    }

    fs::write(&output_path, &compressed).expect("Failed to write output file");
    let ratio = compressed.len() as f64 / file_size as f64 * 100.0;
    let speed = file_size as f64 / 1_000_000.0 / elapsed.as_secs_f64();
    println!("{} -> {} ({:.2}%, {:.1} MB/s)", input_path, output_path, ratio, speed);
}

fn search_cmd(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: cuz search <file.cuz> <term> [-n max] [--count]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let search_term = &args[1];
    let mut max_results: usize = 100;
    let mut count_only = false;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            max_results = args[i + 1].parse().unwrap_or(100);
            i += 2;
        } else if args[i] == "--count" || args[i] == "-c" {
            count_only = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if data.len() < 4 || &data[0..4] != b"CUZ1" {
        eprintln!("Error: not a searchable .cuz file");
        std::process::exit(1);
    }

    let reader = match CrystalReaderV10::new(&data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    };

    let results = reader.search_parallel(search_term.as_bytes());

    if count_only {
        println!("{} matches", results.len());
        return;
    }

    if results.is_empty() {
        println!("No matches found.");
    } else {
        for (i, line) in results.iter().take(max_results).enumerate() {
            let highlighted = highlight_term(line, search_term);
            println!("{:5}: {}", i + 1, highlighted);
        }
        if results.len() > max_results {
            println!("... {} more", results.len() - max_results);
        }
    }
}

fn similar_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz similar <file.cuz>");
        eprintln!();
        eprintln!("Similarity search requires a vector index embedded via the library API.");
        eprintln!("Use embed_archive() to attach embeddings, then search_similar() with");
        eprintln!("a query embedding vector.");
        std::process::exit(1);
    }

    let input_path = &args[0];

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let reader = match CrystalReaderV10::new(&data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    };

    if !reader.has_jl_sketch() {
        eprintln!("Archive has no vector index.");
        eprintln!("Use embed_archive() in the library API to attach embeddings.");
        std::process::exit(1);
    }

    println!("Archive has vector index: dim={}, bits={}, blocks={}",
        reader.sketch_dim(), reader.sketch_bits(), reader.block_count());
    println!("Use the library API (search_similar / search_similar_top_k) with");
    println!("a query embedding vector to search.");
}

fn highlight_term(line: &str, term: &str) -> String {
    let lower_line = line.to_lowercase();
    let lower_term = term.to_lowercase();

    if let Some(pos) = lower_line.find(&lower_term) {
       
        let before = &line[..pos];
        let matched = &line[pos..pos + term.len()];
        let after = &line[pos + term.len()..];
        format!("{}\x1b[1;33m{}\x1b[0m{}", before, matched, after)
    } else {
        line.to_string()
    }
}

fn append_cmd(args: &[String]) {
    use crystal_unified::decompress::CrystalReaderV10;
    use crystal_unified::hash::{BloomFilter, TrigramBloom};
    use crystal_unified::block::{BlockData, ParseBuffer};
    use crystal_unified::compress::compress_no_dict;
    use crystal_unified::constants::{
        MAGIC, VERSION, HEADER_SIZE, BLOCK_INDEX_ENTRY_SIZE, BLOOM_SIZE_BYTES,
        TRIGRAM_BLOOM_SIZE_BYTES, FLAG_TRAILING_NEWLINE, FLAG_HAS_TRIGRAMS, FLAG_FAST_MODE,
    };
    use byteorder::{LittleEndian, WriteBytesExt};

    if args.len() < 2 {
        eprintln!("Usage: cuz append <archive.cuz> <data|->");
        std::process::exit(1);
    }

    let archive_path = &args[0];
    let new_data_path = &args[1];

   
    let archive_data = match fs::read(archive_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read archive {}: {}", archive_path, e);
            std::process::exit(1);
        }
    };

   
    let reader = match CrystalReaderV10::new(&archive_data) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to parse archive: {:?}", e);
            std::process::exit(1);
        }
    };

   
    let new_data = if new_data_path == "-" {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf).expect("Failed to read stdin");
        buf
    } else {
        match fs::read(new_data_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to read new data {}: {}", new_data_path, e);
                std::process::exit(1);
            }
        }
    };

    if new_data.is_empty() {
        println!("No new data to append.");
        return;
    }

    let start = Instant::now();

   
    let old_block_count = reader.block_count() as usize;
    let old_row_count = reader.row_count();
    let block_size = reader.block_size() as usize;
    let use_trigrams = reader.has_trigrams();
    let fast_mode = reader.is_fast_mode();
    let level = reader.compression_level();
    let trailing_newline = new_data.last() == Some(&b'\n');

    let mut all_blocks: Vec<(Vec<u8>, BloomFilter, Option<TrigramBloom>, usize)> = Vec::new();

   
    for i in 0..old_block_count {
        let (raw_data, decompressed_size) = reader.get_raw_block(i as u64).unwrap();
        let bloom = reader.get_block_bloom(i as u64);
        let trigram = reader.get_block_trigram(i as u64);
        all_blocks.push((raw_data.to_vec(), bloom, trigram, decompressed_size));
    }

   
    let content = if trailing_newline { &new_data[..new_data.len() - 1] } else { &new_data[..] };
    let mut current_block = BlockData::with_fast_mode(use_trigrams, fast_mode);
    let mut parse_buf = ParseBuffer::new();
    let mut new_row_count = 0u64;

    let mut line_start = 0;
    for (i, &byte) in content.iter().enumerate() {
        if byte == b'\n' {
            let line = &content[line_start..i];
            current_block.add_line(line, &mut parse_buf);
            new_row_count += 1;
            line_start = i + 1;

            if current_block.row_count() >= block_size {
                let raw = if fast_mode { current_block.serialize_fast() } else { current_block.serialize() };
                let decompressed_size = raw.len();
                let compressed = compress_no_dict(&raw, level);
                all_blocks.push((compressed, current_block.bloom, current_block.trigram_bloom, decompressed_size));
                current_block = BlockData::with_fast_mode(use_trigrams, fast_mode);
            }
        }
    }

   
    if line_start < content.len() {
        let line = &content[line_start..];
        current_block.add_line(line, &mut parse_buf);
        new_row_count += 1;
    }

   
    if current_block.row_count() > 0 {
        let raw = if fast_mode { current_block.serialize_fast() } else { current_block.serialize() };
        let decompressed_size = raw.len();
        let compressed = compress_no_dict(&raw, level);
        all_blocks.push((compressed, current_block.bloom, current_block.trigram_bloom, decompressed_size));
    }

   
    let total_row_count = old_row_count + new_row_count;
    let block_count = all_blocks.len();
    let new_block_count = block_count - old_block_count;

   
    let index_size = block_count * BLOCK_INDEX_ENTRY_SIZE;
    let bloom_size = block_count * BLOOM_SIZE_BYTES;
    let trigram_size = if use_trigrams { block_count * TRIGRAM_BLOOM_SIZE_BYTES } else { 0 };
    let data_size: usize = all_blocks.iter().map(|(d, _, _, _)| d.len()).sum();

    let mut output = Vec::with_capacity(HEADER_SIZE + index_size + bloom_size + trigram_size + data_size);

   
    output.extend_from_slice(MAGIC);
    output.write_u32::<LittleEndian>(VERSION).unwrap();
    output.write_u64::<LittleEndian>(block_size as u64).unwrap();
    output.write_u64::<LittleEndian>(total_row_count).unwrap();
    output.write_u64::<LittleEndian>(block_count as u64).unwrap();

    let mut flags = if trailing_newline { FLAG_TRAILING_NEWLINE } else { 0 };
    if use_trigrams { flags |= FLAG_HAS_TRIGRAMS; }
    if fast_mode { flags |= FLAG_FAST_MODE; }
    output.write_u64::<LittleEndian>(flags).unwrap();
    output.write_i32::<LittleEndian>(level).unwrap();
    output.write_u64::<LittleEndian>(0).unwrap();
    output.write_u64::<LittleEndian>(0).unwrap();

    while output.len() < HEADER_SIZE { output.push(0); }

   
    let mut current_offset = 0u64;
    for (data, _, _, decompressed_size) in &all_blocks {
        output.write_u64::<LittleEndian>(current_offset).unwrap();
        output.write_u64::<LittleEndian>(data.len() as u64).unwrap();
        output.write_u64::<LittleEndian>(*decompressed_size as u64).unwrap();
        current_offset += data.len() as u64;
    }

   
    for (_, bloom, _, _) in &all_blocks {
        for &v in bloom { output.write_u64::<LittleEndian>(v).unwrap(); }
    }

   
    if use_trigrams {
        for (_, _, trigram, _) in &all_blocks {
            if let Some(ref tb) = trigram {
                for &v in tb { output.write_u64::<LittleEndian>(v).unwrap(); }
            }
        }
    }

   
    for (data, _, _, _) in all_blocks {
        output.extend_from_slice(&data);
    }

   
    fs::write(archive_path, &output).expect("Failed to write archive");

    let elapsed = start.elapsed();
    let speed = new_data.len() as f64 / 1_000_000.0 / elapsed.as_secs_f64();

    println!("{}: +{} rows ({} total, {:.1} MB/s)", archive_path, new_row_count, total_row_count, speed);
}

const ARCHIVE_MAGIC: &[u8; 4] = b"CUZA";
const ARCHIVE_VERSION: u8 = 1;

struct ArchiveEntry {
    name: String,
    original_size: u64,
    compressed_size: u64,
}

fn archive_cmd(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: cuz archive <output.cuz> <file1> [file2...] [-r]");
        std::process::exit(1);
    }

    let output_path = &args[0];
    let mut files: Vec<String> = Vec::new();
    let mut recursive = false;

   
    for arg in &args[1..] {
        if arg == "-r" {
            recursive = true;
        } else {
           
            let path = Path::new(arg);
            if path.is_dir() && recursive {
                collect_files_recursive(path, &mut files);
            } else if path.is_file() {
                files.push(arg.clone());
            } else if path.is_dir() {
               
                eprintln!("Warning: {} is a directory, use -r for recursive", arg);
            } else {
                eprintln!("Warning: {} not found, skipping", arg);
            }
        }
    }

    if files.is_empty() {
        eprintln!("No files to archive!");
        std::process::exit(1);
    }

    let start = Instant::now();
    let mut entries: Vec<ArchiveEntry> = Vec::new();
    let mut compressed_data: Vec<u8> = Vec::new();
    let mut total_raw = 0u64;
    let mut total_compressed = 0u64;

    for file_path in &files {
        let path = Path::new(file_path);
        let name = path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());

        match fs::read(file_path) {
            Ok(data) => {
                let original_size = data.len() as u64;
                total_raw += original_size;

               
                let options = CompressionOptions::default()
                    .with_compression_level(6)
                    .with_dictionary(true)
                    .with_dictionary_size(128 * 1024)
                    .with_transform(TRANSFORM_NONE);
                let compressed = compress_with_options(&data, options);
                let compressed_size = compressed.len() as u64;
                total_compressed += compressed_size;

                entries.push(ArchiveEntry {
                    name,
                    original_size,
                    compressed_size,
                });

                compressed_data.extend_from_slice(&compressed);
            }
            Err(e) => {
                eprintln!("  ! {} - Failed to read: {}", file_path, e);
            }
        }
    }

   
    let mut archive: Vec<u8> = Vec::new();

   
    archive.extend_from_slice(ARCHIVE_MAGIC);
    archive.push(ARCHIVE_VERSION);

   
    let file_count = entries.len() as u32;
    archive.extend_from_slice(&file_count.to_le_bytes());

   
    for entry in &entries {
       
        let name_bytes = entry.name.as_bytes();
        archive.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        archive.extend_from_slice(name_bytes);

       
        archive.extend_from_slice(&entry.original_size.to_le_bytes());

       
        archive.extend_from_slice(&entry.compressed_size.to_le_bytes());
    }

   
    archive.extend_from_slice(b"DATA");

   
    archive.extend_from_slice(&compressed_data);

   
    fs::write(output_path, &archive).expect("Failed to write archive");

    let elapsed = start.elapsed();
    let overall_ratio = total_compressed as f64 / total_raw as f64 * 100.0;
    let speed = total_raw as f64 / 1_000_000.0 / elapsed.as_secs_f64();

    println!("{}: {} files ({:.2}%, {:.1} MB/s)", output_path, entries.len(), overall_ratio, speed);
}

fn collect_files_recursive(dir: &Path, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_string_lossy().to_string());
            } else if path.is_dir() {
                collect_files_recursive(&path, files);
            }
        }
    }
}

fn list_archive_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz list <archive.cuz>");
        std::process::exit(1);
    }

    let archive_path = &args[0];
    let data = fs::read(archive_path).expect("Failed to read archive");

   
    if data.len() < 9 || &data[0..4] != ARCHIVE_MAGIC {
        eprintln!("Error: {} is not a valid CUZ archive", archive_path);
        std::process::exit(1);
    }

    let version = data[4];
    let file_count = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;

    println!("Archive: {} (version {})", archive_path, version);
    println!("Files: {}", file_count);
    println!();
    println!("File             | Raw Size | Compressed | Ratio  ");
    println!("-----------------|----------|------------|--------");

    let mut cursor = 9;
    let mut total_raw = 0u64;
    let mut total_compressed = 0u64;

    for _ in 0..file_count {
        if cursor + 2 > data.len() { break; }

       
        let name_len = u16::from_le_bytes([data[cursor], data[cursor + 1]]) as usize;
        cursor += 2;

        if cursor + name_len > data.len() { break; }
        let name = String::from_utf8_lossy(&data[cursor..cursor + name_len]).to_string();
        cursor += name_len;

        if cursor + 16 > data.len() { break; }

       
        let original_size = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let compressed_size = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;

        total_raw += original_size;
        total_compressed += compressed_size;

        let ratio = compressed_size as f64 / original_size as f64 * 100.0;
        let display_name = if name.len() > 16 {
            format!("{}...", &name[..13])
        } else {
            format!("{:16}", name)
        };

        let raw_str = format_size(original_size);
        let comp_str = format_size(compressed_size);

        println!("{} | {:>8} | {:>10} | {:>5.1}%", display_name, raw_str, comp_str, ratio);
    }

    println!("-----------------|----------|------------|--------");
    let overall_ratio = if total_raw > 0 {
        total_compressed as f64 / total_raw as f64 * 100.0
    } else {
        0.0
    };
    println!("TOTAL            | {:>8} | {:>10} | {:>5.1}%",
             format_size(total_raw), format_size(total_compressed), overall_ratio);
}

fn extract_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz extract <archive.cuz> [output_dir]");
        std::process::exit(1);
    }

    let archive_path = &args[0];
    let output_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        ".".to_string()
    };

    let data = fs::read(archive_path).expect("Failed to read archive");

   
    if data.len() < 9 || &data[0..4] != ARCHIVE_MAGIC {
        eprintln!("Error: {} is not a valid CUZ archive", archive_path);
        std::process::exit(1);
    }

    let file_count = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;

   
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    println!("Extracting {} files to {}", file_count, output_dir);
    println!();

    let start = Instant::now();

   
    let mut cursor = 9;
    let mut entries: Vec<(String, u64, u64)> = Vec::new();

    for _ in 0..file_count {
        if cursor + 2 > data.len() { break; }

        let name_len = u16::from_le_bytes([data[cursor], data[cursor + 1]]) as usize;
        cursor += 2;

        if cursor + name_len > data.len() { break; }
        let name = String::from_utf8_lossy(&data[cursor..cursor + name_len]).to_string();
        cursor += name_len;

        if cursor + 16 > data.len() { break; }

        let original_size = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let compressed_size = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;

        entries.push((name, original_size, compressed_size));
    }

   
    if cursor + 4 <= data.len() && &data[cursor..cursor + 4] == b"DATA" {
        cursor += 4;
    }

   
    let mut extracted = 0;
    let mut total_size = 0u64;

    for (name, original_size, compressed_size) in &entries {
        if cursor + *compressed_size as usize > data.len() {
            eprintln!("  ! {} - Truncated data", name);
            continue;
        }

        let compressed = &data[cursor..cursor + *compressed_size as usize];
        cursor += *compressed_size as usize;

        match decompress(compressed) {
            Ok(decompressed) => {
                if decompressed.len() as u64 != *original_size {
                    eprintln!("  ! {} - Size mismatch", name);
                    continue;
                }

                let output_path = Path::new(&output_dir).join(name);
                fs::write(&output_path, &decompressed).expect("Failed to write file");
                extracted += 1;
                total_size += decompressed.len() as u64;
            }
            Err(e) => {
                eprintln!("  ! {} - Decompression failed: {:?}", name, e);
            }
        }
    }

    let elapsed = start.elapsed();
    let speed = total_size as f64 / 1_000_000.0 / elapsed.as_secs_f64();
    println!("{} -> {} files ({:.1} MB/s)", output_dir, extracted, speed);
}

fn firmware_cmd(args: &[String]) {
    use crystal_unified::{BlockRawCompressor, BlockRawReader};

    if args.is_empty() {
        eprintln!("Usage: cuz firmware <input> [output] [-l level] [-b block_kb]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut output_path = format!("{}.cuzb", input_path);
    let mut level = 3i32;
    let mut block_size = 64 * 1024usize;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-l" && i + 1 < args.len() {
            level = args[i + 1].parse().unwrap_or(3);
            i += 2;
        } else if args[i] == "-b" && i + 1 < args.len() {
            block_size = args[i + 1].parse::<usize>().unwrap_or(64) * 1024;
            i += 2;
        } else if !args[i].starts_with('-') && i == 1 {
            output_path = args[i].clone();
            i += 1;
        } else {
            i += 1;
        }
    }

    let data = fs::read(input_path).expect("Failed to read input file");
    let start = Instant::now();

    let compressor = BlockRawCompressor::new()
        .with_block_size(block_size)
        .with_level(level);

    let compressed = compressor.compress(&data);
    fs::write(&output_path, &compressed).expect("Failed to write output");

    let elapsed = start.elapsed();
    let speed = data.len() as f64 / 1_000_000.0 / elapsed.as_secs_f64();
    let ratio = compressed.len() as f64 / data.len() as f64 * 100.0;

   
    let reader = BlockRawReader::new(&compressed).unwrap();

    println!("{} -> {} ({:.2}%, {:.1} MB/s)", input_path, output_path, ratio, speed);
}

fn append_raw_cmd(args: &[String]) {
    use crystal_unified::BlockRawReader;

    if args.len() < 2 {
        eprintln!("Usage: cuz append-raw <archive.cuzb> <data>");
        std::process::exit(1);
    }

    let archive_path = &args[0];
    let data_path = &args[1];

    let archive_data = fs::read(archive_path).expect("Failed to read archive");
    let new_data = fs::read(data_path).expect("Failed to read data file");

    if new_data.is_empty() {
        println!("No data to append.");
        return;
    }

    let start = Instant::now();

    let reader = BlockRawReader::new(&archive_data).expect("Failed to parse archive");
    let old_blocks = reader.block_count();
    let old_size = reader.original_size();
    let level = reader.level();

    let archive = reader.to_archive();
    let new_archive = archive.append(&new_data, level);
    let output = new_archive.to_bytes();

    fs::write(archive_path, &output).expect("Failed to write archive");

    let elapsed = start.elapsed();
    let speed = new_data.len() as f64 / 1_000_000.0 / elapsed.as_secs_f64();

    println!("{}: +{} bytes ({} total, {:.1} MB/s)", archive_path, new_data.len(), new_archive.original_size, speed);
}

fn patch_cmd(args: &[String]) {
    use crystal_unified::BlockRawReader;

    if args.len() < 3 {
        eprintln!("Usage: cuz patch <archive.cuzb> <block#> <data>");
        std::process::exit(1);
    }

    let archive_path = &args[0];
    let block_idx: usize = args[1].parse().expect("Invalid block index");
    let data_path = &args[2];

    let archive_data = fs::read(archive_path).expect("Failed to read archive");
    let patch_data = fs::read(data_path).expect("Failed to read patch data");

    let start = Instant::now();

    let reader = BlockRawReader::new(&archive_data).expect("Failed to parse archive");
    let level = reader.level();

    if block_idx as u64 >= reader.block_count() {
        eprintln!("Block index {} out of range (archive has {} blocks)", block_idx, reader.block_count());
        std::process::exit(1);
    }

    let archive = reader.to_archive();
    let patched = archive.patch_block(block_idx, &patch_data, level)
        .expect("Failed to patch block");

    let output = patched.to_bytes();
    fs::write(archive_path, &output).expect("Failed to write archive");

    let elapsed = start.elapsed();

    println!("{}: block {} patched ({} bytes)", archive_path, block_idx, patch_data.len());
}

fn delta_cmd(args: &[String]) {
    use crystal_unified::BlockRawReader;
    use byteorder::{LittleEndian, WriteBytesExt};

    if args.len() < 4 {
        eprintln!("Usage: cuz delta <old> <new> -o <patch.cuzd>");
        std::process::exit(1);
    }

    let old_path = &args[0];
    let new_path = &args[1];

   
    let mut output_path = String::new();
    for i in 0..args.len() - 1 {
        if args[i] == "-o" {
            output_path = args[i + 1].clone();
            break;
        }
    }

    if output_path.is_empty() {
        eprintln!("Missing -o <output> parameter");
        std::process::exit(1);
    }

    let start = Instant::now();

   
    let old_data = read_binary_or_cuzb(old_path);
    let new_data = read_binary_or_cuzb(new_path);

   
    let block_size = 4096usize; // 4KB blocks for diff granularity
    let mut patches: Vec<(u64, Vec<u8>)> = Vec::new(); // (offset, data)

    let max_len = old_data.len().max(new_data.len());
    let mut i = 0;
    while i < max_len {
        let old_block = if i < old_data.len() {
            &old_data[i..(i + block_size).min(old_data.len())]
        } else {
            &[]
        };
        let new_block = if i < new_data.len() {
            &new_data[i..(i + block_size).min(new_data.len())]
        } else {
            &[]
        };

        if old_block != new_block {
           
            let diff_start = i;
            let mut diff_end = (i + block_size).min(new_data.len());

           
            while diff_end < new_data.len() {
                let check_start = diff_end;
                let check_end = (diff_end + block_size).min(new_data.len());
                let old_check = if check_start < old_data.len() {
                    &old_data[check_start..check_end.min(old_data.len())]
                } else {
                    &[]
                };
                let new_check = &new_data[check_start..check_end];

                if old_check == new_check && !old_check.is_empty() {
                    break;
                }
                diff_end = check_end;
            }

            patches.push((diff_start as u64, new_data[diff_start..diff_end].to_vec()));
            i = diff_end;
        } else {
            i += block_size;
        }
    }

   
   
    let mut output = Vec::new();
    output.extend_from_slice(b"CUZD");
    output.write_u32::<LittleEndian>(100).unwrap();
    output.write_u64::<LittleEndian>(old_data.len() as u64).unwrap();
    output.write_u64::<LittleEndian>(new_data.len() as u64).unwrap();
    output.write_u64::<LittleEndian>(patches.len() as u64).unwrap();

   
    for (offset, data) in &patches {
        let compressed = zstd::stream::encode_all(&data[..], 3).unwrap();
        output.write_u64::<LittleEndian>(*offset).unwrap();
        output.write_u64::<LittleEndian>(data.len() as u64).unwrap();
        output.write_u64::<LittleEndian>(compressed.len() as u64).unwrap();
        output.extend_from_slice(&compressed);
    }

    fs::write(&output_path, &output).expect("Failed to write delta");

    let elapsed = start.elapsed();
    let changed_bytes: usize = patches.iter().map(|(_, d)| d.len()).sum();

    println!("{} -> {} ({} bytes, {} changes)", output_path, new_path, output.len(), patches.len());
}

fn apply_cmd(args: &[String]) {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Cursor;

    if args.len() < 2 {
        eprintln!("Usage: cuz apply <file> <patch.cuzd> [-o output]");
        std::process::exit(1);
    }

    let file_path = &args[0];
    let patch_path = &args[1];

    let mut output_path = format!("{}.patched", file_path);
    for i in 0..args.len() - 1 {
        if args[i] == "-o" {
            output_path = args[i + 1].clone();
            break;
        }
    }

    let start = Instant::now();

   
    let mut data = read_binary_or_cuzb(file_path);

   
    let patch_data = fs::read(patch_path).expect("Failed to read patch");

    if patch_data.len() < 32 || &patch_data[0..4] != b"CUZD" {
        eprintln!("Invalid delta patch file");
        std::process::exit(1);
    }

    let mut cursor = Cursor::new(&patch_data[4..]);
    let _version = cursor.read_u32::<LittleEndian>().unwrap();
    let old_size = cursor.read_u64::<LittleEndian>().unwrap() as usize;
    let new_size = cursor.read_u64::<LittleEndian>().unwrap() as usize;
    let patch_count = cursor.read_u64::<LittleEndian>().unwrap() as usize;

    if data.len() != old_size {
        eprintln!("Warning: File size mismatch (expected {}, got {})", old_size, data.len());
    }

   
    data.resize(new_size, 0);

   
    let mut pos = 32usize;
    for _ in 0..patch_count {
        if pos + 24 > patch_data.len() {
            eprintln!("Truncated patch data");
            std::process::exit(1);
        }

        let offset = u64::from_le_bytes(patch_data[pos..pos+8].try_into().unwrap()) as usize;
        let orig_size = u64::from_le_bytes(patch_data[pos+8..pos+16].try_into().unwrap()) as usize;
        let comp_size = u64::from_le_bytes(patch_data[pos+16..pos+24].try_into().unwrap()) as usize;
        pos += 24;

        if pos + comp_size > patch_data.len() {
            eprintln!("Truncated patch data");
            std::process::exit(1);
        }

        let compressed = &patch_data[pos..pos+comp_size];
        let decompressed = zstd::stream::decode_all(compressed).expect("Failed to decompress patch");

        if decompressed.len() != orig_size {
            eprintln!("Patch size mismatch");
            std::process::exit(1);
        }

       
        let end = (offset + orig_size).min(data.len());
        data[offset..end].copy_from_slice(&decompressed[..end-offset]);
        pos += comp_size;
    }

    fs::write(&output_path, &data).expect("Failed to write output");

    let elapsed = start.elapsed();

    println!("{} + {} -> {} ({} bytes)", file_path, patch_path, output_path, data.len());
}

fn read_binary_or_cuzb(path: &str) -> Vec<u8> {
    use crystal_unified::BlockRawReader;

    let data = fs::read(path).expect(&format!("Failed to read {}", path));

    if BlockRawReader::is_block_raw_format(&data) {
        let reader = BlockRawReader::new(&data).expect("Failed to parse .cuzb");
        reader.decompress_all().expect("Failed to decompress")
    } else {
        data
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// DNA Reference Compression Commands
// ============================================================================

fn dna_index_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz dna-index <reference.fa> [output.cdni]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let output_path = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("{}.cdni", input_path)
    };

    println!("Building reference index from {}...", input_path);
    let start = Instant::now();

    let data = fs::read(input_path).expect("Failed to read reference file");
    let data_size = data.len();

    let index = ReferenceIndex::build(&data);
    let index_bytes = index.to_bytes();

    fs::write(&output_path, &index_bytes).expect("Failed to write index");

    let elapsed = start.elapsed();
    let speed = data_size as f64 / 1_000_000.0 / elapsed.as_secs_f64();

    println!("{} -> {} ({}, {:.1} MB/s)",
        input_path,
        output_path,
        format_size(index_bytes.len() as u64),
        speed
    );
}

fn dna_compress_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz dna-compress <input.fa> -r <reference.cdni> [-o output]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut ref_path: Option<String> = None;
    let mut output_path = format!("{}.cdnr", input_path);

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-r" && i + 1 < args.len() {
            ref_path = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "-o" && i + 1 < args.len() {
            output_path = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    let ref_path = ref_path.unwrap_or_else(|| {
        eprintln!("Error: Reference index required. Use -r <reference.cdni>");
        std::process::exit(1);
    });

    println!("Loading reference index...");
    let ref_data = fs::read(&ref_path).expect("Failed to read reference index");
    let index = ReferenceIndex::load(&ref_data).expect("Failed to parse reference index");

    println!("Compressing {}...", input_path);
    let start = Instant::now();

    let input_data = fs::read(input_path).expect("Failed to read input file");
    let input_size = input_data.len();

    let compressed = encode_dna_with_reference(&input_data, &index);

    // Apply zstd on top for additional compression
    let final_compressed = zstd::stream::encode_all(&compressed[..], 19).expect("zstd compression failed");

    fs::write(&output_path, &final_compressed).expect("Failed to write output");

    let elapsed = start.elapsed();
    let ratio = final_compressed.len() as f64 / input_size as f64 * 100.0;
    let speed = input_size as f64 / 1_000_000.0 / elapsed.as_secs_f64();

    println!("{} -> {} ({:.4}%, {:.1} MB/s)",
        input_path,
        output_path,
        ratio,
        speed
    );
    println!("  Original: {}", format_size(input_size as u64));
    println!("  Compressed: {}", format_size(final_compressed.len() as u64));
}

fn dna_decompress_cmd(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: cuz dna-decompress <input.cdnr> -r <reference.cdni> [-o output]");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut ref_path: Option<String> = None;
    let mut output_path = input_path.trim_end_matches(".cdnr").to_string();
    if output_path == *input_path {
        output_path = format!("{}.fa", input_path);
    }

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-r" && i + 1 < args.len() {
            ref_path = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "-o" && i + 1 < args.len() {
            output_path = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    let ref_path = ref_path.unwrap_or_else(|| {
        eprintln!("Error: Reference index required. Use -r <reference.cdni>");
        std::process::exit(1);
    });

    println!("Loading reference index...");
    let ref_data = fs::read(&ref_path).expect("Failed to read reference index");
    let index = ReferenceIndex::load(&ref_data).expect("Failed to parse reference index");

    println!("Decompressing {}...", input_path);
    let start = Instant::now();

    let compressed_data = fs::read(input_path).expect("Failed to read input file");

    // Decompress zstd first
    let ref_compressed = zstd::stream::decode_all(&compressed_data[..]).expect("zstd decompression failed");

    let decompressed = decode_dna_with_reference(&ref_compressed, &index);

    fs::write(&output_path, &decompressed).expect("Failed to write output");

    let elapsed = start.elapsed();
    let speed = decompressed.len() as f64 / 1_000_000.0 / elapsed.as_secs_f64();

    println!("{} -> {} ({}, {:.1} MB/s)",
        input_path,
        output_path,
        format_size(decompressed.len() as u64),
        speed
    );
}
