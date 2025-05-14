use capture::{Config, Format, Recorder};
use std::time::Duration;
fn main() {
    let config = Config::new(Format::BGRA);
    let mut recorder = Recorder::new(config, |frame, fps| {
        println!(
            "Frame: {}x{}, fps: {}, data: {}, format: {:?}",
            frame.width,
            frame.height,
            fps,
            frame.buffer.len(),
            frame.format
        );
    });
    recorder.start();
    std::thread::sleep(Duration::from_secs(500));
    recorder.stop();
}
