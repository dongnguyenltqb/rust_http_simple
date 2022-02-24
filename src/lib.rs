use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

pub type JobFn = Box<dyn FnOnce() + Send + 'static>;

enum Message<T> {
    Exit,
    Job(T),
}

pub struct ThreadPool<T> {
    cap: u32,
    current_worker: i32,
    txs: Vec<Sender<Message<T>>>,
    rxs: Vec<Arc<Mutex<Receiver<Message<T>>>>>,
}

impl<T> ThreadPool<T>
where
    T: Send + 'static + FnOnce() -> (),
{
    pub fn new(size: usize) -> ThreadPool<T> {
        let mut txs = Vec::with_capacity(size);
        let mut rxs = Vec::with_capacity(size);
        for _ in 0..size {
            let (tx, rx) = channel::<Message<T>>();
            txs.push(tx);
            rxs.push(Arc::new(Mutex::new(rx)));
        }
        ThreadPool {
            cap: size as u32,
            current_worker: 0,
            txs,
            rxs,
        }
    }
    pub fn run(&self) {
        for i in 0..self.cap {
            let worker_id = i;
            let rx = Arc::clone(&self.rxs[i as usize]);
            thread::spawn(move || {
                println!("Worker  {} is running.", worker_id);
                let rx = rx.lock().unwrap();
                while let Ok(msg) = rx.recv() {
                    match msg {
                        Message::Exit => {
                            println!("Received exit signal.");
                        }
                        Message::Job(f) => {
                            println!("Worker {} received new request.", worker_id);
                            f();
                        }
                    }
                }
            });
        }
    }
    pub fn execute(&mut self, f: T) {
        self.current_worker += 1;
        if self.current_worker == self.cap as i32 {
            self.current_worker = 0;
        };
        self.txs[self.current_worker as usize].send(Message::Job(f));
    }
}
