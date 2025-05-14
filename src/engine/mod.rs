mod macos;
use crate::Frame;
pub use engine::*;
use macos as engine;
pub trait FrameHandler: Fn(Frame, u32) + Send + Sync + 'static {}
impl<T> FrameHandler for T where T: Fn(Frame, u32) + Send + Sync + 'static {}
