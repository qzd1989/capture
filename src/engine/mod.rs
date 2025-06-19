use crate::Frame;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

pub type FrameHandler = Box<dyn Fn(Frame) + Send + Sync>;
