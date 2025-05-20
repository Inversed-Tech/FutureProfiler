use crate::Profiler;
use std::time::{Duration, Instant};

pub struct DefaultProfiler {
    wake_time: Duration,
    idle_time: Duration,
    last_poll_time: Option<Instant>,
    last_wait_time: Option<Instant>,
}

impl DefaultProfiler {
    pub fn wake_time(&self) -> Duration {
        self.wake_time
    }

    pub fn idle_time(&self) -> Duration {
        self.idle_time
    }
}

impl Profiler for DefaultProfiler {
    fn new(_label: &str) -> Self {
        Self {
            wake_time: Duration::ZERO,
            idle_time: Duration::ZERO,
            last_poll_time: None,
            last_wait_time: None,
        }
    }

    fn prepare(&mut self) {
        if let Some(last_sleep) = self.last_wait_time.take() {
            self.idle_time += last_sleep.elapsed();
        }
        self.last_poll_time.replace(Instant::now());
    }

    fn update(&mut self, is_ready: bool) {
        if let Some(last_poll) = self.last_poll_time.take() {
            self.wake_time += last_poll.elapsed();
        }

        if !is_ready {
            self.last_wait_time.replace(Instant::now());
        }
    }

    fn finish(&self, label: &str) {
        println!(
            "FutureProfiler: {}, wake_time: {:.3} ms, sleep_time: {:.3} ms",
            label,
            self.wake_time.as_micros() as f64 * 0.001,
            self.idle_time.as_micros() as f64 * 0.001,
        );
    }

    fn error(&self, label: &str) {
        eprintln!("FutureProfiler: {label} was not polled to completion");
    }
}
