#[cfg_attr(target_os = "windows", path = "windows.rs")]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(not(any(target_os = "windows", target_os = "macos")), path = "any.rs")]
mod sys;

pub use sys::*;
