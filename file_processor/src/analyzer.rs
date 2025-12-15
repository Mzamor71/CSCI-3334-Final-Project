// analyzer.rs
//
// This module is responsible for analyzing files and collecting
// statistics such as word count, line count, character frequencies,
// file size, and processing time. It also tracks any errors that occur
// during processing.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::time::{Duration, Instant};

/// Holds statistical information about a single file
#[derive(Debug, Clone)]
pub struct FileStats {
    /// Total number of words in the file
    pub word_count: usize,

    /// Total number of lines in the file
    pub line_count: usize,

    /// Frequency count of each character in the file
    pub char_frequencies: HashMap<char, usize>,

    /// File size in bytes
    pub size_bytes: u64,
}

impl FileStats {
    /// Creates a new `FileStats` instance with all values initialized to zero
    pub fn new() -> Self {
        FileStats {
            word_count: 0,
            line_count: 0,
            char_frequencies: HashMap::new(),
            size_bytes: 0,
        }
    }
}

/// Represents an error that occurred during file processing
#[derive(Debug)]
pub struct ProcessingError {
    /// Name of the file being processed
    pub filename: String,

    /// Operation during which the error occurred (e.g., open, read, metadata)
    pub operation: String,

    /// Error message describing the failure
    pub message: String,
}

/// Represents a high-level result of an analysis operation
/// (not directly used in `analyze_file`, but useful for aggregation)
#[derive(Debug)]
pub struct AnalysisResult {
    pub total_files: String,
    pub operation: String,
    pub message: String,
}

/// Contains the full analysis output for a single file
#[derive(Debug)]
pub struct FileAnalysis {
    /// File path or name
    pub filename: String,

    /// Collected file statistics
    pub stats: FileStats,

    /// List of errors encountered during processing
    pub errors: Vec<ProcessingError>,

    /// Total time taken to analyze the file
    pub processing_time: Duration,
}

/// Analyzes a file at the given path and returns a `FileAnalysis`
///
/// The function attempts to:
/// 1. Read file metadata (size)
/// 2. Read the file as UTF-8 text line-by-line
/// 3. Fall back to raw binary processing if UTF-8 decoding fails
pub fn analyze_file(path: &str) -> FileAnalysis {
    // Start timing the analysis
    let start = Instant::now();

    // Initialize statistics and error tracking
    let mut stats = FileStats::new();
    let mut errors = Vec::new();

    // Attempt to read file metadata to get size in bytes
    match std::fs::metadata(path) {
        Ok(metadata) => {
            stats.size_bytes = metadata.len();
        }
        Err(e) => {
            errors.push(ProcessingError {
                filename: path.to_string(),
                operation: "metadata".to_string(),
                message: format!("{:?}", e),
            });
        }
    }

    // Attempt to open the file
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            // If the file cannot be opened, return early
            errors.push(ProcessingError {
                filename: path.to_string(),
                operation: "open".to_string(),
                message: format!("{:?}", e),
            });
            return FileAnalysis {
                filename: path.to_string(),
                stats,
                errors,
                processing_time: start.elapsed(),
            };
        }
    };

    // Wrap the file in a buffered reader for efficient line-by-line reading
    let mut reader = BufReader::new(file);

    // Accumulators for analysis results
    let mut total_words = 0usize;
    let mut total_lines = 0usize;
    let mut freq: HashMap<char, usize> = HashMap::new();

    // Buffer for reading lines
    let mut line_buf = String::new();

    // Flag indicating whether UTF-8 reading was successful
    let mut utf8_ok = true;

    // Read the file line by line (UTF-8 path)
    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => break, // End of file
            Ok(_) => {
                total_lines += 1;

                // Count words using whitespace separation
                total_words += line_buf.split_whitespace().count();

                // Count character frequencies
                for ch in line_buf.chars() {
                    *freq.entry(ch).or_insert(0) += 1;
                }
            }
            Err(e) => {
                // UTF-8 decoding failed; switch to binary fallback
                utf8_ok = false;
                errors.push(ProcessingError {
                    filename: path.to_string(),
                    operation: "read_line".to_string(),
                    message: format!("{:?}", e),
                });
                break;
            }
        }
    }

    // ----------------------------
    // Binary fallback path
    // ----------------------------
    // If UTF-8 reading failed, re-read the file as raw bytes
    if !utf8_ok {
        if let Ok(mut f) = std::fs::File::open(path) {
            let mut buf = [0u8; 8 * 1024];

            loop {
                match f.read(&mut buf) {
                    Ok(0) => break, // End of file
                    Ok(n) => {
                        // Count byte-level character frequencies
                        for &b in &buf[..n] {
                            let ch = b as char;
                            *freq.entry(ch).or_insert(0) += 1;
                        }

                        // Count newline characters as lines
                        total_lines += buf[..n]
                            .iter()
                            .filter(|&&c| c == b'\n')
                            .count();

                        // Count words by splitting on common whitespace bytes
                        total_words += buf[..n]
                            .split(|&c| c == b' ' || c == b'\n' || c == b'\t' || c == b'\r')
                            .filter(|slice| !slice.is_empty())
                            .count();
                    }
                    Err(e2) => {
                        errors.push(ProcessingError {
                            filename: path.to_string(),
                            operation: "read_raw".to_string(),
                            message: format!("{:?}", e2),
                        });
                        break;
                    }
                }
            }
        }
    }

    // Store final computed statistics
    stats.word_count = total_words;
    stats.line_count = total_lines;
    stats.char_frequencies = freq;

    // Return the completed analysis
    FileAnalysis {
        filename: path.to_string(),
        stats,
        errors,
        processing_time: start.elapsed(),
    }
}
