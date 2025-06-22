use crate::Frame;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

pub type FrameHandler = Box<dyn Fn(Frame) + Send + Sync>;
