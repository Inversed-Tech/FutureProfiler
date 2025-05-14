pub use tracing_profile_perfetto_sys::{BackendConfig, Error, PerfettoGuard};

/// Creates an in-process backend. Perfetto TraceEvents will be collected only while the guard is active.
pub fn perfetto_guard(buffer_size_kb: usize, output_path: &str) -> Result<PerfettoGuard, Error> {
    PerfettoGuard::new(BackendConfig::InProcess { buffer_size_kb }, output_path)
}
