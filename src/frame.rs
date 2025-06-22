use crate::{Format, utils::bgra_to_rgba};
use anyhow::{Result, anyhow};
use image::{ImageBuffer, Rgba};
use std::path::Path;

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
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = self.clone().try_into()?;
        image_buffer.save(path)?;
        Ok(true)
    }
}

impl TryFrom<Frame> for ImageBuffer<Rgba<u8>, Vec<u8>> {
    type Error = anyhow::Error;
    fn try_from(frame: Frame) -> std::result::Result<Self, Self::Error> {
        let mut buffer = frame.buffer;
        let buffer: Vec<u8> = match frame.format {
            Format::BGRA => {
                bgra_to_rgba(&mut buffer);
                buffer
            }
            Format::RGBA => buffer,
        };
        ImageBuffer::from_vec(frame.width, frame.height, buffer)
            .ok_or_else(|| anyhow!("Failed to convert frame to image buffer."))
    }
}
