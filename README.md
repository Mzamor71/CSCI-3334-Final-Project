# Parallel File Processor

## Project Overview

This project implements a **parallel file processing system** in Rust using a **custom-built thread pool**. The system processes large collections of files concurrently and computes detailed statistics, including word count, line count, character frequencies, file size, and per-file processing time.

⚠️ **Assignment Constraint**: This project strictly uses **only Rust’s standard library**. No third-party libraries (e.g., `rayon`, `tokio`) and no asynchronous runtime are used, in full compliance with the course requirements.

---

## Features

### Core Functionality

* Custom thread pool implemented from first principles
* Concurrent processing of files across one or more directories
* Deterministic task coordination and graceful shutdown
* Shared-state synchronization using `Arc`, `Mutex`, and atomic types
* Cancellation support for long-running workloads
* Robust error handling for file system and I/O operations

### File Analysis Capabilities

For each processed file, the system computes:

* Total word count
* Total line count
* Character frequency histogram
* File size in bytes
* Processing time

### Progress Tracking and Reporting

* Real-time progress updates via message passing
* Per-file start, completion, and failure notifications
* Error reporting with contextual information
* Persistent progress tracking written to `progress.json`

---

## Project Structure

```
CSCI-3334-Final-Project/
├── books/                 # Project Gutenberg dataset (text files)
├── src/
│   ├── analyzer.rs        # File analysis logic
│   ├── processor.rs       # Task orchestration and progress tracking
│   ├── thread_pool.rs     # Custom thread pool implementation
│   ├── main.rs            # Program entry point
│   └── tests.rs           # Unit and integration tests
├── download_books.sh      # Script to download Gutenberg books
├── Cargo.toml
├── Cargo.lock
└── README.md
```

---

## Dataset Preparation

This project is designed to process **at least 100 text files** from Project Gutenberg.

To download the dataset, run the following commands from the project root:

```bash
chmod +x download_books.sh
bash download_books.sh books
```

All downloaded files will be saved as `.txt` files in the `books/` directory.

---

## Build Instructions

Ensure that Rust and Cargo are installed:

```bash
rustc --version
cargo --version
```

Build the project using:

```bash
cargo build
```

---

## Running the File Processor

### Command Format

```bash
cargo run -- <num_threads> <dir1> [dir2 ...]
```

### Example

To process all files in the `books/` directory using 8 worker threads:

```bash
cargo run -- 8 books
```

---

## Example Output

```text
Started: books/1342.txt
Progress: 1/100
Completed: books/1342.txt (words: 121533, lines: 6934, bytes: 783291, time: 45ms, errs: 0)
...
Done. Total bytes processed: 80234123
```

---

## Output Artifacts

* File analysis results are printed to the console in real time
* Aggregate progress is persisted to the following file:

```
progress.json
```

Example contents:

```json
{"completed":42,"total":100}
```

---

## Concurrency Design

* A **custom thread pool** is implemented using `std::thread` and `std::sync::mpsc`
* Work is submitted as boxed closures and executed by worker threads
* Shared state is protected using `Arc<Mutex<>>`
* Atomic variables are used for cancellation signaling and byte-count aggregation

No asynchronous runtime or parallel processing libraries are used.

---

## Testing

### Running Tests

```bash
cargo test
```

### Test Coverage

* Thread pool correctness and job execution (unit tests)
* File analysis accuracy on known inputs (integration tests)
* Error-handling scenarios and edge cases

---

## Compliance with Assignment Requirements

✔ Uses only the Rust standard library
✔ No `rayon`, `tokio`, or async constructs
✔ Custom thread pool implementation
✔ Processes 100+ files concurrently
✔ Graceful shutdown and cancellation support
✔ Correct use of `Arc`, `Mutex`, and atomic types

---

## Known Limitations

* UTF-8 is the primary supported encoding; a byte-based fallback is used for non-UTF-8 files
* Dynamic resizing of the thread pool is intentionally stubbed and documented
* Output is currently console-based (can be extended to structured formats such as CSV or JSON)

---

## Author

Michael Zamora
CSCI 3334 – Operating Systems / Parallel Programming

---

## License

This project uses only public-domain input data (Project Gutenberg texts) and is intended solely for educational use.
