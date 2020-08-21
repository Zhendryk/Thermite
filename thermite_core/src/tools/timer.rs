use std::time::{Duration, Instant};

pub struct Time {
    start: Instant,
    last_tick: Option<Instant>,
    delta: Duration,
    delta_sec: f32,
    delta_sec_f64: f64,
    seconds_since_start: f64,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            last_tick: None,
            delta: Duration::from_secs(0),
            delta_sec: 0.0,
            delta_sec_f64: 0.0,
            seconds_since_start: 0.0,
        }
    }
}

impl Time {
    pub fn tick(&mut self) {
        let tick = Instant::now();
        if let Some(last_tick) = self.last_tick {
            self.delta = tick - last_tick;
            self.delta_sec = self.delta.as_secs_f32();
            self.delta_sec_f64 = self.delta.as_secs_f64();
        }
        let duration_since_start = tick - self.start;
        self.seconds_since_start = duration_since_start.as_secs_f64();
        self.last_tick = Some(tick);
    }

    pub fn time_elapsed_since_start(&self) -> Duration {
        Instant::now() - self.start
    }
}

pub enum TimerMagnitude {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
}

pub struct Timer {
    pub magnitude: TimerMagnitude,
    pub elapsed: f32,
    pub duration: f32,
    pub finished: bool,
}

impl Timer {
    pub fn new(duration: f32, magnitude: TimerMagnitude) -> Self {
        Self {
            magnitude: magnitude,
            elapsed: 0.0,
            duration: duration,
            finished: false,
        }
    }

    pub fn from_nanoseconds(duration: f32) -> Self {
        Self::new(duration, TimerMagnitude::Nanosecond)
    }

    pub fn from_microseconds(duration: f32) -> Self {
        Self::new(duration, TimerMagnitude::Microsecond)
    }

    pub fn from_milliseconds(duration: f32) -> Self {
        Self::new(duration, TimerMagnitude::Millisecond)
    }

    pub fn from_seconds(duration: f32) -> Self {
        Self::new(duration, TimerMagnitude::Second)
    }

    pub fn tick(&mut self, delta: f32) {
        self.elapsed = (self.elapsed + delta).min(self.duration);
        if self.elapsed >= self.duration {
            self.finished = true;
        }
    }

    pub fn reset(&mut self) {
        self.finished = false;
        self.elapsed = 0.0;
    }
}
