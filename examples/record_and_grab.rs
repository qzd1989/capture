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
    println!("record start");
    engine.start_background().expect("Engine start failed.");
    sleep(Duration::from_millis(3000));
    engine.stop();
    println!("record finished");
    sleep(Duration::from_millis(3000));
    if let Ok(frame) = engine.grab(1000) {
        frame.save("record_and_grab.png").unwrap();
    }
    println!("grab finished");
    sleep(Duration::from_millis(3000));
    println!("record start");
    engine.start_background().expect("Engine start failed.");
    sleep(Duration::from_millis(3000));
    engine.stop();
    println!("finished");
    sleep(Duration::from_millis(100000));
}
