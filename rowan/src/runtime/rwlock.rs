use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct RwLock<T> {
    data: UnsafeCell<T>,
}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self { data: UnsafeCell::new(data) }
    }

    pub fn read(&self) -> Result<&T, ()> {
        unsafe { Ok(&*self.data.get()) }
    }

    pub fn write(&self) -> Result<&mut T, ()> {
        unsafe { Ok(&mut *self.data.get()) }
    }
}

unsafe impl<T: Sync> Sync for RwLock<T> {}
unsafe impl<T: Send> Send for RwLock<T> {}