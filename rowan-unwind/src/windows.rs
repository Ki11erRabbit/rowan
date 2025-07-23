use std::mem::MaybeUninit;
use std::sync::LazyLock;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenThread, THREAD_ALL_ACCESS, GetCurrentThreadId};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::System::Diagnostics::Debug::*;
use windows_sys::Win32::System::SystemInformation::{IMAGE_FILE_MACHINE, IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64};
use crate::Frame;

static PROCESS_HANDLE: LazyLock<ProcessHandle> = LazyLock::new(|| ProcessHandle::new());

pub struct ProcessHandle(HANDLE);

impl ProcessHandle {
    pub fn new() -> Self {
        unsafe {
            let handle = GetCurrentProcess();
            SymInitialize(handle, std::ptr::null_mut::<u8>(), 1);
            ProcessHandle(handle)
        }
    }

    pub fn get_handle(&self) -> HANDLE {
        self.0
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            SymCleanup(self.0);
        }
    }
}

unsafe impl Sync for ProcessHandle {}
unsafe impl Send for ProcessHandle {}


#[link(name = "ntdll")]
unsafe extern "C" {
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

#[inline]
pub fn backtrace<F>(mut func: F) where F: FnMut(Frame) -> bool {
    let thread_handle = unsafe { OpenThread(THREAD_ALL_ACCESS, FALSE, GetCurrentThreadId()) };
    let mut context: Box<CONTEXT> = unsafe { Box::new(std::mem::zeroed()) };
    unsafe {
        RtlCaptureContext(context.as_mut());
    }
    let process_handle = PROCESS_HANDLE.get_handle();
    let mut stack = STACKFRAME64::default();
    initialize_stack(&mut stack, &context);
    loop {
        let result = unsafe {
            StackWalk64(
                MACHINE_TYPE as u32,
                process_handle,
                thread_handle,
                &mut stack,
                context.as_mut() as *mut CONTEXT as *mut _,
                None,
                Some(SymFunctionTableAccess64),
                Some(SymGetModuleBase64),
                None
            )
        };
        if result == 0 {
            break
        }

        let frame = Frame::new(stack.AddrStack.Offset as usize, stack.AddrPC.Offset as usize);

        if !func(frame) {
            break
        }
    }
    unsafe {
        CloseHandle(thread_handle);
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