use super::FrameHandler;
use crate::{Config, Frame};
use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
use std::sync::mpsc::channel;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame as WindowsCaptureFrame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};

pub struct Capture {
    engine: Arc<Engine>,
    fps_map: HashMap<u64, u32>,
    now: Instant,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = Arc<Engine>;

    type Error = anyhow::Error;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
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
    ) -> Result<(), Self::Error> {
        if !self.engine.is_running.load().as_ref() {
            capture_control.stop();
            return Ok(());
        }
        {
            let elapsed_secs = self.now.elapsed().as_secs();
            *self.fps_map.entry(elapsed_secs).or_insert(0) += 1;
            if elapsed_secs >= 1 {
                if let Some(&fps) = self.fps_map.get(&(elapsed_secs - 1)) {
                    self.engine.fps.swap(Arc::new(fps));
                }
            }
            if self.fps_map.len() > 3 {
                if let Some(&min_key) = self.fps_map.keys().min() {
                    self.fps_map.remove(&min_key);
                }
            }
        }
        let frame = {
            let mut data = frame.buffer()?;
            let buffer = data.as_nopadding_buffer()?.to_vec();
            let (width, height) = (data.width(), data.height());
            Frame::new(width, height, buffer, self.engine.config.format)
        };
        let guard = self.engine.on_frame_arrived.load();
        let hander = guard.as_ref();
        (hander)(frame);
        Ok(())
    }
}

pub struct Engine {
    config: Config,
    on_frame_arrived: ArcSwap<FrameHandler>,
    fps: ArcSwap<u32>,
    is_running: ArcSwap<bool>,
}

impl Engine {
    pub fn new(config: Config, on_frame_arrived: FrameHandler) -> Arc<Self> {
        Arc::new(Engine {
            config,
            on_frame_arrived: ArcSwap::new(Arc::new(on_frame_arrived)),
            fps: ArcSwap::new(Arc::new(0)),
            is_running: ArcSwap::new(Arc::new(false)),
        })
    }

    pub fn start(self: &Arc<Self>) -> Result<()> {
        if self.is_running() {
            return Err(anyhow!("The capture engine is already running."));
        }
        let _guard = RestoreIsRunning {
            target: &self.is_running,
            original: Arc::new(false),
        };
        let item = Monitor::primary().map_err(|error| anyhow!(error))?;
        let cursor_capture = CursorCaptureSettings::WithoutCursor;
        let draw_border = DrawBorderSettings::Default;
        let color_format = match self.config.format {
            crate::Format::BGRA => ColorFormat::Bgra8,
            crate::Format::RGBA => ColorFormat::Rgba8,
        };
        let flags = Arc::clone(self);
        let settings = Settings::new(item, cursor_capture, draw_border, color_format, flags);
        self.is_running.swap(Arc::new(true));
        Capture::start(settings).map_err(|error| anyhow!(error))
    }

    pub fn start_background(self: &Arc<Self>) -> Result<()> {
        if self.is_running() {
            return Err(anyhow!("The capture engine is already running."));
        }
        let item = Monitor::primary().map_err(|error| anyhow!(error))?;
        let cursor_capture = CursorCaptureSettings::WithoutCursor;
        let draw_border = DrawBorderSettings::Default;
        let color_format = match self.config.format {
            crate::Format::BGRA => ColorFormat::Bgra8,
            crate::Format::RGBA => ColorFormat::Rgba8,
        };
        let flags = Arc::clone(self);
        let settings = Settings::new(item, cursor_capture, draw_border, color_format, flags);
        self.is_running.swap(Arc::new(true));
        let result = Capture::start_free_threaded(settings);
        if result.is_err() {
            self.is_running.swap(Arc::new(false));
        }
        Ok(())
    }

    pub fn stop(&self) {
        self.is_running.swap(Arc::new(false));
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load().as_ref().clone()
    }

    pub fn grab(self: &Arc<Self>, time_out_millis: u64) -> Result<Frame> {
        if self.is_running() {
            return Err(anyhow!("The capture engine is already running."));
        }
        let original_handler = self.on_frame_arrived.load().clone();

        let _guard = RestoreHandler {
            target: &self.on_frame_arrived,
            original: original_handler,
        };

        let (tx, rx) = channel::<Frame>();
        let frame_handler: FrameHandler = Box::new(move |frame: Frame| {
            let _ = tx.send(frame);
        });
        self.on_frame_arrived.swap(Arc::new(frame_handler));
        self.start_background()?;
        let frame = rx.recv_timeout(Duration::from_millis(time_out_millis))?;
        self.stop();
        Ok(frame)
    }
}

struct RestoreIsRunning<'a> {
    target: &'a ArcSwap<bool>,
    original: Arc<bool>,
}

impl Drop for RestoreIsRunning<'_> {
    fn drop(&mut self) {
        self.target.swap(self.original.clone());
    }
}

struct RestoreHandler<'a> {
    target: &'a ArcSwap<FrameHandler>,
    original: Arc<FrameHandler>,
}
impl Drop for RestoreHandler<'_> {
    fn drop(&mut self) {
        self.target.swap(self.original.clone());
    }
}
