use std::time::Instant;

use capture::{Config, Controller, Format};

fn main() {
    let now = Instant::now();
    let config = Config::new(Format::BGRA);
    println!("config took {:?}", now.elapsed());
    let mut frame = Controller::grab(config).unwrap();
    println!("grab took {:?}", now.elapsed());
    frame
        .to_image_buffer_rgba8()
        .unwrap()
        .save("screenshot.png")
        .unwrap();
    println!("save took {:?}", now.elapsed());
}
