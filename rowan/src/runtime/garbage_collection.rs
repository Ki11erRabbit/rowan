use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::yield_now;
use crate::runtime::{Context, Reference, DO_GARBAGE_COLLECTION, THREAD_COUNT};

pub static mut GC_SENDER: Option<Sender<HashSet<Reference>>> = None;

pub struct GarbageCollection {
    live_objects: HashSet<Reference>,
    receiver: Receiver<HashSet<Reference>>,
}

impl GarbageCollection {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        unsafe {
            GC_SENDER = Some(sender);
        }

        Self {
            live_objects: HashSet::new(),
            receiver,
        }
    }

    pub fn main_loop(&mut self) {
        let mut start = std::time::Instant::now();
        loop {
            let now = std::time::Instant::now();
            let duration = now.duration_since(start);

            if duration.as_secs() >= 0 {// TODO: make this 5 mins configurable
                let mut thread_count = unsafe {
                    THREAD_COUNT.load(std::sync::atomic::Ordering::Relaxed)
                };

                let lock = unsafe {
                    DO_GARBAGE_COLLECTION.write().unwrap()
                };

                loop {
                    match self.receiver.recv() {
                        Ok(live_objects) => {
                            println!("Received live objects: {live_objects:?}");
                            for live_object in live_objects.iter() {
                                Context::gc_explore_object(*live_object, &mut self.live_objects);
                                self.live_objects.insert(*live_object);
                            }
                            thread_count -= 1;

                            if thread_count == 0 {
                                println!("Completed collecting all threads");
                                break;
                            }
                        }
                        Err(_) => panic!("GarbageCollection sender closed"),
                    }
                }

                Context::gc_collect_garbage(&self.live_objects);
                self.live_objects.clear();

                start = std::time::Instant::now();
                continue;
            }
            yield_now()
        }
    }
}


