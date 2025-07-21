use std::cell::UnsafeCell;

pub struct FakeLock<T> {
    internal: UnsafeCell<T>,
}

impl<T> FakeLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            internal: UnsafeCell::new(value),
        }
    }

    pub fn read(&self) -> &T {
        unsafe {
            self.internal.get().as_ref().unwrap()
        }
    }

    pub fn write(&self) -> &mut T {
        unsafe {
            self.internal.get().as_mut().unwrap()
        }
    }
}

unsafe impl<T> Send for FakeLock<T> {}
unsafe impl<T> Sync for FakeLock<T> {}