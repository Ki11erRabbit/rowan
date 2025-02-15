
pub mod common;
pub mod dynamic;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64 as machine;
//pub mod machine;
