use super::DefaultProfiler;
use crate::Profiler;
use metrics_logger::metrics;

pub struct MetricsProfiler {
    default_profiler: DefaultProfiler,
}

impl Profiler for MetricsProfiler {
    fn new(label: &str) -> Self {
        Self {
            default_profiler: DefaultProfiler::new(label),
        }
    }
    fn prepare(&mut self) {
        self.default_profiler.prepare();
    }

    fn update(&mut self, is_ready: bool) {
        self.default_profiler.update(is_ready);
    }

    fn finish(&self, label: &str) {
        let wake_time_ms = self.default_profiler.wake_time().as_micros() as f64 * 0.001;
        let idle_time_ms = self.default_profiler.idle_time().as_micros() as f64 * 0.001;
        metrics::histogram!("wake_time_ms", "label" => label.to_string()).record(wake_time_ms);
        metrics::histogram!("idle_time_ms", "label" => label.to_string()).record(idle_time_ms);
    }

    fn error(&self, label: &str) {
        tracing::error!("FutureProfiler: {} was not polled to completion", label);
    }
}
