use std::ops::{Deref, DerefMut};
use crate::Complete;
use crate::pool::Pool;

pub trait PoolBoxAllocator<T: Complete>: Default {
    fn fetch_pool(&self) -> impl DerefMut<Target = Pool<T>>;
    fn request_pointer(&self) -> (*mut T, usize) {
        self.fetch_pool().request_pointer()
    }
    fn release_pointer(&mut self, index: usize) {
        self.fetch_pool().release_pointer(index);
    }
}

pub struct PoolBox<T: Complete, A: PoolBoxAllocator<T>> {
    pointer: *mut T,
    pool_index: usize,
    alloc: A,
}

impl<T: Complete, A: PoolBoxAllocator<T>> PoolBox<T, A> {
    pub fn new(item: T) -> Self {
        let alloc = A::default();
        let (ptr, index) = alloc.request_pointer();
        unsafe {
            ptr.write(item);
        }
        Self {
            pointer: ptr,
            alloc,
            pool_index: index,
        }
    }
}

impl<T: Complete, A: PoolBoxAllocator<T>> Deref for PoolBox<T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.pointer.as_ref().unwrap()
        }
    }
}

impl<T: Complete, A: PoolBoxAllocator<T>> DerefMut for PoolBox<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.pointer.as_mut().unwrap()
        }
    }
}

impl<T: Complete, A: PoolBoxAllocator<T>> Drop for PoolBox<T, A> {
    fn drop(&mut self) {
        self.alloc.release_pointer(self.pool_index);
    }
}

