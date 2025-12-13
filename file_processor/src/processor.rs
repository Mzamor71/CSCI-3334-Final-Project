use crate::analyzer::{analyze_file, FileAnalysis, ProcessingError};
use crate::thread_pool::ThreadPool;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time;

pub enum ProgressMessage {
    FileStarted(String),
    FileCompleted(FileAnalysis),
    FileFailed(String, ProcessingError),
    OverallProgress { completed: usize, total: usize },
}

pub struct FileProcessor {
    pool: Arc<Mutex<ThreadPool>>,
    cancellation: Arc<AtomicBool>,
    total_bytes_processed: Arc<AtomicUsize>,
    persist_path: String,
}

impl FileProcessor {
    pub fn new(pool: ThreadPool, persist_path: impl Into<String>) -> Self {
        FileProcessor {
            pool: Arc::new(Mutex::new(pool)),
            cancellation: Arc::new(AtomicBool::new(false)),
            total_bytes_processed: Arc::new(AtomicUsize::new(0)),
            persist_path: persist_path.into(),
        }
    }

    pub fn cancel(&self) {
        self.cancellation.store(true, Ordering::SeqCst);
    }

    pub fn process_dirs(&self, dirs: Vec<String>) -> mpsc::Receiver<ProgressMessage> {
        let (tx, rx) = mpsc::channel::<ProgressMessage>();

        let paths = collect_files(dirs);
        let total = paths.len();

        let tx_main = tx.clone();
        let cancellation = Arc::clone(&self.cancellation);
        let total_bytes = Arc::clone(&self.total_bytes_processed);
        let pool_arc = Arc::clone(&self.pool);
        let persist_path = self.persist_path.clone();

        thread::spawn(move || {
            let mut completed = 0usize;

            for path in paths {
                if cancellation.load(Ordering::SeqCst) {
                    break;
                }

                let tx_file = tx_main.clone();
                tx_file
                    .send(ProgressMessage::FileStarted(path.clone()))
                    .ok();

                let path_clone = path.clone();
                let tx_submit = tx_file.clone();
                let cancel_for_task = Arc::clone(&cancellation);
                let bytes_for_task = Arc::clone(&total_bytes);

                let job = move || {
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

                    let analysis = analyze_file(&path_clone);

                    bytes_for_task.fetch_add(
                        analysis.stats.size_bytes as usize,
                        Ordering::SeqCst,
                    );

                    let _ = tx_submit.send(ProgressMessage::FileCompleted(analysis));
                };

                let pool_guard = pool_arc.lock().unwrap();
                pool_guard.execute(job);

                completed += 1;

                let _ = tx_main.send(ProgressMessage::OverallProgress {
                    completed,
                    total,
                });

                let _ = persist_progress(&persist_path, completed, total);

                thread::sleep(time::Duration::from_millis(10));
            }
        });

        rx
    }

    pub fn total_bytes_processed(&self) -> usize {
        self.total_bytes_processed.load(Ordering::SeqCst)
    }
}

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

fn persist_progress(path: &str, completed: usize, total: usize) -> std::io::Result<()> {
    let json = format!(r#"{{"completed":{},"total":{}}}"#, completed, total);
    std::fs::write(path, json)
}
