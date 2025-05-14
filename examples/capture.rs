use capture::{Config, Controller, Format};
use std::time::Duration;

fn main() {
    let config = Config::new(Format::BGRA);
    let mut controller = Controller::new(config, |frame, fps| {
        println!("Frame: {}x{}, fps: {}", frame.width, frame.height, fps);
    });
    controller.start();
    std::thread::sleep(Duration::from_secs(5));
    controller.stop();
}
