use std::ffi::CString;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub fn register_name(name: &str, address: *const (), size: usize) {
    let c_string = CString::new(format!("jitted::{name}")).unwrap();
    let address = address as usize;

    windows::register_name(c_string.as_ptr(), address, size);
}