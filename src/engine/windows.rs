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
    type Error = anyhow::Error;
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

    fn update_fps(&mut self) {
        let elapsed_secs = self.now.elapsed().as_secs();
        *self.fps_map.entry(elapsed_secs).or_insert(0) += 1;
        if elapsed_secs >= 1 {
            if let Some(&fps) = self.fps_map.get(&(elapsed_secs - 1)) {
                self.engine.fps.store(fps, Ordering::SeqCst);
            }
        }
        if self.fps_map.len() > 3 {
            if let Some(&min_key) = self.fps_map.keys().min() {
                self.fps_map.remove(&min_key);
            }
        }
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut WindowsCaptureFrame,
        capture_control: InternalCaptureControl,
    ) -> std::result::Result<(), Self::Error> {
        if !self.engine.running.load(Ordering::SeqCst) {
            capture_control.stop();
            return Ok(());
        }
        self.update_fps();
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
    running: Arc<AtomicBool>,
}
impl Engine {
    pub fn new(config: Config, on_frame_arrived: Box<dyn FrameHandler>) -> Arc<Self> {
        Arc::new(Engine {
            config,
            on_frame_arrived,
            fps: Arc::new(AtomicU32::new(0)),
            running: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn start(self: &Arc<Self>) -> Result<()> {
        let item = Monitor::primary().map_err(|error| anyhow!(error))?;
        let cursor_capture = CursorCaptureSettings::WithoutCursor;
        let draw_border = DrawBorderSettings::Default;
        let color_format = match self.config.format {
            crate::Format::BGRA => ColorFormat::Bgra8,
            crate::Format::RGBA => ColorFormat::Rgba8,
        };
        let flags = Arc::clone(self);
        let settings = Settings::new(item, cursor_capture, draw_border, color_format, flags);
        self.running.store(true, Ordering::SeqCst);
        let result = Capture::start(settings);
        if result.is_err() {
            self.running.store(false, Ordering::SeqCst);
        }
        result.map_err(|e| anyhow!(e))
    }
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
