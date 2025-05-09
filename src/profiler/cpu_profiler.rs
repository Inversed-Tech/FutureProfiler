use crate::Profiler;
use std::time::Duration;

use perf_event::events::Hardware;
use perf_event::{Builder, Counter};

use std::cell::RefCell;

struct StaticVars {
    counter: Counter,
    stack_depth: usize,
}

impl StaticVars {
    fn new(counter: Counter) -> Self {
        Self {
            counter,
            stack_depth: 0,
        }
    }

    // for re-entrancy, need to return the current hardware counter value.
    fn prepare(&mut self) -> u64 {
        if self.stack_depth == 0 {
            self.counter.reset().unwrap();
            self.counter.enable().unwrap();
        }
        self.stack_depth += 1;
        self.counter.read().unwrap()
    }

    // for re-entrancy, only disable the hardware counter when the stack depth
    // is zero.
    fn update(&mut self) -> u64 {
        assert!(
            self.stack_depth > 0,
            "CPU profiler failed. this error should be unreachable"
        );
        self.stack_depth -= 1;
        if self.stack_depth == 0 {
            self.counter.disable().unwrap();
        }
        self.counter.read().unwrap()
    }
}

thread_local! {
    static GV: RefCell<StaticVars> = RefCell::new(StaticVars::new(
        Builder::new(Hardware::INSTRUCTIONS)
            .build()
            .expect("failed to create perf counter"))
    );
}

/// This CpuProfiler is intended for local testing and profiling. Counting hardware instructions is likely too
/// expensive to use in production. With this assumption, this code will panic if the hardware counter fails
/// to start, stop, or read a value.
/// The CpuProfiler is re-entrant - it can be used to profile nested futures on the same thread.
pub struct CpuProfiler {
    total_instructions: u64,
    instruction_start: u64,
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
            instruction_start: 0,
        }
    }

    fn prepare(&mut self) {
        GV.with(|gv| {
            let mut gv = gv.borrow_mut();
            self.instruction_start = gv.prepare();
        });
    }

    fn update(&mut self) {
        GV.with(|gv| {
            let mut gv = gv.borrow_mut();
            self.total_instructions += gv.update() - self.instruction_start;
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

    #[tokio::test]
    async fn cpu_prof_nested() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let future2 = async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                (0..1000000).sum::<u64>()
            };
            let profiler = FutureProfiler::<_, _, CpuProfiler>::new("nested_future", future2);
            let result = profiler.await;
            println!("result: {result}");
            (0..1000000).sum::<u64>()
        };

        let profiler = FutureProfiler::<_, _, CpuProfiler>::new("outer_future", future);
        let result = profiler.await;
        println!("result: {result}");
    }
}
