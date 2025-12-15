// processor.rs
//
// This module coordinates directory traversal, file submission to the
// thread pool, progress reporting, cancellation handling, and persistence
// of overall progress to disk.

use crate::analyzer::{analyze_file, FileAnalysis, ProcessingError};
use crate::thread_pool::ThreadPool;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time;

/// Messages sent from worker threads to the main thread
/// to report progress and status updates.
pub enum ProgressMessage {
    /// A file has begun processing
    FileStarted(String),

    /// A file completed successfully with analysis results
    FileCompleted(FileAnalysis),

    /// A file failed to process
    FileFailed(String, ProcessingError),

    /// Overall progress update
    OverallProgress { completed: usize, total: usize },
}

/// Manages file processing using a thread pool and reports progress.
///
/// Responsibilities:
/// - Traverse directories and collect files
/// - Dispatch file analysis jobs to the thread pool
/// - Track progress and total bytes processed
/// - Support cancellation
/// - Persist progress to disk
pub struct FileProcessor {
    /// Shared thread pool used for executing file analysis jobs
    pool: Arc<Mutex<ThreadPool>>,

    /// Cancellation flag shared across threads
    cancellation: Arc<AtomicBool>,

    /// Total number of bytes processed across all files
    total_bytes_processed: Arc<AtomicUsize>,

    /// Path where progress information is persisted (JSON)
    persist_path: String,
}

impl FileProcessor {
    /// Creates a new `FileProcessor`
    pub fn new(pool: ThreadPool, persist_path: impl Into<String>) -> Self {
        FileProcessor {
            pool: Arc::new(Mutex::new(pool)),
            cancellation: Arc::new(AtomicBool::new(false)),
            total_bytes_processed: Arc::new(AtomicUsize::new(0)),
            persist_path: persist_path.into(),
        }
    }

    /// Signals all processing threads to cancel execution
    pub fn cancel(&self) {
        self.cancellation.store(true, Ordering::SeqCst);
    }

    /// Processes a list of directories and returns a receiver
    /// for progress updates.
    ///
    /// This function:
    /// - Collects all files from the provided directories
    /// - Submits file analysis jobs to the thread pool
    /// - Sends progress updates over an MPSC channel
    pub fn process_dirs(&self, dirs: Vec<String>) -> mpsc::Receiver<ProgressMessage> {
        let (tx, rx) = mpsc::channel::<ProgressMessage>();

        // Collect all file paths to be processed
        let paths = collect_files(dirs);
        let total = paths.len();

        // Clone shared state for the worker thread
        let tx_main = tx.clone();
        let cancellation = Arc::clone(&self.cancellation);
        let total_bytes = Arc::clone(&self.total_bytes_processed);
        let pool_arc = Arc::clone(&self.pool);
        let persist_path = self.persist_path.clone();

        // Spawn a dispatcher thread that submits jobs to the pool
        thread::spawn(move || {
            let mut completed = 0usize;

            for path in paths {
                // Stop processing if cancellation is requested
                if cancellation.load(Ordering::SeqCst) {
                    break;
                }

                // Notify that processing for this file has started
                let tx_file = tx_main.clone();
                tx_file
                    .send(ProgressMessage::FileStarted(path.clone()))
                    .ok();

                let path_clone = path.clone();
                let tx_submit = tx_file.clone();
                let cancel_for_task = Arc::clone(&cancellation);
                let bytes_for_task = Arc::clone(&total_bytes);

                // Job executed by the thread pool
                let job = move || {
                    // Check for cancellation before starting work
                    if cancel_for_task.load(Ordering::SeqCst) {
                        let _ = tx_submit.send(ProgressMessage::FileFailed(
                            path_clone.clone(),
                            ProcessingError {
                                filename: path_clone.clone(),
                                operation: "cancelled".into(),
                                message: "Cancelled before start".into(),
                            },
                        ));
                        return;
                    }

                    // Perform file analysis
                    let analysis = analyze_file(&path_clone);

                    // Update total bytes processed atomically
                    bytes_for_task.fetch_add(
                        analysis.stats.size_bytes as usize,
                        Ordering::SeqCst,
                    );

                    // Send completion message
                    let _ = tx_submit.send(ProgressMessage::FileCompleted(analysis));
                };

                // Submit job to the thread pool
                let pool_guard = pool_arc.lock().unwrap();
                pool_guard.execute(job);

                completed += 1;

                // Send overall progress update
                let _ = tx_main.send(ProgressMessage::OverallProgress {
                    completed,
                    total,
                });

                // Persist progress to disk
                let _ = persist_progress(&persist_path, completed, total);

                // Small delay to avoid overwhelming the system
                thread::sleep(time::Duration::from_millis(10));
            }
        });

        rx
    }

    /// Returns the total number of bytes processed so far
    pub fn total_bytes_processed(&self) -> usize {
        self.total_bytes_processed.load(Ordering::SeqCst)
    }
}

/// Collects all files from the given list of directories or file paths.
///
/// - If a path is a file, it is added directly
/// - If a path is a directory, all files in that directory are added
fn collect_files(dirs: Vec<String>) -> Vec<String> {
    let mut files = Vec::new();

    for d in dirs {
        let p = Path::new(&d);

        if p.is_file() {
            files.push(d.clone());
        } else if p.is_dir() {
            if let Ok(entries) = fs::read_dir(p) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_file() {
                        if let Some(s) = path.to_str() {
                            files.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    files
}

/// Persists overall progress to disk as a JSON file.
///
/// Example output:
/// `{ "completed": 5, "total": 20 }`
fn persist_progress(path: &str, completed: usize, total: usize) -> std::io::Result<()> {
    let json = format!(r#"{{"completed":{},"total":{}}}"#, completed, total);
    std::fs::write(path, json)
}
