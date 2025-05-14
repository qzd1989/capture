use crate::{Config, Engine, FrameHandler};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
pub struct Controller<T: FrameHandler> {
    engine: Arc<Engine<T>>,
    handle: Option<JoinHandle<()>>,
}
impl<T: FrameHandler> Controller<T> {
    pub fn new(config: Config, callback: T) -> Self {
        let engine = Arc::new(Engine::new(config, callback));
        Self {
            engine,
            handle: None,
        }
    }
    pub fn start(&mut self) {
        let engine = Arc::clone(&self.engine);
        let handle = thread::spawn(move || {
            engine.start().unwrap();
        });
        self.handle = Some(handle);
    }
    pub fn stop(&mut self) {
        self.engine.stop();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join(); // 可以选择不阻塞
        }
    }
}
