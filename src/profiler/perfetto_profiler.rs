use crate::Profiler;
use crate::debug_print;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::VecDeque;
use tracing_profile_perfetto_sys::{EventData, TraceEvent};

const MAX_OPEN_TRACKS: usize = 32;
type TrackId = u32;

static TRACK_MANAGER: Lazy<Mutex<TrackManager>> =
    Lazy::new(|| Mutex::new(TrackManager::new(MAX_OPEN_TRACKS)));

thread_local! {
    static TRACK_ID: RefCell<Option<TrackId>> = const { RefCell::new(None) };
    static CALL_DEPTH: RefCell<u32> = const { RefCell::new(0) };
    static FIRST_UPDATE: RefCell<bool> = const { RefCell::new(false) };
}

struct TrackManager {
    available: VecDeque<TrackId>,
}

impl TrackManager {
    fn new(max_tracks: usize) -> Self {
        let available = (0_u32..max_tracks as TrackId).collect();
        Self { available }
    }

    fn get(&mut self) -> Option<TrackId> {
        self.available.pop_back()
    }

    fn release(&mut self, id: TrackId) {
        self.available.push_front(id);
    }
}

pub struct PerfettoProfiler {
    _label: String,
    track_id: Option<TrackId>,
    track_event: Option<TraceEvent>,
    idle_event: Option<TraceEvent>,
}

impl Profiler for PerfettoProfiler {
    fn new(label: &str) -> Self {
        let is_root = CALL_DEPTH.with(|depth| *depth.borrow() == 0);

        let track_id = if is_root {
            TRACK_MANAGER.lock().get()
        } else {
            TRACK_ID.with(|cell| *cell.borrow())
        };

        let mut track_event = None;
        if let Some(track_id) = track_id {
            let mut event = EventData::new(label);
            event.set_track_id(track_id as u64);
            track_event = Some(TraceEvent::new(event));
        }

        debug_print!("new {}.{:?}", label, track_id);
        PerfettoProfiler {
            _label: label.into(),
            track_id,
            track_event,
            idle_event: None,
        }
    }

    fn prepare(&mut self) {
        debug_print!("prepare {}.{:?}", self._label, self.track_id);
        self.idle_event.take();

        let is_root = CALL_DEPTH.with(|depth| *depth.borrow() == 0);
        if is_root && self.track_id.is_some() {
            TRACK_ID.with(|cell| *cell.borrow_mut() = self.track_id);
        }

        CALL_DEPTH.with(|d| *d.borrow_mut() += 1);
        FIRST_UPDATE.with(|d| *d.borrow_mut() = true);
    }

    fn update(&mut self, is_ready: bool) {
        debug_print!("update {}.{:?}", self._label, self.track_id);
        let is_root = CALL_DEPTH.with(|d| {
            let mut b = d.borrow_mut();
            *b -= 1;
            *b == 0
        });

        let is_end = FIRST_UPDATE.with(|d| {
            let b = *d.borrow();
            *d.borrow_mut() = false;
            b
        });

        if is_ready {
            self.track_event.take();
            if is_root {
                if let Some(id) = self.track_id {
                    TRACK_MANAGER.lock().release(id);
                }
            }
        } else if is_end {
            if let Some(track_id) = self.track_id {
                let mut event = EventData::new("idle");
                event.set_track_id(track_id as u64);
                self.idle_event.replace(TraceEvent::new(event));
            }
        }
    }

    fn finish(&self, _label: &str) {
        debug_print!("finish {}.{:?}", _label, self.track_id);
    }

    fn error(&self, label: &str) {
        panic!("future was not polled to completion: {}", label);
    }
}
