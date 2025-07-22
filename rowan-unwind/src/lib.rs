use std::ffi::CString;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub trait Cursor<T: ThreadContext>: Iterator<Item=T> {}

pub trait ThreadContext {
    fn stack_pointer(&self) -> u64;
    fn instruction_pointer(&self) -> u64;
    fn has_name(&self) -> bool;
}

#[cfg(unix)]
pub fn get_cursor() -> impl Cursor<unix::LibUnwindContext> {
    unix::LibUnwindCursor::new()
}
#[cfg(windows)]
pub fn get_cursor() -> impl Cursor<windows::WindowsUnwindContext> {
    windows::WindowsUnwindCursor::new()
}

#[cfg(windows)]
pub fn register_name(name: &str, address: *const (), size: usize) {
    let c_string = CString::new(format!("jitted::{name}")).unwrap();
    let address = address as usize;

    windows::register_name(c_string.as_ptr(), address, size);
}