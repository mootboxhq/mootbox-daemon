#[cfg(unix)]
#[path = "service/unix.rs"]
mod implementation;
#[cfg(windows)]
#[path = "service/windows.rs"]
mod implementation;

pub use implementation::*;
