use std::path::Path;

use crate::{Format, utils::bgra_to_rgba};
use anyhow::{Result, anyhow};
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
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<bool> {
        let image_buffer = self.to_image_buffer_rgba8()?;
        image_buffer.save(path)?;
        Ok(true)
    }
    pub fn to_image_buffer_rgba8(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        match self.format {
            Format::BGRA => {
                bgra_to_rgba(&mut self.buffer);
            }
            _ => {}
        }
        match ImageBuffer::from_vec(self.width, self.height, self.buffer.clone()) {
            Some(image_buffer) => Ok(image_buffer),
            None => Err(anyhow!("Failed to create image buffer")),
        }
    }
}
