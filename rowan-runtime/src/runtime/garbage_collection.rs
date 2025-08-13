use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::yield_now;
use crate::fake_lock::FakeLock;
use crate::runtime::{Runtime, Reference, WrappedReference, DO_GARBAGE_COLLECTION, THREAD_COUNT};

pub static GC_SENDER: LazyLock<FakeLock<Option<Sender<HashSet<WrappedReference>>>>> = LazyLock::new(|| {
    FakeLock::new(None)
});

pub struct GarbageCollection {
    live_objects: HashSet<Reference>,
    receiver: Receiver<HashSet<WrappedReference>>,
}

impl GarbageCollection {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        {
            *GC_SENDER.write() = Some(sender);
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

            if duration.as_secs() >= 2 {// TODO: make this 5 mins configurable
                let mut thread_count = {
                    THREAD_COUNT.read().load(std::sync::atomic::Ordering::Relaxed)
                };

                let lock = {
                    DO_GARBAGE_COLLECTION.write().unwrap()
                };

                loop {
                    match self.receiver.recv() {
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

                Runtime::gc_collect_garbage(&self.live_objects);
                self.live_objects.clear();
                drop(lock);

                start = std::time::Instant::now();
                continue;
            }
            yield_now()
        }
    }
}

unsafe impl Send for GarbageCollection {}
unsafe impl Sync for GarbageCollection {}


