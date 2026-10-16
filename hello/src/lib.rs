use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.

    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(f);

        self.sender
            .send(job)
            .unwrap_or_else(|_| panic!("Failed to send job"));
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // 获取任务
                let message = {
                    let receiver = receiver.lock().unwrap();
                    receiver.recv()
                };

                // Mutex 锁已经释放
                match message {
                    Ok(job) => {
                        println!(
                            "Worker {} got a job; executing.",
                            id
                        );

                        // 执行任务
                        job.call_box();
                    }

                    Err(_) => {
                        println!(
                            "Worker {} shutting down.",
                            id
                        );
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}