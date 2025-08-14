use std::collections::HashSet;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::LazyLock;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::yield_now;
use crate::fake_lock::FakeLock;
use crate::runtime::{Runtime, Reference, WrappedReference, DO_GARBAGE_COLLECTION, THREAD_COUNT};

static MAX_HEAP_SIZE: LazyLock<AtomicI64> = LazyLock::new(|| {
    AtomicI64::new(4 * 1024 * 1024 * 1024) // 4 GB
});

static CURRENT_HEAP_SIZE: LazyLock<AtomicI64> = LazyLock::new(|| {
    AtomicI64::new(0)
});

static GC_SENDER: LazyLock<FakeLock<Option<Sender<HashSet<WrappedReference>>>>> = LazyLock::new(|| {
    FakeLock::new(None)
});

static TRIGGER_COLLECTION: LazyLock<FakeLock<Option<Sender<()>>>> = LazyLock::new(|| {
    FakeLock::new(None)
});

pub struct GarbageCollection {
    live_objects: HashSet<Reference>,
    gc_receiver: Receiver<HashSet<WrappedReference>>,
    start_collection: Receiver<()>,
}

impl GarbageCollection {
    pub fn new() -> Self {
        let (gc_sender, gc_receiver) = std::sync::mpsc::channel();
        let (trigger_collection, start_collection) = std::sync::mpsc::channel();

        {
            *GC_SENDER.write() = Some(gc_sender);
            *TRIGGER_COLLECTION.write() = Some(trigger_collection);
        }

        Self {
            live_objects: HashSet::new(),
            gc_receiver,
            start_collection,
        }
    }

    pub fn send_references(live_memory: HashSet<WrappedReference>) {
        GC_SENDER.read()
            .as_ref()
            .unwrap()
            .send(live_memory)
            .unwrap()
    }
    
    pub fn trigger_gc() {
        TRIGGER_COLLECTION.read().as_ref().unwrap().send(()).unwrap();
    }
    
    pub fn update_heap_size(size: i64) {
        let size = CURRENT_HEAP_SIZE.fetch_add(size, Ordering::Relaxed) + size;
        if size > MAX_HEAP_SIZE.load(Ordering::Relaxed) {
            Self::trigger_gc()
        }
    }
    
    pub fn initialize(max_heap_size: Option<i64>) {
        max_heap_size.map(|size| {
            MAX_HEAP_SIZE.store(size, Ordering::Relaxed);
        });
        std::thread::Builder::new().name("Garbage Collection".to_owned())
            .spawn(move || {
                let mut gc = GarbageCollection::new();
                gc.main_loop()
            }).expect("Thread 'new' panicked at 'Garbage Collection'");
    }

    pub fn main_loop(&mut self) {
        loop {
            match self.start_collection.recv() {
                Ok(_) => {},
                Err(_) => {
                    break;
                }
            }
            
            let mut thread_count = {
                THREAD_COUNT.read().load(std::sync::atomic::Ordering::Relaxed)
            };

            let lock = {
                DO_GARBAGE_COLLECTION.write().unwrap()
            };

            loop {
                match self.gc_receiver.recv() {
                    Ok(live_objects) => {
                        //println!("Received live objects: {live_objects:?}");
                        for live_object in live_objects.iter() {
                            Runtime::gc_explore_object(live_object.0, &mut self.live_objects);
                            self.live_objects.insert(live_object.0);
                        }
                        thread_count -= 1;

                        if thread_count == 0 {
                            break;
                        }
                    }
                    Err(_) => panic!("GarbageCollection sender closed"),
                }
            }
            let mut static_objects = HashSet::new();
            Runtime::collect_static_members(&mut static_objects);
            for live_object in static_objects.into_iter() {
                Runtime::gc_explore_object(live_object, &mut self.live_objects);
                self.live_objects.insert(live_object);
            }

            Runtime::gc_collect_garbage(&self.live_objects);
            self.live_objects.clear();
            drop(lock);
        }
    }
}

unsafe impl Send for GarbageCollection {}
unsafe impl Sync for GarbageCollection {}


