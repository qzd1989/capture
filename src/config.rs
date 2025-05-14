use crate::Format;

pub struct Config {
    pub format: Format,
}

impl Config {
    pub fn new(format: Format) -> Self {
        Config { format }
    }
}
