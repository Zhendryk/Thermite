use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timer<'a> {
    name: &'a str,
    genesis: std::time::Instant,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Self {
        Timer {
            name: name,
            genesis: std::time::Instant::now(),
        }
    }

    pub fn start(&self) {
        unimplemented!()
    }

    pub fn stop(&self) -> std::time::Duration {
        self.genesis.elapsed()
    }

    pub fn now() -> std::time::Instant {
        std::time::Instant::now()
    }

    pub fn elapsed_since(previous: std::time::Instant) -> std::time::Duration {
        let now = std::time::Instant::now();
        now.duration_since(previous)
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {}
}
