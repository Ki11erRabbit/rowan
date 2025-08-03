use std::cell::UnsafeCell;
use unwind_sys as unwind;
use unwind_sys::{UNW_TDEP_IP, UNW_TDEP_SP};
use crate::Frame;

trait GetPointers {
    fn stack_pointer(&self) -> u64;

    fn instruction_pointer(&self) -> u64;
}

pub struct LibUnwindCursor {
    cursor: UnsafeCell<unwind::unw_cursor_t>
}

impl LibUnwindCursor {

    pub fn new() -> LibUnwindCursor {
        let mut ctx: unwind::unw_context_t = unsafe { std::mem::zeroed() };
        let _result = unsafe { unwind_sys::unw_tdep_getcontext!(&mut ctx) };

        let mut cursor = unwind::unw_cursor_t {
            opaque: unsafe { std::mem::zeroed() },
        };

        let _result = unsafe {
            unwind::unw_init_local(&mut cursor, &mut ctx as *mut unwind::unw_context_t)
        };

        LibUnwindCursor {
            cursor: UnsafeCell::new(cursor),
        }
    }
}
impl Iterator for LibUnwindCursor {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if unsafe { unwind::unw_step(self.cursor.get())} > 0 {
            let sp = self.stack_pointer();
            let ip = self.instruction_pointer();
            let frame = Frame::new(sp as usize, ip as usize);

            Some(frame)
        } else {
            None
        }
    }
}


impl GetPointers for LibUnwindCursor {
    fn stack_pointer(&self) -> u64 {
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(self.cursor.get(), UNW_TDEP_SP, &mut value)
        };
        assert_eq!(result, 0, "unw_get_reg() returned an error");
        value
    }

    fn instruction_pointer(&self) -> u64 {
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(self.cursor.get(), UNW_TDEP_IP, &mut value)
        };
        assert_eq!(result, 0, "unw_get_reg() returned an error");
        value
    }
}

#[inline]
pub fn backtrace<F>(mut func: F) where F: FnMut(Frame) -> bool {
    for frame in LibUnwindCursor::new() {
        if !func(frame) {
            break;
        }
    }
}
