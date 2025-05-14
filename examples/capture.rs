use capture::{Capturer, Config, Format};
fn main() {
    let config = Config::new(Format::BGRA);
    let mut frame = Capturer::screenshot(config).unwrap();
    let image_buffer = frame.to_image_buffer_rgba8().unwrap();
    image_buffer.save("a.png").unwrap();
}
