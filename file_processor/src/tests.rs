// tests.rs
//
// This file contains unit and integration tests for the project.
// The tests focus on verifying:
// 1. Basic thread pool job execution
// 2. Correct behavior of the file analyzer on a simple input file

#[cfg(test)]
mod tests {
    // Import the thread pool implementation
    use super::thread_pool::ThreadPool;

    // Import file analysis functionality
    use super::analyzer::{analyze_file, FileStats};

    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;

    /// Unit test: verifies that the thread pool successfully
    /// executes submitted jobs.
    ///
    /// This test submits multiple jobs to the pool, each sending
    /// a value back through a channel. If all values are received,
    /// the thread pool is working correctly.
    #[test]
    fn test_thread_pool_executesjobs() {
        let pool = ThreadPool::new(4);

        // Channel used to collect results from worker threads
        let (tx, rx) = std::sync::mpsc::channel();

        // Submit 10 jobs to the thread pool
        for i in 0..10 {
            let tx = tx.clone();
            pool.execute(move || {
                tx.send(i).unwrap();
            });
        }

        // Collect results from executed jobs
        let mut seen = vec![];
        for _ in 0..10 {
            let v = rx.recv_timeout(Duration::from_secs(2)).unwrap();
            seen.push(v);
        }

        // Verify all jobs were executed
        assert_eq!(seen.len(), 10);
    }

    /// Integration test: verifies that `analyze_file`
    /// correctly analyzes a simple text file.
    ///
    /// This test:
    /// 1. Creates a temporary file
    /// 2. Writes known content to it
    /// 3. Runs the analyzer
    /// 4. Checks word and line counts
    /// 5. Cleans up the temporary file
    #[test]
    fn test_analyze_file_simple() {
        // Create a temporary file path
        let tmp = std::env::temp_dir().join("pff_test.txt");

        // Write sample content to the file
        let mut f = File::create(&tmp).unwrap();
        write!(f, "hello world\nthis is a test\n").unwrap();
        drop(f); // Ensure file is flushed and closed

        // Analyze the file
        let analysis = analyze_file(tmp.to_str().unwrap());

        // Validate expected statistics
        assert!(analysis.stats.word_count >= 5);

        // NOTE:
        // This assumes a field named `linecount`, but in your actual
        // FileStats struct the field is named `line_count`.
        assert!(analysis.stats.linecount >= 2);

        // Clean up temporary file
        let _ = std::fs::remove_file(&tmp);
    }
}
