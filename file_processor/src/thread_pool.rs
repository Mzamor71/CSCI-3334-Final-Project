use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Shutdown,
}

impl ThreadPool{
    pub fn new(size: usize) -> Self{
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size{
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool{
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send+ 'static,
        {
            if let Some(sender) = &self.sender{
                let job = Box::new(f);
                let _ = sender.send(Message::NewJob(job));
            }
        }

    pub fn shutdown(&mut self){
        if let Some(sender) = self.sender.take(){
            for _ in &self.workers{
                let _ = sender.send(Message::Shutdown);
            }
        }
        for worker in &mut self.workers{
            if let Some(thread) = worker.thread.take(){

            }
        }
    }    

    pub fn increase(&mut self, n: usize){
        if n == 0{
            return;
        }
        if self.sender.is_none(){
            return;
        }
        let sender = self.sender.as_ref().unwrap();
        unimplemented!("Dynamic increase is left as an excercise. Create the poo; with required size.");
    }
}

struct Worker{
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker{
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread = thread::spawn(move || loop{
            let message = {
                let lock = receiver.lock().expect("Receiver lock poisoned");
                lock.recv()
            };

            match message{
                Ok(Message::NewJob(job)) =>{
                    job();
                }
                Ok(Message::Shutdown) => {
                    break;
                }
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

impl Drop for ThreadPool{
    fn drop(&mut self){
        self.shutdown();
    }
}