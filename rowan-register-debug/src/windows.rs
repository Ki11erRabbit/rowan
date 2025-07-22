use std::ffi::c_char;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Diagnostics::Debug::SymAddSymbol;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

pub fn register_name(name: *c_char, address: usize, size: usize) {
    let result = unsafe {
        SymAddSymbol(GetCurrentProcess(), 0, name, address as u64, size as u32, 0)
    };

    if result != 0 {
        let code = unsafe { GetLastError() };
        panic!("Failed to register name. Error code: {code}");
    }
}