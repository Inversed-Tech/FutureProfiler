use crate::Profiler;
use std::time::Duration;

use perf_event::events::Hardware;
use perf_event::{Builder, Counter};

use std::cell::RefCell;

thread_local! {
    static CYCLES: RefCell<Counter> = RefCell::new(
        Builder::new(Hardware::INSTRUCTIONS)
            .build()
            .expect("failed to create perf counter")
    );
}

pub struct CpuProfiler {
    total_instructions: u64,
}

impl CpuProfiler {
    pub fn instructions(&self) -> u64 {
        self.total_instructions
    }
}

impl Profiler for CpuProfiler {
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
            "FutureProfiler: {label}, Executed Instructions: {}, wake_time: {:.3} ms, sleep_time: {:.3} ms",
            self.total_instructions,
            wake_time.as_micros() as f64 * 0.001,
            sleep_time.as_micros() as f64 * 0.001,
        );
    }

    fn error(&self, label: &str) {
        eprintln!("FutureProfiler: {label} was not polled to completion");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FutureProfiler;

    #[tokio::test]
    async fn cpu_prof1() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            (0..1000000).sum::<u64>()
        };

        let profiler = FutureProfiler::<_, _, CpuProfiler>::new("custom_profiler", future);
        let result = profiler.await;
        println!("result: {result}");
    }

    #[tokio::test]
    async fn cpu_prof2() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            1_u64
        };

        let profiler = FutureProfiler::<_, _, CpuProfiler>::new("custom_profiler", future);
        let result = profiler.await;
        println!("result: {result}");
    }
}
