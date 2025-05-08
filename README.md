# Future Profiler

The `future-profiler` crate provides a utility for profiling asynchronous Rust code.

## Overview

The `FutureProfiler` struct wraps a future and collects data before and after each invocation of the `poll()` function. It tracks time spent executing and sleeping. Users can implement the `Profiler` trait for additional data collection. The crate includes several `Profiler` implementations that can be composed into custom solutions.

## `Profiler` Trait
- **`new`**: Creates a new instance of the profiler.
- **`prepare`**: Called before polling the future.
- **`update`**: Called after polling the future.
- **`finish`**: Emits the collected metrics when the future is dropped.
- **`error`**: Detects if the future was dropped before completion and emits an error.

## Examples

### Basic Example

```rust
use future_profiler::{FutureProfiler, DefaultProfiler};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let future = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        42
    };

    let profiler = FutureProfiler::<_, _, DefaultProfiler>::new("example_future", future);
    let result = profiler.await;
    println!("Future result: {}", result);
}
```

### Custom Profiler Example

```rust
use future_profiler::{FutureProfiler, Profiler, CpuProfiler};
use std::time::Duration;

struct CustomProfiler {
    cpu_profiler: CpuProfiler,
}

impl Profiler for CustomProfiler {
    fn new() -> Self {
        Self {
            cpu_profiler: CpuProfiler::new(),
        }
    }

    fn prepare(&mut self) {
        self.cpu_profiler.prepare();
    }

    fn update(&mut self) {
        self.cpu_profiler.update();
    }

    fn finish(&self, label: &str, wake_time: Duration, sleep_time: Duration) {
        log::debug!(
            "{label}, wake_time: {}ms, sleep_time: {}ms, cpu_instructions: {}",
            wake_time.as_millis(),
            sleep_time.as_millis(),
            self.cpu_profiler.instructions()
        );
    }

    fn error(&self, label: &str) {
        log::error!("future didn't finish: {label}");
    }
}

#[tokio::main]
async fn main() {
    let future = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        (0..100000).sum::<u64>()
    };

    let profiler = FutureProfiler::<_, _, CustomProfiler>::new("custom_profiler", future);
    let result = profiler.await;
    println!("Future result: {}", result);
}
```

## License

This crate is licensed under the MIT License.  