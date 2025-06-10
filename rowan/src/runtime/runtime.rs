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
    }

    pub fn main_loop(&mut self, main_symbol: Symbol) {
        let main_object_ref = Context::new_object(main_symbol);
        self.live_objects.insert(main_object_ref);
        self.thread_channels[0].send(Command::Ready(main_object_ref)).unwrap();
        
        let mut start_time = Instant::now();
        loop {
            let current_time = Instant::now();
            let duration = current_time.duration_since(start_time);
            let delta = duration.as_secs_f64();

            self.handle_attachments();
            
            *self.semaphore.lock().unwrap() = 0;
            
            // Fill queue with tasks
            for reference in &self.live_objects {
                Self::push_tick(*reference, delta);
            }
            
            // Notify threads of work to be done
            for channel in &self.thread_channels {
                channel.send(Command::Tick).unwrap()
            }
            
            loop {
                if *self.semaphore.lock().unwrap() == self.thread_channels.len() {
                    break;
                }
            }

            self.handle_attachments();
            
            // TODO: handle messages sent out
            
            start_time = current_time;
        }
    }
    
    fn handle_attachments(&mut self) {
        while let Ok(attach) = self.attachment_receiver.try_recv() {
            match attach {
                AttachObject::Attach {
                    parent,
                    child
                } => {
                    println!("parent: {}, child: {}", parent, child);
                    self.live_objects.insert(child);
                    self.parent_to_children.entry(parent)
                        .or_insert(HashSet::new())
                        .insert(child);
                }
                AttachObject::Detach {
                    parent,
                    child
                } => {
                    self.live_objects.remove(&child);
                    self.parent_to_children.entry(parent)
                        .or_insert(HashSet::new())
                        .remove(&child);
                }
            }
        }
    }
    
    fn push_tick(reference: Reference, delta: f64) {
        TICK_QUEUE.push(Tick {
            object: reference,
            delta
        })
    }
    
    pub fn pop_tick() -> Steal<Tick> {
        TICK_QUEUE.steal()
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