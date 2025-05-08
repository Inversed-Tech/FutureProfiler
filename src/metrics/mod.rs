//! Contains implementations of the AsyncMetrics trait

use crate::AsyncMetrics;
use std::time::Duration;

#[cfg(feature = "cpu-metrics")]
mod cpu_metrics;
#[cfg(feature = "cpu-metrics")]
pub use cpu_metrics::*;

pub struct DefaultMetrics {}

impl AsyncMetrics for DefaultMetrics {
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
            "AsyncMetrics: {}, wake_time: {:.3} ms, sleep_time: {:.3} ms",
            label,
            wake_time.as_micros() as f64 * 0.001,
            sleep_time.as_micros() as f64 * 0.001,
        );
    }

    fn error(&self, label: &str) {
        eprintln!("AsyncMetrics: {label} was not polled to completion");
    }
}
