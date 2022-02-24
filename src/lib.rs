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
    current_worker: u32,
    total_request: u32,
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
            total_request: 0,
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
                            return;
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
    pub fn execute(&mut self, f: T) -> u32 {
        self.current_worker += 1;
        self.total_request += 1;
        if self.current_worker == self.cap as u32 {
            self.current_worker = 0;
        };
        self.txs[self.current_worker as usize]
            .send(Message::Job(f))
            .unwrap();

        self.total_request
    }

    pub fn stop(&self) {
        for i in 0..self.cap {
            self.txs[i as usize].send(Message::Exit).unwrap();
        }
    }
}
