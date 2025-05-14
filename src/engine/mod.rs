#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as engine;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as engine;

use crate::Frame;
pub use engine::*;

pub trait FrameHandler: Fn(Frame, u32) + Send + Sync + 'static {}
impl<T> FrameHandler for T where T: Fn(Frame, u32) + Send + Sync + 'static {}
