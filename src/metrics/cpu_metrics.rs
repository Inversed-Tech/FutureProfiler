use crate::AsyncMetrics;
use std::time::Duration;

use perf_event::events::Hardware;
use perf_event::{Builder, Counter};

use std::cell::RefCell;

thread_local! {
    static CYCLES: RefCell<Counter> = RefCell::new(
        Builder::new()
            .kind(Hardware::INSTRUCTIONS)
            .build()
            .expect("failed to create perf counter")
    );
}

pub struct CpuMetrics {
    total_instructions: u64,
}

impl CpuMetrics {
    pub fn instructions(&self) -> u64 {
        self.total_instructions
    }
}

impl AsyncMetrics for CpuMetrics {
    fn new() -> Self {
        Self {
            total_instructions: 0,
        }
    }

    fn prepare(&mut self) {
        CYCLES.with(|counter_cell| {
            let mut counter = counter_cell.borrow_mut();
            counter.reset().unwrap();
            counter.enable().unwrap();
        });
    }

    fn update(&mut self) {
        CYCLES.with(|counter_cell| {
            let mut counter = counter_cell.borrow_mut();
            counter.disable().unwrap();
            self.total_instructions += counter.read().unwrap();
        });
    }

    fn finish(&self, label: &str, wake_time: Duration, sleep_time: Duration) {
        println!(
            "AsyncMetrics: {label}, Executed Instructions: {}, wake_time: {:.3} ms, sleep_time: {:.3} ms",
            self.total_instructions,
            wake_time.as_micros() as f64 * 0.001,
            sleep_time.as_micros() as f64 * 0.001,
        );
    }

    fn error(&self, label: &str) {
        eprintln!("AsyncMetrics: {label} was not polled to completion");
    }
}
