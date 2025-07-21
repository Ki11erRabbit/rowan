
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