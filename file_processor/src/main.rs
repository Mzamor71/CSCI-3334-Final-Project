mod analyzer;
mod processor;
mod thread_pool;

use analyzer::FileAnalysis;
use processor::{FileProcessor, ProgressMessage};
use std::env;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use thread_pool::ThreadPool;

fn main() {
    // Example usage:
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <num_threads> <dir1> [dir2 ...]", args[0]);
        std::process::exit(1);
    }
    let num_threads: usize = args[1].parse().expect("invalid num_threads");
    let dirs: Vec<String> = args[2..].to_vec();

    let pool = ThreadPool::new(num_threads);
    let processor = FileProcessor::new(pool, "progress.json");

    let rx = processor.process_dirs(dirs);

    // Listen for progress
    let mut completed = 0usize;
    let mut total = 0usize;
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(msg) => match msg {
                ProgressMessage::FileStarted(name) => {
                    println!("Started: {}", name);
                }
                ProgressMessage::FileCompleted(analysis) => {
                    handle_completed(analysis);
                    completed += 1;
                }
                ProgressMessage::FileFailed(name, err) => {
                    eprintln!("Failed {}: {} - {}", name, err.operation, err.message);
                    completed += 1;
                }
                ProgressMessage::OverallProgress { completed: c, total: t } => {
                    total = t;
                    println!("Progress: {}/{}", c, t);
                }
            },
            Err(RecvTimeoutError::Timeout) => {
                // timeout: we can print heartbeat or exit if finished
                // Basic exit heuristic: when progress file shows completed==total
            }
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    println!("Done. Total bytes processed: {}", processor.total_bytes_processed());

    
}

fn handle_completed(analysis: FileAnalysis) {
    println!(
        "Completed: {} (words: {}, lines: {}, bytes: {}, time: {:?}, errs: {})",
        analysis.filename,
        analysis.stats.word_count,
        analysis.stats.line_count,
        analysis.stats.size_bytes,
        analysis.processing_time,
        analysis.errors.len()
    );
}