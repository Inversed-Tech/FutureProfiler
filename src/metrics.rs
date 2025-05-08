//! Contains implementations of the AsyncMetrics trait

use std::time::Duration;

use crate::AsyncMetrics;

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
            "AsyncMetrics: {}, Wake Time: {:.3} ms, Sleep Time: {:.3} ms",
            label,
            wake_time.as_micros() as f64 * 0.001,
            sleep_time.as_micros() as f64 * 0.001,
        );
    }

    fn error(&self, label: &str) {
        println!("{label} was dropped early");
    }
}
