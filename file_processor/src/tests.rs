#[cfg(test)]
mod tests {
    use super::thread_pool::ThreadPool;
    use super::analyzer::{analyze_file, FileStats};
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;

    // Unit test for thread pool basic execution
    #[test]
    fn test_thread_pool_executesjobs() {
        let pool = ThreadPool::new(4);
        let (tx, rx) = std::sync::mpsc::channel();
        for i in 0..10 {
            let tx = tx.clone();
            pool.execute(move || {
                tx.send(i).unwrap();
            });
        }
        // collect results
        let mut seen = vec![];
        for  in 0..10 {
            let v = rx.recv_timeout(Duration::from_secs(2)).unwrap();
            seen.push(v);
        }
        assert_eq!(seen.len(), 10);
    }

    // Integration test for analyzer
    #[test]
    fn test_analyze_file_simple() {
        let tmp = std::env::temp_dir().join("pff_test.txt");
        let mut f = File::create(&tmp).unwrap();
        write!(f, "hello world\nthis is a test\n").unwrap();
        drop(f);

        let analysis = analyze_file(tmp.to_str().unwrap());
        assert!(analysis.stats.word_count >= 5);
        assert!(analysis.stats.linecount >= 2);
        // cleanup
        let  = std::fs::remove_file(&tmp);
    }
}