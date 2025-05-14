use crate::debug_print;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::VecDeque;
use tracing::{
    field::{Field, Visit},
    span,
};
use tracing_profile_perfetto_sys::{EventData, TraceEvent};

pub struct PerfettoLayer {}

impl PerfettoLayer {
    pub fn new() -> Self {
        Self {}
    }
}

const MAX_OPEN_TRACKS: usize = 32;
type TrackId = u32;

static TRACK_MANAGER: Lazy<Mutex<TrackManager>> =
    Lazy::new(|| Mutex::new(TrackManager::new(MAX_OPEN_TRACKS)));

struct TrackManager {
    available: VecDeque<TrackId>,
}

impl TrackManager {
    fn new(max_tracks: usize) -> Self {
        let available = (0_u32..max_tracks as u32).collect();
        Self { available }
    }

    fn get(&mut self) -> Option<TrackId> {
        self.available.pop_back()
    }

    fn release(&mut self, id: TrackId) {
        self.available.push_front(id);
    }
}

struct PerfettoMetadata {
    track_id: Option<TrackId>,
    root_event: Option<TraceEvent>,
    trace_event: Option<TraceEvent>,
    call_depth: usize,
}

impl PerfettoMetadata {
    fn public(&self) -> PublicMetadata {
        PublicMetadata {
            track_id: self.track_id,
            call_depth: self.call_depth,
        }
    }
}

struct PublicMetadata {
    track_id: Option<TrackId>,
    call_depth: usize,
}

impl Drop for PerfettoMetadata {
    fn drop(&mut self) {
        if self.call_depth == 0 {
            if let Some(track_id) = self.track_id {
                self.root_event.take();
                debug_print!("drop {:?}", track_id);
                TRACK_MANAGER.lock().release(track_id);
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
        attrs: &span::Attributes<'_>,
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

        let (track_id, call_depth) = match parent_meta {
            Some(meta) => (meta.track_id, meta.call_depth + 1),
            None => {
                let track_id = TRACK_MANAGER.lock().get();
                (track_id, 0)
            }
        };

        debug_print!("new_span {}.{:?}", span.name(), track_id);

        let mut root_event = None;
        if call_depth == 0 {
            if let Some(id) = track_id {
                let track_name = format!("track_{}", id);
                let mut event = EventData::new(&track_name);
                event.set_track_id(id as u64);
                root_event.replace(TraceEvent::new(event));
            }
        }

        let meta = PerfettoMetadata {
            track_id,
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

        debug_print!("on_enter: {}.{:?}", span.name(), meta.track_id);

        let trace_event = match meta.track_id {
            Some(id) => {
                let mut event = EventData::new(&span.name());
                event.set_track_id(id as u64);
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

        debug_print!("on_exit: {}.{:?}", span.name(), meta.track_id);

        meta.trace_event.take();
    }
}
