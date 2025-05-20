// coded with chat-gpt

#[macro_export]
macro_rules! __instrument_span {
    (trace, $name:literal $(, $fields:tt)*) => {
        ::tracing::trace_span!($name $(, $fields)*)
    };
    (debug, $name:literal $(, $fields:tt)*) => {
        ::tracing::debug_span!($name $(, $fields)*)
    };
    (info, $name:literal $(, $fields:tt)*) => {
        ::tracing::info_span!($name $(, $fields)*)
    };
    (warn, $name:literal $(, $fields:tt)*) => {
        ::tracing::warn_span!($name $(, $fields)*)
    };
    (error, $name:literal $(, $fields:tt)*) => {
        ::tracing::error_span!($name $(, $fields)*)
    };
}

#[macro_export]
macro_rules! instrument_fut {
    ($name:literal ; $call:expr) => {{
        #[cfg(feature = "perfetto")]
        {
            $crate::FutureProfiler::<_, _, $crate::PerfettoProfiler>::new($name, $call)
        }
        #[cfg(feature = "metrics")]
        {
            $crate::FutureProfiler::<_, _, $crate::MetricsProfiler>::new($call, $call)
        }
        #[cfg(not(any(feature = "perfetto", feature = "metrics")))]
        {
            $call
        }
    }};
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug_test")]
        {
            println!($($arg)*);
        }
    };
}
