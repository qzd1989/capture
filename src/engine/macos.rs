use super::FrameHandler;
use crate::Config;
use crate::Format;
use crate::Frame;
use crate::utils::bgra_to_rgba;
use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
use core_foundation::error::CFError;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use display_info::DisplayInfo;
use screencapturekit::{
    output::LockTrait,
    shareable_content::SCShareableContent,
    stream::{
        SCStream,
        configuration::{SCStreamConfiguration, pixel_format::PixelFormat},
        content_filter::SCContentFilter,
        output_trait::SCStreamOutputTrait,
        output_type::SCStreamOutputType,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Duration;
use std::time::Instant;

struct StreamOutput {
    sender: Sender<CMSampleBuffer>,
}

impl StreamOutput {
    pub fn new(sender: Sender<CMSampleBuffer>) -> Self {
        Self { sender }
    }
}

impl SCStreamOutputTrait for StreamOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        self.sender
            .send(sample_buffer)
            .expect("Could not send to output_buffer.");
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
        let (tx, rx) = channel();
        let display = {
            let get_primary_display_id = || -> Result<u32> {
                let display_infos = DisplayInfo::all().unwrap();
                for display_info in display_infos {
                    if display_info.is_primary {
                        return Ok(display_info.id);
                    }
                }
                Err(anyhow!("Primary display is not found."))
            };
            let primary_display_id = get_primary_display_id()?;
            let displays = SCShareableContent::get().unwrap().displays();
            let display = displays
                .into_iter()
                .find(move |x| x.display_id() == primary_display_id)
                .unwrap();
            display
        };

        let err_callback = |error: CFError| anyhow!(error.to_string());
        let config = SCStreamConfiguration::new()
            .set_captures_audio(false)
            .map_err(err_callback)?
            .set_pixel_format(PixelFormat::BGRA)
            .map_err(err_callback)?
            .set_width(display.width())
            .map_err(err_callback)?
            .set_height(display.height())
            .map_err(err_callback)?
            .set_shows_cursor(false)
            .map_err(err_callback)?;
        let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::new(&filter, &config);
        stream.add_output_handler(StreamOutput::new(tx), SCStreamOutputType::Screen);
        stream.start_capture().map_err(err_callback)?;

        let mut fps_map: HashMap<u64, u32> = HashMap::new();
        let now = Instant::now();
        self.is_running.swap(Arc::new(true));
        loop {
            let elapsed = now.elapsed();
            if !self.is_running.load().as_ref() {
                self.stop();
                return Ok(());
            }
            if let Ok(sample) = rx.try_recv() {
                if let Ok(buffer) = sample.get_pixel_buffer() {
                    {
                        let key = elapsed.as_secs();
                        if fps_map.contains_key(&key) {
                            let value = fps_map.get_mut(&key).unwrap();
                            *value += 1;
                        } else {
                            fps_map.insert(key, 1);
                        }
                        if key >= 1 {
                            let prev_key = key - 1;
                            if let Some(fps) = fps_map.get(&prev_key) {
                                self.fps.swap(Arc::new(*fps));
                            }
                        }
                        if fps_map.len() > 3 {
                            let min_key = *fps_map.keys().min().unwrap();
                            fps_map.remove(&min_key);
                        }
                    }
                    let mut frame = {
                        let guard = buffer.lock().unwrap();
                        Frame::new(
                            display.width(),
                            display.height(),
                            guard.as_slice().to_vec(),
                            self.config.format,
                        )
                    };
                    match self.config.format {
                        Format::RGBA => {
                            bgra_to_rgba(&mut frame.buffer);
                        }
                        _ => {}
                    }
                    let guard = self.on_frame_arrived.load();
                    let hander = guard.as_ref();
                    (hander)(frame);
                }
            }
        }
    }

    pub fn start_background(self: &Arc<Self>) -> Result<()> {
        let engine = Arc::clone(self);
        thread::spawn(move || engine.start());
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
