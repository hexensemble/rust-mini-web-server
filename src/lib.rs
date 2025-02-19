use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new Threadpool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The 'new' function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Submits a job to the thread pool for execution.
    ///
    /// This function takes a closure and sends it to a worker thread for execution.
    /// If the thread pool has already been shut down, the job will not be executed.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            if let Err(e) = sender.send(Box::new(f)) {
                eprintln!("Failed to send job to worker: {}", e);
            }
        } else {
            eprintln!("ThreadPool has been shut down. Cannot execute job.");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    eprintln!("Failed to join worker {}: {:?}", worker.id, e);
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = match receiver.lock() {
                Ok(rx) => match rx.recv() {
                    Ok(job) => job,

                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                },
                Err(_) => {
                    eprintln!("Worker {id} failed to acquire lock; shutting down.");
                    break;
                }
            };

            println!("Worker {id} got a job; executing.");
            job();
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
