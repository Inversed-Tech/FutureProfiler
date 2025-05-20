use crate::{
    debug_print,
    track_manager::{MULTI_TRACK_MANAGER, MultiTrack},
};
use tracing::span;
use tracing_profile_perfetto_sys::{EventData, TraceEvent};

pub struct PerfettoLayer {}

impl Default for PerfettoLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfettoLayer {
    pub fn new() -> Self {
        Self {}
    }
}

struct PerfettoMetadata {
    track: Option<MultiTrack>,
    root_event: Option<TraceEvent>,
    trace_event: Option<TraceEvent>,
    call_depth: usize,
}

impl PerfettoMetadata {
    fn public(&self) -> PublicMetadata {
        PublicMetadata {
            track: self.track,
            call_depth: self.call_depth,
        }
    }
}

struct PublicMetadata {
    track: Option<MultiTrack>,
    call_depth: usize,
}

impl Drop for PerfettoMetadata {
    fn drop(&mut self) {
        if self.call_depth == 0 {
            if let Some(mt) = self.track {
                self.root_event.take();
                debug_print!("drop {:?}", mt);
                MULTI_TRACK_MANAGER.lock().release(mt);
            }
        }
    }
}

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
        _attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let Some(span) = ctx.span(id) else {
            eprintln!("failed to get span");
            return;
        };

        let parent_meta = span.parent().and_then(|parent| {
            let extensions = parent.extensions();
            extensions.get::<PerfettoMetadata>().map(|x| x.public())
        });

        let (track, call_depth) = match parent_meta {
            Some(meta) => (meta.track, meta.call_depth + 1),
            None => {
                let track_type = match span.name() {
                    "search_task" => Some(0),
                    "match_task" => Some(1),
                    _ => None,
                };
                (
                    track_type.and_then(|t| MULTI_TRACK_MANAGER.lock().get(t)),
                    0,
                )
            }
        };

        debug_print!("new_span {}.{:?}", span.name(), track);

        let mut root_event = None;
        if call_depth == 0 {
            if let Some(mt) = track {
                let track_name = format!("track_{}", mt.id);
                let mut event = EventData::new(&track_name);
                event.set_track_id(mt.id as u64);
                root_event.replace(TraceEvent::new(event));
            }
        }

        let meta = PerfettoMetadata {
            track,
            call_depth,
            trace_event: None,
            root_event,
        };

        let mut extensions = span.extensions_mut();
        extensions.insert(meta);
    }

    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            eprintln!("failed to get span");
            return;
        };

        let mut extensions = span.extensions_mut();
        let Some(meta) = extensions.get_mut::<PerfettoMetadata>() else {
            eprintln!("failed to get meta");
            return;
        };

        debug_print!("on_enter: {}.{:?}", span.name(), meta.track);

        let trace_event = match meta.track {
            Some(mt) => {
                let mut event = EventData::new(span.name());
                event.set_track_id(mt.id as u64);
                Some(TraceEvent::new(event))
            }
            None => None,
        };

        meta.trace_event = trace_event;
    }

    fn on_exit(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            eprintln!("failed to get span");
            return;
        };

        let mut extensions = span.extensions_mut();
        let Some(meta) = extensions.get_mut::<PerfettoMetadata>() else {
            eprintln!("failed to get meta");
            return;
        };

        debug_print!("on_exit: {}.{:?}", span.name(), meta.track);

        meta.trace_event.take();
    }
}
