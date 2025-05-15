use capture::{Config, Controller, Engine, Format, Frame};
use std::{
    sync::Arc,
    thread::{sleep, spawn},
    time::Duration,
};
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
    let engine = Engine::new(config, on_frame_arrived);
    let engine_clone = Arc::clone(&engine);
    println!("Starting capture...");
    spawn(move || {
        let _ = engine_clone.start();
    });
    sleep(Duration::from_secs(1));
    println!("Stopping capture...");
    engine.stop();
    sleep(Duration::from_secs(2));
    println!("Starting capture again...");
    if let Err(error) = engine.start() {
        println!("Error starting capture: {}", error);
    }
    loop {
        println!("Waitting...");
        sleep(Duration::from_secs(1));
    }
}
