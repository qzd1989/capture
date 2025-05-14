// use windows_capture::{
//     capture::{Context, GraphicsCaptureApiHandler},
//     encoder::{AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder},
//     frame::Frame,
//     graphics_capture_api::InternalCaptureControl,
//     monitor::Monitor,
//     settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
// };
// struct Capture {}
// impl GraphicsCaptureApiHandler for Capture {
//     type Flags = String;
//     type Error = Box<dyn std::error::Error + Send + Sync>;
//     fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
//         Ok(Self {})
//     }
//     fn on_frame_arrived(
//         &mut self,
//         frame: &mut Frame,
//         capture_control: InternalCaptureControl,
//     ) -> Result<(), Self::Error> {
//         print!(
//             "\rRecording for: {} seconds",
//             self.start.elapsed().as_secs()
//         );
//         io::stdout().flush()?;

//         let mut data = frame.buffer()?;
//         dbg!((
//             data.as_raw_buffer().len(),
//             data.width(),
//             data.height(),
//             (data.as_raw_buffer().len() / 4) / (data.width() as usize)
//         ));
//         data.save_as_image("a.png", windows_capture::frame::ImageFormat::Png)
//             .unwrap();

//         capture_control.stop();

//         Ok(())
//     }

//     // Optional handler called when the capture item (usually a window) closes.
//     fn on_closed(&mut self) -> Result<(), Self::Error> {
//         println!("Capture session ended");

//         Ok(())
//     }
// }
// pub struct Engine {}

// impl Engine {
//     pub fn new() -> Self {
//         Engine {}
//     }

//     pub fn start(&self) {
//         // Start the engine
//         println!("Engine started");
//     }

//     pub fn stop(&self) {
//         // Stop the engine
//         println!("Engine stopped");
//     }
// }
