use capture::{Config, Controller, Format};
use std::time::Instant;
fn main() {
    let now = Instant::now();
    let config = Config::new(Format::BGRA);
    println!("config took {:?}", now.elapsed());
    let mut frame = Controller::grab(config).unwrap();
    println!(
        "frame size: ({}, {}), frame data len: {}",
        frame.width,
        frame.height,
        frame.buffer.len()
    );
    println!("grab took {:?}", now.elapsed());
    frame.save("screenshot.png").unwrap();
    println!("save took {:?}", now.elapsed());
}
