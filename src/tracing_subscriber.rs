use tracing::{
    field::{Field, Visit},
    span,
};

use crate::{PerfettoProfiler, Profiler};

pub struct PerfettoLayer {}

impl<S> tracing_subscriber::Layer<S> for PerfettoLayer
where
    S: tracing::Subscriber,
    // this lets you access the parent span.
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
    }

    fn on_record(
        &self,
        _id: &span::Id,
        _values: &span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match ctx.span(id) {
            Some(span) => {
                let profiler = PerfettoProfiler::new(&span.name());
                let mut extensions = span.extensions_mut();
                extensions.insert(profiler);
            }
            None => {
                eprintln!("failed to get span on_enter");
                return;
            }
        };
    }

    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            eprintln!("failed to get span");
            return;
        };

        let mut extensions = span.extensions_mut();
        let Some(profiler) = extensions.get_mut::<PerfettoProfiler>() else {
            eprintln!("failed to get profiler");
            return;
        };
        profiler.prepare();
    }

    fn on_exit(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            eprintln!("failed to get span");
            return;
        };

        let mut extensions = span.extensions_mut();
        let Some(profiler) = extensions.get_mut::<PerfettoProfiler>() else {
            eprintln!("failed to get profiler");
            return;
        };
        // there is no way to tell if the future polled to completion.
        profiler.update(true);
    }
}
