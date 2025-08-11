mod pool;
mod poolbox;

pub use poolbox::PoolBox;
pub use poolbox::PoolBoxAllocator;
pub use pool::Pool;

/// This marker trait designates that all of a type's constructors will completely initialize
/// a mutable reference to some data
pub unsafe trait Complete: Sized {}

unsafe impl Complete for u8 {}
unsafe impl Complete for u16 {}
unsafe impl Complete for u32 {}
unsafe impl Complete for u64 {}
unsafe impl Complete for usize {}
unsafe impl Complete for i8 {}
unsafe impl Complete for i16 {}
unsafe impl Complete for i32 {}
unsafe impl Complete for i64 {}
unsafe impl Complete for isize {}
unsafe impl Complete for f32 {}
unsafe impl Complete for f64 {}