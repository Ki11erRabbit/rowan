use std::collections::{HashMap, HashSet};
use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::{mpsc, Arc, LazyLock, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use crossbeam_deque::{Injector, Steal};
use crate::runtime::{Context, Reference, Symbol};

pub static MESSAGE_QUEUE: LazyLock<Injector<Message>> = LazyLock::new(|| {
    Injector::<Message>::new()
});

static TICK_QUEUE: LazyLock<Injector<Tick>> = LazyLock::new(|| {
    Injector::<Tick>::new()
});

pub struct Runtime {
    parent_to_children: HashMap<Reference, HashSet<Reference>>,
    live_objects: HashSet<Reference>,
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
            parent_to_children: HashMap::new(),
            live_objects: HashSet::new(),
            thread_handles: Vec::with_capacity(pool_size),
            thread_channels: Vec::with_capacity(pool_size),
            semaphore: Arc::new(Mutex::new(0)),
            attachment_receiver: receiver,
            attachment_sender: sender,
        }
    }
}

pub struct Tick {
    pub object: Reference,
    pub delta: f64,
}

pub struct Message {}

pub enum Command {
    /// Tells a context to perform a tick on the object reference with the time since tick
    Tick,
    /// Tells a context to perform a ready on the object reference
    Ready(Reference),
}

pub enum AttachObject {
    Attach {
        parent: Reference,
        child: Reference,
    },
    Detach {
        parent: Reference,
        child: Reference,
    }
}