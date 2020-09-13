use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    tx: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Instantiate ThreadPool
    ///
    /// The size specifies the number of threads
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, rx.clone()));
        }

        ThreadPool { workers, tx }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(Message::NewJob(Box::new(f))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Shutting down ThreadPool");
        for _ in &self.workers {
            self.tx.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
        println!("ThreadPool dropped");
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            if let Ok(msg) = rx.lock().unwrap().recv() {
                match msg {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job", id);
                        let st = SystemTime::now();
                        job();
                        let et = SystemTime::now();
                        let dur = et.duration_since(st).unwrap();
                        println!("Executed job in {} ms", dur.as_millis());
                    }
                    Message::Terminate => {
                        println!("Terminating worker {}", id);
                        break;
                    }
                    _ => println!("Msg unhandled"),
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
