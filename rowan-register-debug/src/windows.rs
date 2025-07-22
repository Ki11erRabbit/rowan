use std::ffi::c_char;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Diagnostics::Debug::{SymAddSymbol, SymCleanup, SymInitialize};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

pub fn register_name(name: *const c_char, address: usize, size: usize) {
    let result = unsafe {
        let handle = GetCurrentProcess();
        SymInitialize(handle, std::ptr::null_mut::<u8>(), 1);
        let result = SymAddSymbol(GetCurrentProcess(), 0, name as *const u8, address as u64, size as u32, 0);
        SymCleanup(handle);
        result
    };

    if result == 0 {
        let code = unsafe { GetLastError() };
        panic!("Failed to register name. Error code: {code}");
    }
}