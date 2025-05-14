use crate::{Format, utils::bgra_to_rgba};
use image::{ImageBuffer, Rgba};

#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
    pub format: Format,
}
impl Frame {
    pub fn new(width: u32, height: u32, buffer: Vec<u8>, format: Format) -> Self {
        Frame {
            width,
            height,
            buffer,
            format,
        }
    }
    pub fn to_image_buffer_rgba8(&mut self) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        match self.format {
            Format::BGRA => {
                bgra_to_rgba(&mut self.buffer);
            }
            _ => {}
        }
        ImageBuffer::from_vec(self.width, self.height, self.buffer.clone())
    }
}
