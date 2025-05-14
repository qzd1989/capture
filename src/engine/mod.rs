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

pub fn bgra_to_rgba(bgra_data: &[u8], rgba_data: &mut Vec<u8>) {
    let len = bgra_data.len();
    assert!(len % 4 == 0);
    if rgba_data.capacity() < len {
        rgba_data.reserve_exact(len - rgba_data.capacity());
    }
    unsafe {
        rgba_data.set_len(len);
        let src = bgra_data.as_ptr();
        let dst = rgba_data.as_mut_ptr();
        let mut i = 0;
        while i < len {
            *dst.add(i) = *src.add(i + 2); // R
            *dst.add(i + 1) = *src.add(i + 1); // G
            *dst.add(i + 2) = *src.add(i); // B
            *dst.add(i + 3) = *src.add(i + 3); // A
            i += 4;
        }
    }
}
