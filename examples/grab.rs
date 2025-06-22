use std::time::Instant;

use capture::{Config, Engine, Format, Frame, FrameHandler};
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
    let now = Instant::now();
    if let Ok(frame) = engine.grab(1000) {
        frame.save("screenshoot.png").unwrap();
    }
    println!("finished: {:#?}", now.elapsed());
}
