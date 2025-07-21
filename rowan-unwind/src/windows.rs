use windows_sys::Win32::System::Threading::{GetCurrentThread, GetCurrentProcess};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Diagnostics::Debug::*;
use windows_sys::Win32::System::SystemInformation::{IMAGE_FILE_MACHINE, IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64};
use crate::{Cursor, ThreadContext};

unsafe extern "system" {
    fn RtlCaptureContext(context: *mut CONTEXT);
}

#[cfg(target_arch = "x86_64")]
const CONTEXT_FULL: CONTEXT_FLAGS = windows_sys::Win32::System::Diagnostics::Debug::CONTEXT_FULL_AMD64;
#[cfg(target_arch = "aarch64")]
const CONTEXT_FULL: CONTEXT_FLAGS = windows_sys::Win32::System::Diagnostics::Debug::CONTEXT_FULL_ARM64;

#[cfg(target_arch = "x86_64")]
const MACHINE_TYPE: IMAGE_FILE_MACHINE = IMAGE_FILE_MACHINE_AMD64;
#[cfg(target_arch = "aarch64")]
const MACHINE_TYPE: IMAGE_FILE_MACHINE = IMAGE_FILE_MACHINE_ARM64;

pub struct WindowsUnwindCursor {
    thread_handle: HANDLE,
    context: CONTEXT,
    process_handle: HANDLE,
}

impl WindowsUnwindCursor {
    pub fn new() -> Self {
        let thread_handle = unsafe { GetCurrentThread() };
        let mut context = CONTEXT::default();
        context.ContextFlags = CONTEXT_FULL;
        unsafe {
            RtlCaptureContext(&mut context);
        }
        let process_handle = unsafe { GetCurrentProcess() };

        Self {
            thread_handle,
            context,
            process_handle,
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn initialize_stack(stack: &mut STACKFRAME64, context: &CONTEXT) {
        stack.AddrPC.Offset = context.Rip;
        stack.AddrPC.Mode = AddrModeFlat;
        stack.AddrFrame.Offset = context.Rbp;
        stack.AddrFrame.Mode = AddrModeFlat;
        stack.AddrStack.Offset = context.Rsp;
        stack.AddrStack.Mode = AddrModeFlat;
    }

    #[cfg(target_arch = "aarch64")]
    fn initialize_stack(stack: &mut STACKFRAME64, context: &CONTEXT) {
        unimplemented!("Implement initializing stack on Windows for ARM64")
    }
}

impl Iterator for WindowsUnwindCursor {
    type Item = WindowsUnwindContext;

    fn next(&mut self) -> Option<Self::Item> {
        let mut stack = STACKFRAME64::default();
        Self::initialize_stack(&mut stack, &self.context);

        let result = unsafe {
            StackWalk64(
                MACHINE_TYPE as u32,
                self.process_handle,
                self.thread_handle,
                &mut stack,
                &mut self.context as *mut CONTEXT as *mut _,
                std::mem::transmute::<_, PREAD_PROCESS_MEMORY_ROUTINE64>(std::ptr::null_mut::<usize>()),
                Some(SymFunctionTableAccess64),
                Some(SymGetModuleBase64),
                std::mem::transmute::<_, PTRANSLATE_ADDRESS_ROUTINE64>(std::ptr::null_mut::<usize>()),
            )
        };

        if result == 0 {
            None
        } else {
            Some(WindowsUnwindContext::new(stack, self.process_handle))
        }

    }
}

impl Cursor<WindowsUnwindContext> for WindowsUnwindCursor {}

pub struct WindowsUnwindContext {
    stack: STACKFRAME64,
    process_handle: HANDLE,
}

impl WindowsUnwindContext {
    pub fn new(stack: STACKFRAME64, process_handle: HANDLE) -> Self {
        Self {
            stack,
            process_handle,
        }
    }
}


impl ThreadContext for WindowsUnwindContext {
    fn stack_pointer(&self) -> u64 {
        self.stack.AddrStack.Offset
    }

    fn instruction_pointer(&self) -> u64 {
        self.stack.AddrPC.Offset
    }

    fn has_name(&self) -> bool {
        let mut buffer = [0; std::mem::size_of::<SYMBOL_INFO>() + MAX_SYM_NAME as usize];
        let mut symbol = unsafe {
            (buffer.as_mut_ptr() as *mut SYMBOL_INFO).as_mut().unwrap()
        };
        symbol.SizeOfStruct = std::mem::size_of::<SYMBOL_INFO>() as u32;
        symbol.MaxNameLen = MAX_SYM_NAME;

        let mut displacement = 0;
        if unsafe { SymFromAddr(self.process_handle, self.stack.AddrPC.Offset, &mut displacement, symbol) } != 0 {
            true
        } else {
            false
        }
    }
}