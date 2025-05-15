use crate::{Config, Engine, FrameHandler, frame::Frame, utils::rgba_to_bgra};
use anyhow::{Result, anyhow};
use fast_image_resize::{PixelType, Resizer, images::Image};
use std::sync::atomic::Ordering;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
pub struct Controller {
    engine: Arc<Engine>,
    handle: Option<JoinHandle<()>>,
}
impl Controller {
    pub fn new(config: Config, on_frame_arrived: Box<dyn FrameHandler>) -> Self {
        let engine = Engine::new(config, on_frame_arrived);
        Self {
            engine,
            handle: None,
        }
    }
    pub fn start(&mut self) {
        if self.is_running() {
            self.stop();
        }
        let engine = Arc::clone(&self.engine);
        let handle = thread::spawn(move || {
            engine.start().unwrap();
        });
        self.handle = Some(handle);
    }
    pub fn stop(&mut self) {
        self.engine.stop();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
    pub fn is_running(&self) -> bool {
        self.engine.status.load(Ordering::Relaxed)
    }
    pub fn grab(config: Config) -> Result<Frame> {
        let Some(primary_monitor) = xcap::Monitor::all()
            .map_err(|error| anyhow!(error))?
            .into_iter()
            .find(|monitor| monitor.is_primary().unwrap())
        else {
            return Err(anyhow!("No primary monitor found"));
        };
        let rgba_image = primary_monitor.capture_image().unwrap();
        let (mut width, mut height, mut buffer) =
            (rgba_image.width(), rgba_image.height(), rgba_image.to_vec());
        if cfg!(target_os = "macos") {
            let scale_factor = primary_monitor.scale_factor().unwrap() as u32;
            let mut resizer = Resizer::new();
            let src_image = Image::from_vec_u8(width, height, buffer, PixelType::U8x4)
                .map_err(|error| anyhow!(error))?;
            width = width / scale_factor;
            height = height / scale_factor;
            let mut dst_image = Image::new(width, height, PixelType::U8x4);
            resizer
                .resize(&src_image, &mut dst_image, None)
                .map_err(|error| anyhow!(error))?;
            buffer = dst_image.buffer().to_vec();
        }
        let mut frame = Frame::new(width, height, buffer, config.format);
        match config.format {
            crate::Format::BGRA => {
                rgba_to_bgra(&mut frame.buffer);
            }
            _ => {}
        }
        Ok(frame)
    }
}
