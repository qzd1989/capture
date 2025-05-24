use super::FrameHandler;
use crate::{Config, frame::Frame};
use anyhow::{Result, anyhow};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Instant,
};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame as WindowsCaptureFrame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};
struct Capture {
    engine: Arc<Engine>,
    fps_map: HashMap<u64, u32>,
    now: Instant,
}
impl GraphicsCaptureApiHandler for Capture {
    type Flags = Arc<Engine>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn new(ctx: Context<Self::Flags>) -> std::result::Result<Self, Self::Error> {
        let engine = ctx.flags;
        let fps_map = HashMap::new();
        let now = Instant::now();
        Ok(Self {
            engine,
            fps_map,
            now,
        })
    }
    fn on_frame_arrived(
        &mut self,
        frame: &mut WindowsCaptureFrame,
        capture_control: InternalCaptureControl,
    ) -> std::result::Result<(), Self::Error> {
        if self.engine.status.load(Ordering::SeqCst) == false {
            capture_control.stop();
            return Ok(());
        }
        {
            let elapsed = self.now.elapsed();
            let key = elapsed.as_secs();
            *self.fps_map.entry(key).or_insert(0) += 1;
            if key >= 1 {
                let prev_key = key - 1;
                if let Some(fps) = self.fps_map.get(&prev_key) {
                    self.engine.fps.store(*fps, Ordering::SeqCst);
                }
            }
            if self.fps_map.len() > 3 {
                let min_key = *self.fps_map.keys().min().unwrap();
                self.fps_map.remove(&min_key);
            }
        }
        let mut data = frame.buffer()?;
        let buffer = data.as_nopadding_buffer()?.to_vec();
        let (width, height) = (data.width(), data.height());
        let frame = Frame::new(width, height, buffer, self.engine.config.format);
        (self.engine.on_frame_arrived)(frame, self.engine.fps.load(Ordering::SeqCst));
        Ok(())
    }
}
pub struct Engine {
    pub config: Config,
    pub on_frame_arrived: Box<dyn FrameHandler>,
    pub fps: Arc<AtomicU32>,
    status: Arc<AtomicBool>,
}
impl Engine {
    pub fn new(config: Config, on_frame_arrived: Box<dyn FrameHandler>) -> Arc<Self> {
        Arc::new(Engine {
            config,
            on_frame_arrived,
            fps: Arc::new(AtomicU32::new(0)),
            status: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn start(self: &Arc<Self>) -> Result<()> {
        let item = Monitor::primary().map_err(|error| anyhow!(error))?;
        let cursor_capture = CursorCaptureSettings::WithoutCursor;
        let draw_border = DrawBorderSettings::Default;
        let color_format = {
            match self.config.format {
                crate::Format::BGRA => ColorFormat::Bgra8,
                crate::Format::RGBA => ColorFormat::Rgba8,
            }
        };
        let flags = Arc::clone(self);
        let settings = Settings::new(item, cursor_capture, draw_border, color_format, flags);
        self.status.store(true, Ordering::SeqCst);
        if let Err(error) = Capture::start(settings) {
            self.status.store(false, Ordering::SeqCst);
            return Err(anyhow!(error));
        }
        Ok(())
    }
    pub fn stop(&self) {
        self.status.store(false, Ordering::SeqCst);
    }
}
