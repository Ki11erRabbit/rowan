use std::cell::UnsafeCell;

#[derive(Debug)]
pub enum Error {
    // Success
    Success,
    /// Unspecified (general) error
    Unspecified,
    /// Out of Memory
    NoMemory,
    ///Bad Register Number
    BadRegister,
    /// Attempt to write read-only register
    ReadOnlyRegister,
    /// Stop Unwinding
    StopUnwind,
    /// invalid IP
    InvalidIp,
    /// Bad Frame
    BadFrame,
    /// unsupported operation or bad value
    Invalid,
    /// unwind info has unsupported version
    BadVersion,
    ///no unwind info found
    NoInfo
}

impl Error {
    pub fn is_success(&self) -> bool {
        match self {
            Error::Success => true,
            _ => false
        }
    }
}

impl From<std::os::raw::c_int> for Error {
    fn from(source: std::os::raw::c_int) -> Self {
        match source {
            0 => Error::Success,
            -1 => Error::Unspecified,
            -2 => Error::NoMemory,
            -3 => Error::BadRegister,
            -4 => Error::ReadOnlyRegister,
            -5 => Error::StopUnwind,
            -6 => Error::InvalidIp,
            -7 => Error::BadFrame,
            -8 => Error::Invalid,
            -9 => Error::BadVersion,
            -10 => Error::NoInfo,
            _ => panic!("invalid error code"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum CachingPolicy {
    /// No caching
    None,
    /// shared global cache
    Global,
    /// per-thread caching
    PerThread,
}

impl From<std::os::raw::c_uint> for CachingPolicy {
    fn from(source: std::os::raw::c_uint) -> Self {
        match source {
            0 => CachingPolicy::None,
            1 => CachingPolicy::Global,
            2 => CachingPolicy::PerThread,
            _ => panic!("invalid caching policy"),
        }
    }
}

pub trait UnwindCursorData {
    /// This is for internal use only and should not be called
    fn get_cursor(&self) -> *mut crate::unw_cursor_t;
    fn get_register(&self, reg: super::machine::Register) -> Result<u64> {
        let cursor = self.get_cursor();
        let register = reg.into();
        let mut value = 0;
        let result = unsafe {
            crate::_Ux86_64_get_reg(cursor, register, &mut value)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(value)
        } else {
            Err(error)
        }
    }
    fn set_register(&mut self, reg: super::machine::Register, value: u64) -> Result<()> {
        let cursor = self.get_cursor();
        let register = reg.into();
        let result = unsafe {
            crate::_Ux86_64_set_reg(cursor, register, value)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(())
        } else {
            Err(error)
        }
    }
    fn resume(&self) -> ! {
        let cursor = self.get_cursor() as *const crate::unw_cursor_t;
        unsafe {
            crate::_Ux86_64_resume(cursor as *mut crate::unw_cursor_t)
        };
        todo!("remote resume");
    }
    fn get_procedure_name(&self, buf: &mut [u8]) -> Result<usize> {
        let cursor = self.get_cursor();
        let mut offset = 0;
        let result = unsafe {
            crate::_Ux86_64_get_proc_name(cursor, buf.as_mut_ptr() as *mut i8, buf.len(), &mut offset)
        };

        let error: Error = result.into();
        if error.is_success() {
            Ok(offset as usize)
        } else {
            Err(error)
        }
    }
}


/// The unwind cursor starts at the youngest (most deeply nested) frame
/// and is used to track the frame state as the unwinder steps from
/// frame to frame. It is safe to make (shallow) copies of variables
/// of this type.
pub struct Cursor<'a> {
    cursor: UnsafeCell<crate::unw_cursor_t>,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl Iterator for Cursor<'_> {
    type Item = CursorData;
    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { crate::_Ux86_64_step(self.cursor.get()) } > 0 {
            Some(CursorData {
                cursor: self.cursor.get(),
            })
        } else {
            None
        }
    }
}

pub struct CursorData {
    cursor: *mut crate::unw_cursor_t,
}

impl UnwindCursorData for CursorData {
    fn get_cursor(&self) -> *mut crate::unw_cursor_t {
        self.cursor
    }
}

/// The unwind cursor starts at the youngest (most deeply nested) frame
/// and is used to track the frame state as the unwinder steps from
/// frame to frame. It is safe to make (shallow) copies of variables
/// of this type.
pub struct CursorMut<'a> {
    cursor: UnsafeCell<crate::unw_cursor_t>,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl Iterator for CursorMut<'_> {
    type Item = CursorDataMut;
    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { crate::_Ux86_64_step(self.cursor.get()) } > 0 {
            Some(CursorDataMut {
                cursor: self.cursor.get(),
            })
        } else {
            None
        }
    }
}

pub struct CursorDataMut {
    cursor: *mut crate::unw_cursor_t,
}

impl UnwindCursorData for CursorDataMut {
    fn get_cursor(&self) -> *mut crate::unw_cursor_t {
        self.cursor
    }
}

pub struct Context {
    ctx: crate::unw_context_t,
}

impl Context {
    pub fn get_context() -> Result<Self> {
        let mut ctx = crate::unw_context_t {
            uc_flags: 0,
            uc_link: unsafe { std::mem::zeroed() },
            uc_stack: unsafe { std::mem::zeroed() },
            uc_sigmask: unsafe { std::mem::zeroed() },
            uc_mcontext: unsafe { std::mem::zeroed() },
            __fpregs_mem: unsafe { std::mem::zeroed() },
            __ssp: unsafe { std::mem::zeroed() },
            
            
        };
        let result = unsafe { crate::_Ux86_64_getcontext(&mut ctx) };
        let error: Error = result.into();
        if error.is_success() {
            Ok(Context {
                ctx
            })
        } else {
            Err(error)
        }
    }

    pub fn cursor<'a>(&'a self) -> Result<Cursor<'a>> {
        let mut cursor = crate::unw_cursor_t {
            opaque: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            crate::_Ux86_64_init_local(&mut cursor, &self.ctx as *const crate::unw_context_t  as *mut crate::unw_context_t)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(Cursor {
                cursor: UnsafeCell::new(cursor),
                phantom: std::marker::PhantomData,
            })
        } else {
            Err(error)
        }
    }

    pub fn cursor_mut<'a>(&'a mut self) -> Result<CursorMut<'a>> {
        let mut cursor = crate::unw_cursor_t {
            opaque: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            crate::_Ux86_64_init_local(&mut cursor, &mut self.ctx)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(CursorMut {
                cursor: UnsafeCell::new(cursor),
                phantom: std::marker::PhantomData,
            })
        } else {
            Err(error)
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn backtrace() {
        let context = Context::get_context().unwrap();
        let mut cursor = context.cursor().unwrap();
        // Skip this current function call
        _ = cursor.next();

        while let Some(data) = cursor.next() {
            let mut buffer = vec![0; 1024];

            data.get_procedure_name(&mut buffer).unwrap();
            let mut i = 0;
            while i < buffer.len() && buffer[i] != 0 {
                i += 1;
            }
            buffer.truncate(i + 1);
            let string = unsafe { String::from_utf8_unchecked(buffer) };
            println!("{}", string);
        }
        
        panic!("backtrace");
    }

    
    #[test]
    fn test_backtrace() {
        backtrace();

    }
    
}
