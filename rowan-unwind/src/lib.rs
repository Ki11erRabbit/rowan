use std::ffi::CString;
use std::sync::{LazyLock, RwLock};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

struct JitFunctionMap {
    start: usize,
    end: usize,
}

impl JitFunctionMap {
    fn new() -> Self {
        JitFunctionMap {
            start: 0,
            end: 0,
        }
    }

    fn test_ip(&self, ip: usize) -> bool {
        ip >= self.start && ip <= self.end
    }

    fn register(&mut self, pointer: usize, size: usize) {
        if self.start == self.end {
            self.start = pointer;
            self.end = pointer + size;
            return;
        }

        if pointer + size > self.end {
            self.end = pointer + size;
        }
    }
}

static JIT_MAP: LazyLock<RwLock<JitFunctionMap>> = LazyLock::new(|| {
    RwLock::new(JitFunctionMap::new())
});

pub fn test_ip(ip: usize) -> bool {
    let map = JIT_MAP.read().expect("failed to lock JIT_MAP");
    map.test_ip(ip)
}

pub fn register(pointer: *const (), size: usize) {
    let mut map = JIT_MAP.write().expect("failed to lock JIT_MAP");
    map.register(pointer as usize, size);
}

pub struct Frame {
    sp: usize,
    ip: usize,
}

impl Frame {
    fn new(sp: usize, ip: usize) -> Self {
        Frame { sp, ip }
    }

    pub fn sp(&self) -> usize {
        self.sp
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn is_jitted(&self) -> bool {
        test_ip(self.ip)
    }
}

#[cfg(unix)]
pub fn backtrace<F>(func: F) where F: FnMut(Frame) -> bool {
    unix::backtrace(func)
}

#[cfg(windows)]
pub fn backtrace<F>(func: F) where F: FnMut(Frame) -> bool {
    windows::backtrace(func);
}

