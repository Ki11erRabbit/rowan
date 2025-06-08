use std::collections::{HashMap, HashSet};
use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::{mpsc, Arc, LazyLock, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use crossbeam_deque::Injector;
use crate::runtime::{Context, Reference, Symbol};

pub static MESSAGE_QUEUE: LazyLock<Injector<Message>> = LazyLock::new(|| {
    Injector::<Message>::new()
});

pub struct Runtime {
    parent_to_child: HashMap<Reference, HashSet<Reference>>,
    ref_to_bucket: HashMap<Reference, HashSet<Reference>>,
    buckets: Vec<HashSet<Reference>>,
    thread_handles: Vec<std::thread::JoinHandle<()>>,
    thread_channels: Vec<Sender<Command>>,
    semaphore: Arc<Mutex<usize>>,
    attachment_receiver: Receiver<AttachObject>,
    attachment_sender: Sender<AttachObject>,
}

impl Runtime {
    pub fn new(pool_size: usize) -> Self {        
        let (sender, receiver) = mpsc::channel();
        Self {
            parent_to_child: HashMap::new(),
            ref_to_bucket: HashMap::new(),
            buckets: Vec::with_capacity(pool_size),
            thread_handles: Vec::with_capacity(pool_size),
            thread_channels: Vec::with_capacity(pool_size),
            semaphore: Arc::new(Mutex::new(0)),
            attachment_receiver: receiver,
            attachment_sender: sender,
        }
    }

    pub fn spawn_thread(&mut self) {
              
        let (sender, receiver) = std::sync::mpsc::channel();
        let semaphore = self.semaphore.clone();
        let attachment_sender = self.attachment_sender.clone();

        let handle = std::thread::spawn(|| {
            let mut context = Context::new(receiver, semaphore, attachment_sender);
            context.main_loop();
        });

        self.thread_handles.push(handle);
        self.thread_channels.push(sender);
        self.buckets.push(HashSet::new());
    }

    pub fn main_loop(&mut self, main_symbol: Symbol) {
        let main_object_ref = Context::new_object(main_symbol);
        self.buckets[0].insert(main_object_ref);
        self.thread_channels[0].send(Command::Ready(main_object_ref)).unwrap();
        
        let mut start_time = Instant::now();
        loop {
            let current_time = Instant::now();
            let duration = current_time.duration_since(start_time);
            let delta = duration.as_secs_f64();
            
            for (i, bucket) in self.buckets.iter().enumerate() {
                for object in bucket {
                    // TODO: ensure that this ticks things bottom up to preserve thread safety
                    self.thread_channels[i].send(Command::Tick(*object, delta)).unwrap();
                }
            }
            // TODO: handle messages sent out
            // TODO: handle object attaching
            
            start_time = current_time;
        }
    }
}

pub struct Message {}

pub enum Command {
    /// Tells a context to perform a tick on the object reference with the time since tick
    Tick(Reference, f64),
    /// Tells a context to perform a ready on the object reference
    Ready(Reference),
}

pub struct AttachObject {}