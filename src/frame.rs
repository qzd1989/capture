use crate::Format;
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
}
