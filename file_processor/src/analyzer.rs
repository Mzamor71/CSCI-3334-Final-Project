use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FileStats {
    pub word_count: usize,
    pub line_count: usize,
    pub char_frequencies: HashMap<char, usize>,
    pub size_bytes: u64,
}

impl FileStats {
    pub fn new() -> Self {
        FileStats {
            word_count: 0,
            line_count: 0,
            char_frequencies: HashMap::new(),
            size_bytes: 0,
        }
    }
}

#[derive(Debug)]
pub struct ProcessingError {
    pub filename: String,
    pub operation: String,
    pub message: String,
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub total_files: String,
    pub operation: String,
    pub message: String,
}

#[derive(Debug)]
pub struct FileAnalysis {
    pub filename: String,
    pub stats: FileStats,
    pub errors: Vec<ProcessingError>,
    pub processing_time: Duration,
}

pub fn analyze_file(path: &str) -> FileAnalysis {
    let start = Instant::now();
    let mut stats = FileStats::new();
    let mut errors = Vec::new();

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

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
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

    let mut reader = BufReader::new(file);

    let mut total_words = 0usize;
    let mut total_lines = 0usize;
    let mut freq: HashMap<char, usize> = HashMap::new();

    let mut line_buf = String::new();
    let mut utf8_ok = true;

    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => break,
            Ok(_) => {
                total_lines += 1;
                total_words += line_buf.split_whitespace().count();
                for ch in line_buf.chars() {
                    *freq.entry(ch).or_insert(0) += 1;
                }
            }
            Err(e) => {
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

    // Binary fallback path
    if !utf8_ok {
        if let Ok(mut f) = std::fs::File::open(path) {
            let mut buf = [0u8; 8 * 1024];
            loop {
                match f.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        for &b in &buf[..n] {
                            let ch = b as char;
                            *freq.entry(ch).or_insert(0) += 1;
                        }

                        total_lines += buf[..n]
                            .iter()
                            .filter(|&&c| c == b'\n')
                            .count();

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

    stats.word_count = total_words;
    stats.line_count = total_lines;
    stats.char_frequencies = freq;

    FileAnalysis {
        filename: path.to_string(),
        stats,
        errors,
        processing_time: start.elapsed(),
    }
}
