// thread_pool.rs
//
// This module implements a basic thread pool using worker threads
// and message passing. Jobs are submitted to the pool through a channel
// and executed by worker threads in a FIFO manner.

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// A thread pool that manages a fixed number of worker threads.
///
/// Jobs are submitted via a channel and executed by the workers.
/// The pool supports graceful shutdown through a shutdown message.
pub struct ThreadPool {
    /// Worker threads owned by the pool
    workers: Vec<Worker>,

    /// Sender side of the job queue channel
    sender: Option<mpsc::Sender<Message>>,
}

/// A job is a boxed closure that can be executed once
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Messages sent to worker threads
enum Message {
    /// Execute a new job
    NewJob(Job),

    /// Signal worker threads to shut down
    Shutdown,
}

impl ThreadPool {
    /// Creates a new thread pool with the specified number of workers
    ///
    /// # Panics
    /// Panics if `size` is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        // Create a channel for sending jobs to workers
        let (sender, receiver) = mpsc::channel::<Message>();

        // Share the receiver across all worker threads
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        // Spawn worker threads
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Submits a job to the thread pool for execution
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Only submit jobs if the pool is still active
        if let Some(sender) = &self.sender {
            let job = Box::new(f);
            let _ = sender.send(Message::NewJob(job));
        }
    }

    /// Gracefully shuts down the thread pool
    ///
    /// Sends a shutdown message to each worker and waits for all
    /// worker threads to exit.
    pub fn shutdown(&mut self) {
        // Stop accepting new jobs
        if let Some(sender) = self.sender.take() {
            // Send a shutdown message to each worker
            for _ in &self.workers {
                let _ = sender.send(Message::Shutdown);
            }
        }

        // Join all worker threads
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // Wait for the worker thread to finish
                let _ = thread.join();
            }
        }
    }

    /// Attempts to increase the number of worker threads dynamically
    ///
    /// NOTE: This functionality is intentionally left unimplemented.
    pub fn increase(&mut self, n: usize) {
        if n == 0 {
            return;
        }

        // Cannot increase if the pool has already been shut down
        if self.sender.is_none() {
            return;
        }

        let sender = self.sender.as_ref().unwrap();

        unimplemented!(
            "Dynamic increase is left as an exercise. Create the pool with required size."
        );
    }
}

/// Represents a single worker thread in the pool
struct Worker {
    /// Worker identifier (useful for debugging)
    id: usize,

    /// Handle to the worker thread
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker thread
    ///
    /// Each worker waits for messages and executes jobs until it
    /// receives a shutdown signal.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Block until a message is received
            let message = {
                let lock = receiver.lock().expect("Receiver lock poisoned");
                lock.recv()
            };

            match message {
                // Execute a submitted job
                Ok(Message::NewJob(job)) => {
                    job();
                }

                // Shutdown signal received
                Ok(Message::Shutdown) => {
                    break;
                }

                // Channel disconnected
                Err(_) => {
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/// Automatically shuts down the thread pool when it goes out of scope
impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}
