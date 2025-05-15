use capture::{Config, Controller, Format, Frame};
use std::time::Duration;
fn main() {
    let config = Config::new(Format::BGRA);
    let on_frame_arrived = Box::new(|frame: Frame, fps| {
        println!(
            "Frame: {}x{}, fps: {}, buffer len: {}, format: {:?}",
            frame.width,
            frame.height,
            fps,
            frame.buffer.len(),
            frame.format
        );
    });
    let mut controller = Controller::new(config, on_frame_arrived);
    controller.start();
    std::thread::sleep(Duration::from_secs(500));
    controller.stop();
}
