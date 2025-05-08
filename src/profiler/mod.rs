//! Contains implementations of the AsyncMetrics trait

use crate::Profiler;
use std::time::Duration;

#[cfg(feature = "perf")]
mod cpu_profiler;
#[cfg(feature = "perf")]
pub use cpu_profiler::*;

pub struct DefaultMetrics {}

impl Profiler for DefaultMetrics {
    fn new() -> Self {
        Self {}
    }

    fn prepare(&mut self) {
        // Do nothing
    }

    fn update(&mut self) {
        // Do nothing
    }

    fn finish(&self, label: &str, wake_time: Duration, sleep_time: Duration) {
        println!(
            "FutureProfiler: {}, wake_time: {:.3} ms, sleep_time: {:.3} ms",
            label,
            wake_time.as_micros() as f64 * 0.001,
            sleep_time.as_micros() as f64 * 0.001,
        );
    }

    fn error(&self, label: &str) {
        eprintln!("FutureProfiler: {label} was not polled to completion");
    }
}
