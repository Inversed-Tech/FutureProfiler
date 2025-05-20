use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::VecDeque;

const MAX_OPEN_TRACKS: usize = 32;
const MAX_TRACK_TYPES: usize = 2;
pub(crate) type TrackId = u32;
pub(crate) type TrackType = usize;

pub(crate) static TRACK_MANAGER: Lazy<Mutex<TrackManager>> =
    Lazy::new(|| Mutex::new(TrackManager::new(MAX_OPEN_TRACKS)));

pub(crate) static MULTI_TRACK_MANAGER: Lazy<Mutex<MultiTrackManager>> = Lazy::new(|| {
    Mutex::new(MultiTrackManager::new(
        MAX_OPEN_TRACKS / MAX_TRACK_TYPES,
        MAX_TRACK_TYPES,
    ))
});

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct MultiTrack {
    pub id: TrackId,
    pub track: TrackType,
}

pub(crate) struct MultiTrackManager {
    available: Vec<VecDeque<TrackId>>,
}

impl MultiTrackManager {
    pub(crate) fn new(max_tracks: usize, max_track_types: usize) -> Self {
        let chunk_size = max_tracks / max_track_types;
        let available = (0..max_tracks * max_track_types)
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .map(|chunk| chunk.iter().map(|x| *x as u32).collect())
            .collect();

        Self { available }
    }

    pub(crate) fn get(&mut self, track: TrackType) -> Option<MultiTrack> {
        self.available[track]
            .pop_back()
            .map(|id| MultiTrack { id, track })
    }

    pub(crate) fn release(&mut self, mt: MultiTrack) {
        self.available[mt.track].push_front(mt.id);
    }
}
