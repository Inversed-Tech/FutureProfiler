use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::VecDeque;

const MAX_OPEN_TRACKS: usize = 32;
pub(crate) type TrackId = u32;

pub(crate) static TRACK_MANAGER: Lazy<Mutex<TrackManager>> =
    Lazy::new(|| Mutex::new(TrackManager::new(MAX_OPEN_TRACKS)));

pub(crate) struct TrackManager {
    available: VecDeque<TrackId>,
}

impl TrackManager {
    pub(crate) fn new(max_tracks: usize) -> Self {
        let available = (0_u32..max_tracks as u32).collect();
        Self { available }
    }

    pub(crate) fn get(&mut self) -> Option<TrackId> {
        self.available.pop_back()
    }

    pub(crate) fn release(&mut self, id: TrackId) {
        self.available.push_front(id);
    }
}
