use std::cell::UnsafeCell;
use std::ffi::{c_char, c_int};
use libunwind_sys as unwind;
use crate::{Cursor, ThreadContext};

pub struct LibUnwindCursor {
    cursor: UnsafeCell<unwind::unw_cursor_t>
}

impl LibUnwindCursor {
    pub fn new() -> LibUnwindCursor {
        let mut ctx = unwind::unw_context_t {
            uc_flags: 0,
            uc_link: unsafe { std::mem::zeroed() },
            uc_stack: unsafe { std::mem::zeroed() },
            uc_sigmask: unsafe { std::mem::zeroed() },
            uc_mcontext: unsafe { std::mem::zeroed() },
            __fpregs_mem: unsafe { std::mem::zeroed() },
            __ssp: unsafe { std::mem::zeroed() },


        };
        let result = unsafe { unwind::unw_getcontext(&mut ctx) };

        let mut cursor = unwind::unw_cursor_t {
            opaque: unsafe { std::mem::zeroed() },
        };

        let result = unsafe {
            unwind::unw_init_local(&mut cursor, &mut ctx as *mut unwind::unw_context_t)
        };

        LibUnwindCursor {
            cursor: UnsafeCell::new(cursor),
        }
    }
}
impl Iterator for LibUnwindCursor {
    type Item = LibUnwindContext;

    fn next(&mut self) -> Option<LibUnwindContext> {
        if unsafe { unwind::unw_step(self.cursor.get())} > 0 {
            Some(LibUnwindContext::new(self.cursor.get()))
        } else {
            None
        }
    }
}

impl Cursor<LibUnwindContext> for LibUnwindCursor {}

pub struct LibUnwindContext {
    cursor: *mut unwind::unw_cursor_t
}

impl LibUnwindContext {
    pub fn new(cursor: *mut unwind::unw_cursor_t) -> LibUnwindContext {
        Self { cursor }
    }
}

#[cfg(target_arch = "x86_64")]
impl ThreadContext for LibUnwindContext {
    fn stack_pointer(&self) -> u64 {
        const STACK_POINTER_INDEX: c_int = 7;
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(self.cursor, STACK_POINTER_INDEX, &mut value)
        };
        assert_eq!(result, 0, "unw_get_reg() returned an error");
        value
    }

    fn instruction_pointer(&self) -> u64 {
        const INSTRUCTION_POINTER_INDEX: c_int = 16;
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(self.cursor, INSTRUCTION_POINTER_INDEX, &mut value)
        };
        assert_eq!(result, 0, "unw_get_reg() returned an error");
        value
    }
}

#[cfg(target_arch = "aarch64")]
impl ThreadContext for LibUnwindContext {
    fn stack_pointer(&self) -> u64 {
        const STACK_POINTER_INDEX: u64 = 31;
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(self.cursor, STACK_POINTER_INDEX, &mut value)
        };
        assert_eq!(result, 0, "unw_get_reg() returned an error");
        value
    }

    fn instruction_pointer(&self) -> u64 {
        const INSTRUCTION_POINTER_INDEX: u64 = 32;
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(self.cursor, INSTRUCTION_POINTER_INDEX, &mut value)
        };
        assert_eq!(result, 0, "unw_get_reg() returned an error");
        value
    }
}