use capture::{Config, Controller, Format, Frame};
use std::{thread::sleep, time::Duration};
fn main() {
    let config = Config::new(Format::BGRA);
    let on_frame_arrived = Box::new(|frame: Frame, fps| {
        // println!(
        //     "Frame: {}x{}, fps: {}, buffer len: {}, format: {:?}",
        //     frame.width,
        //     frame.height,
        //     fps,
        //     frame.buffer.len(),
        //     frame.format
        // );
    });
    let mut controller = Controller::new(config, on_frame_arrived);
    println!("Starting capture...");
    controller.start();
    sleep(Duration::from_secs(5));
    println!("Stopping capture...");
    controller.stop();
    sleep(Duration::from_secs(5));
    println!("Starting capture again...");
    controller.start();
    loop {
        println!("running status: {}", controller.is_running());
        sleep(Duration::from_secs(1));
    }
}
