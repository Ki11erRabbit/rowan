use std::cell::UnsafeCell;
use libunwind_sys as unwind;

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


/// The unwind cursor starts at the youngest (most deeply nested) frame
/// and is used to track the frame state as the unwinder steps from
/// frame to frame. It is safe to make (shallow) copies of variables
/// of this type.
pub struct Cursor<'a> {
    cursor: UnsafeCell<unwind::unw_cursor_t>,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl Iterator for Cursor<'_> {
    type Item = CursorData;
    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { unwind::unw_step(self.cursor.get()) } > 0 {
            Some(CursorData {
                cursor: self.cursor.get(),
            })
        } else {
            None
        }
    }
}

pub struct CursorData {
    cursor: *mut unwind::unw_cursor_t,
}



impl CursorData {
    fn get_cursor(&self) -> *mut unwind::unw_cursor_t {
        self.cursor
    }

    pub fn resume(&self) -> ! {
        let cursor = self.get_cursor() as *const unwind::unw_cursor_t;
        unsafe {
            unwind::unw_resume(cursor as *mut unwind::unw_cursor_t)
        };
        todo!("remote resume");
    }


    pub fn get_register(&self, reg: super::machine::Register) -> Result<u64> {
        let cursor = self.get_cursor();
        let register = reg.into();
        let mut value = 0;
        let result = unsafe {
            unwind::unw_get_reg(cursor, register, &mut value)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(value)
        } else {
            Err(error)
        }
    }

    pub fn set_register(&mut self, reg: super::machine::Register, value: u64) -> Result<()> {
        let cursor = self.get_cursor();
        let register = reg.into();
        let result = unsafe {
            unwind::unw_set_reg(cursor, register, value)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(())
        } else {
            Err(error)
        }
    }

    pub fn get_floating_point_register(&mut self, reg: super::machine::Register) -> Result<unwind::unw_fpreg_t> {
        let cursor = self.get_cursor();
        let register = reg.into();
        let mut value: unwind::unw_fpreg_t = 0;
        let result = unsafe {
            unwind::unw_get_fpreg(cursor, register, &mut value)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(value)
        } else {
            Err(error)
        }
    }

    pub fn set_floating_point_register(&mut self, reg: super::machine::Register, value: unwind::unw_fpreg_t) -> Result<()> {
        let cursor = self.get_cursor();
        let register = reg.into();
        let result = unsafe {
            unwind::unw_set_fpreg(cursor, register, value)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(())
        } else {
            Err(error)
        }
    }

    /// The get_procedure_info() routine returns auxiliary information about the procedure
    /// that created the stack frame identified by the current CursorData.
    pub fn get_proceture_info(&self) -> Result<ProcedureInfo> {
        let cursor = self.get_cursor();
        let mut proc_info = unwind::unw_proc_info_t {
            end_ip: unsafe { std::mem::zeroed() },
            extra: unsafe { std::mem::zeroed() },
            flags: unsafe { std::mem::zeroed() },
            format: unsafe { std::mem::zeroed() },
            gp: unsafe { std::mem::zeroed() },
            handler: unsafe { std::mem::zeroed() },
            lsda: unsafe { std::mem::zeroed() },
            start_ip: unsafe { std::mem::zeroed() },
            unwind_info: unsafe { std::mem::zeroed() },
            unwind_info_size: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            unwind::unw_get_proc_info(cursor, &mut proc_info)
        };
        let error: Error = result.into();
        if error.is_success() {
            Ok(ProcedureInfo { proc_info })
        } else {
            Err(error)
        }
    }

    pub fn get_procedure_name(&self, buf: &mut [u8]) -> Result<usize> {
        let cursor = self.get_cursor();
        let mut offset = 0;
        let result = unsafe {
            unwind::unw_get_proc_name(cursor, buf.as_mut_ptr() as *mut i8, buf.len(), &mut offset)
        };

        let error: Error = result.into();
        if error.is_success() {
            Ok(offset as usize)
        } else {
            Err(error)
        }
    }
}

pub struct ProcedureInfo {
    proc_info: unwind::unw_proc_info_t,
}

impl ProcedureInfo {
    /// The address of the first instruction of the procedure.
    /// If this address cannot be determined (e.g., due to lack of unwind information),
    /// the start_ip member is cleared to 0. 
    pub fn get_start_ip(&self) -> usize {
        self.proc_info.start_ip as usize
    }
    /// The address of the first instruction beyond the end of the procedure.
    /// If this address cannot be determined (e.g., due to lack of unwind information),
    /// the end_ip member is cleared to 0. 
    pub fn get_end_ip(&self) -> usize {
        self.proc_info.start_ip as usize
    }
    /// The address of the language-specific data-area (LSDA).
    /// This area normally contains language-specific information needed during exception handling.
    /// If the procedure has no such area, this member is cleared to 0. 
    pub fn get_lsda(&self) -> usize {
        self.proc_info.lsda as usize
    }
    /// The address of the exception handler routine.
    /// This is sometimes called the personality routine.
    /// If the procedure does not define a personality routine, the handler member is cleared to 0.
    pub fn get_handler(&self) -> *const u8 {
        let ptr = self.proc_info.handler as *const u8;
        ptr
    }
    /// The global-pointer of the procedure.
    /// On platforms that do not use a global pointer,
    /// this member may contain an undefined value.
    /// On all other platforms, it must be set either to the
    /// correct global-pointer value of the procedure or to 0
    /// if the proper global-pointer cannot be obtained for some reason. 
    pub fn get_global_pointer(&self) -> *const u8 {
        self.proc_info.gp as *const u8
    }

    /// A set of flags. There are currently no target-independent flags.
    /// For the IA-64 target, the flag UNW_PI_FLAG_IA64_RBS_SWITCH is set
    /// if the procedure may switch the register-backing store.
    /// TODO: change this so that it can return a bitflags enum.
    pub fn get_flags(&self) -> u64 {
        self.proc_info.flags
    }
}


pub struct Context {
    ctx: unwind::unw_context_t,
}

impl Context {
    pub fn get_context() -> Result<Self> {
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
        let mut cursor = unwind::unw_cursor_t {
            opaque: unsafe { std::mem::zeroed() },
        };
        let result = unsafe {
            unwind::unw_init_local(&mut cursor, &self.ctx as *const unwind::unw_context_t  as *mut unwind::unw_context_t)
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

}


#[cfg(test)]
mod test {
    use super::*;

    fn backtrace() {
        let context = Context::get_context().unwrap();
        let mut cursor = context.cursor().unwrap();
        // Skip this current function call
        _ = cursor.next();

        while let Some(mut data) = cursor.next() {
            let mut buffer = vec![0; 1024];

            data.get_procedure_name(&mut buffer).unwrap();
            let mut i = 0;
            while i < buffer.len() && buffer[i] != 0 {
                i += 1;
            }
            buffer.truncate(i + 1);
            let string = unsafe { String::from_utf8_unchecked(buffer) };
            if string.contains("some_func") {
                _ = data.set_register(crate::x86_64::Register::RAX, 1);
                data.resume();
                break;
            }
        }
        
        panic!("backtrace");
    }

    fn some_func() -> u64 {
        other_func()
    }

    fn other_func() -> u64 {
        backtrace();
        0
    }

    
    #[test]
    fn test_backtrace() {
        if some_func() != 0 {
            assert!(true);
        } else {
            assert!(false);
        }
    }
    
}
