//! Contains implementations of the AsyncMetrics trait

#[cfg(feature = "perf")]
mod cpu_profiler;
#[cfg(feature = "perf")]
pub use cpu_profiler::*;

mod default_profiler;
pub use default_profiler::*;

mod metrics_profiler;
pub use metrics_profiler::*;

#[cfg(feature = "perfetto")]
mod perfetto_profiler;
#[cfg(feature = "perfetto")]
pub use perfetto_profiler::*;
