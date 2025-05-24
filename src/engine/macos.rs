use super::FrameHandler;
use crate::Config;
use crate::Format;
use crate::Frame;
use crate::utils::bgra_to_rgba;
use anyhow::{Result, anyhow};
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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Sender, channel};
use std::time::Instant;
struct StreamOutput {
    sender: Sender<CMSampleBuffer>,
}
impl SCStreamOutputTrait for StreamOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: CMSampleBuffer,
        _of_type: SCStreamOutputType,
    ) {
        self.sender
            .send(sample_buffer)
            .expect("could not send to output_buffer");
    }
}
pub struct Engine {
    pub config: Config,
    pub on_frame_arrived: Box<dyn FrameHandler>,
    pub fps: Arc<AtomicU32>,
    pub status: Arc<AtomicBool>,
}
impl Engine {
    pub fn new(config: Config, on_frame_arrived: Box<dyn FrameHandler>) -> Arc<Self> {
        Arc::new(Self {
            config,
            on_frame_arrived,
            fps: Arc::new(AtomicU32::new(0)),
            status: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn start(&self) -> Result<()> {
        let (tx, rx) = channel();
        let display = {
            let get_primary_display_id = || -> Result<u32> {
                let display_infos = DisplayInfo::all().unwrap();
                for display_info in display_infos {
                    if display_info.is_primary {
                        return Ok(display_info.id);
                    }
                }
                Err(anyhow!("No primary display found"))
            };
            let primary_display_id = get_primary_display_id()?;
            let displays = SCShareableContent::get().unwrap().displays();
            let display = displays
                .into_iter()
                .find(move |x| x.display_id() == primary_display_id)
                .unwrap();
            display
        };
        let config = SCStreamConfiguration::new()
            .set_captures_audio(false)
            .map_err(|error| anyhow!(error.to_string()))?
            .set_pixel_format(PixelFormat::BGRA)
            .map_err(|error| anyhow!(error.to_string()))?
            .set_width(display.width())
            .map_err(|error| anyhow!(error.to_string()))?
            .set_height(display.height())
            .map_err(|error| anyhow!(error.to_string()))?
            .set_shows_cursor(false)
            .map_err(|error| anyhow!(error.to_string()))?;
        let filter = SCContentFilter::new().with_display_excluding_windows(&display, &[]);
        let mut stream = SCStream::new(&filter, &config);
        stream.add_output_handler(StreamOutput { sender: tx }, SCStreamOutputType::Screen);
        self.status.store(true, Ordering::SeqCst);
        if let Err(error) = stream.start_capture() {
            self.status.store(false, Ordering::SeqCst);
            return Err(anyhow!(error.to_string()));
        }
        let mut fps_map: HashMap<u64, u32> = HashMap::new();
        let now = Instant::now();
        loop {
            let elapsed = now.elapsed();
            if !self.status.load(Ordering::SeqCst) {
                stream.stop_capture().ok();
                break;
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
                                self.fps.store(*fps, Ordering::SeqCst);
                            }
                        }
                        if fps_map.len() > 3 {
                            let min_key = *fps_map.keys().min().unwrap();
                            fps_map.remove(&min_key);
                        }
                    }
                    let guard = buffer.lock().unwrap();
                    let mut frame = Frame::new(
                        display.width(),
                        display.height(),
                        guard.as_slice().to_vec(),
                        self.config.format,
                    );
                    match self.config.format {
                        Format::RGBA => {
                            bgra_to_rgba(&mut frame.buffer);
                        }
                        _ => {}
                    }
                    (self.on_frame_arrived)(frame, self.fps.load(Ordering::SeqCst));
                }
            }
        }
        Ok(())
    }
    pub fn stop(&self) {
        self.status.store(false, Ordering::SeqCst);
    }
}
