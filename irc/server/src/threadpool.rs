use std::{
    sync::{
        mpsc::{self, RecvError},
        Arc, Mutex,
    },
    thread,
};
///Estructura de threadpool con múltiples workers que atienden las solicitudes
/// del servidor.
pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut _workers = Vec::with_capacity(size);

        for id in 0..size {
            _workers.push(Worker::new(id, receiver.clone()))
        }

        ThreadPool { _workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    _id: usize,
    _thread: thread::JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(_id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let _thread = thread::spawn(move || {
            while let Ok(job) = get_something_to_execute(receiver.clone()) {
                println!("Atendiendo cliente en el worker {_id}");
                job();
            }
        });

        Worker { _id, _thread }
    }
}

fn get_something_to_execute(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Job, RecvError> {
    receiver.lock().unwrap().recv()
}
