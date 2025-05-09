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

    fn prepare(&mut self) -> u64 {
        if self.stack_depth == 0 {
            self.counter.reset().unwrap();
            self.counter.enable().unwrap();
        }
        self.stack_depth += 1;
        self.counter.read().unwrap()
    }

    fn update(&mut self) -> u64 {
        assert!(self.stack_depth > 0);
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

// this is designed to be re-entrant. If nested futures are profiled on the same thread with this profiler,
// the instruction counter can't be disabled until the last future is finished. it also isn't enough
// to simply read the hardware counter because after the outermost future, the counter won't start at zero.
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
}
