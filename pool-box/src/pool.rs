use std::alloc::Layout;
use crate::Complete;

/// A Pool is a store of pointers
pub struct Pool<T: Complete> {
    pool: Vec<*mut T>,
    free_list: Vec<usize>,
}

impl<T: Complete> Pool<T> {
    pub fn new(capacity: usize) -> Self {
        use std::alloc;
        let mut free_list = Vec::with_capacity(capacity);
        
        let mut pool = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let layout = Layout::new::<T>();
            let ptr: *mut T = unsafe { alloc::alloc(layout) } as *mut T;
            pool.push(ptr);
            free_list.push(i);
        }
        
        Self {
            pool,
            free_list,
        }
    }
    
    fn add_ptrs(&mut self, requested_amount: usize) {
        use std::alloc;
        let layout = alloc::Layout::new::<T>();
        for _ in 0..requested_amount {
            let ptr: *mut T = unsafe { alloc::alloc(layout) } as *mut T;
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            self.pool.push(ptr);
            self.free_list.push(self.pool.len() - 1);
        }
    }
    
    fn resize_if_necessary(&mut self, requested_amount: usize) {
        if self.free_list.is_empty() {
            self.add_ptrs(requested_amount);
        }
    }
    
    pub fn request_pointer(&mut self) -> (*mut T, usize) {
        self.resize_if_necessary(1);
        let Some(next) = self.free_list.pop() else {
            unreachable!("We have already checked and made sure that we have enough space")
        };
        (self.pool[next], next)
    }
    
    pub fn release_pointer(&mut self, index: usize) {
        self.free_list.push(index);
    }
}

unsafe impl<T: Complete> Send for Pool<T> {}
unsafe impl<T: Complete> Sync for Pool<T> {}