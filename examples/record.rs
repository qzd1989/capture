use capture::{Config, Engine, Format, Frame, FrameHandler};
use std::{thread::sleep, time::Duration};
fn main() {
    let frame_handler: FrameHandler = Box::new(|frame: Frame| {
        println!(
            "Frame: {}x{}, buffer len: {}, format: {:?}",
            frame.width,
            frame.height,
            frame.buffer.len(),
            frame.format
        );
    });
    let config = Config::new(Format::RGBA);
    let engine = Engine::new(config, frame_handler);
    println!("start");
    engine.start_background().expect("Engine start failed.");
    sleep(Duration::from_millis(3000));
    engine.stop();
    println!("finished");
    sleep(Duration::from_millis(3000));
    println!("start");
    engine.start_background().expect("Engine start failed.");
    sleep(Duration::from_millis(3000));
    engine.stop();
    println!("finished");
}
