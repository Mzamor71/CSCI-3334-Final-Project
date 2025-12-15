// main.rs
//
// Entry point for the file processing application.
// This file is responsible for:
// 1. Parsing command-line arguments
// 2. Initializing the thread pool and file processor
// 3. Receiving and handling progress updates
// 4. Displaying final statistics to the user

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
    // ----------------------------------------
    // Parse command-line arguments
    // ----------------------------------------
    // Expected usage:
    //   <program> <num_threads> <dir1> [dir2 ...]
    let args: Vec<String> = env::args().collect();

    // Ensure minimum required arguments are provided
    if args.len() < 3 {
        eprintln!("Usage: {} <num_threads> <dir1> [dir2 ...]", args[0]);
        std::process::exit(1);
    }

    // Number of worker threads to create
    let num_threads: usize = args[1].parse().expect("invalid num_threads");

    // Directories to process
    let dirs: Vec<String> = args[2..].to_vec();

    // ----------------------------------------
    // Initialize thread pool and file processor
    // ----------------------------------------
    let pool = ThreadPool::new(num_threads);

    // FileProcessor manages directory traversal, file analysis,
    // and progress reporting (persisted to progress.json)
    let processor = FileProcessor::new(pool, "progress.json");

    // Begin processing directories and receive a channel for progress updates
    let rx = processor.process_dirs(dirs);

    // ----------------------------------------
    // Listen for progress messages
    // ----------------------------------------
    let mut completed = 0usize;
    let mut total = 0usize;

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(msg) => match msg {
                // A file has started processing
                ProgressMessage::FileStarted(name) => {
                    println!("Started: {}", name);
                }

                // A file completed successfully
                ProgressMessage::FileCompleted(analysis) => {
                    handle_completed(analysis);
                    completed += 1;
                }

                // A file failed during processing
                ProgressMessage::FileFailed(name, err) => {
                    eprintln!(
                        "Failed {}: {} - {}",
                        name, err.operation, err.message
                    );
                    completed += 1;
                }

                // Periodic update containing total and completed counts
                ProgressMessage::OverallProgress { completed: c, total: t } => {
                    total = t;
                    println!("Progress: {}/{}", c, t);
                }
            },

            // Timeout allows the loop to remain responsive
            Err(RecvTimeoutError::Timeout) => {
                // Timeout reached without receiving a message.
                // This can be used for heartbeat logs or termination checks.
                // Example exit condition (not implemented here):
                // if completed == total && total > 0 { break; }
            }

            // Channel closed: no more messages will arrive
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    // ----------------------------------------
    // Final summary output
    // ----------------------------------------
    println!(
        "Done. Total bytes processed: {}",
        processor.total_bytes_processed()
    );
}

/// Handles successful file completion events by printing
/// detailed statistics about the analyzed file.
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
