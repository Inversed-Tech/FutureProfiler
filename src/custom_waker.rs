use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{RawWaker, RawWakerVTable, Waker},
};

pub fn with_flag(woken: Arc<AtomicBool>, base: Waker) -> Waker {
    fn into_raw(data: Arc<(AtomicBool, Waker)>) -> RawWaker {
        let ptr = Arc::into_raw(data) as *const ();
        RawWaker::new(ptr, &VTABLE)
    }

    unsafe fn clone(ptr: *const ()) -> RawWaker {
        let arc = Arc::from_raw(ptr as *const (AtomicBool, Waker));
        let cloned = arc.clone(); // bump ref count
        std::mem::forget(arc); // keep original alive
        into_raw(cloned)
    }

    unsafe fn wake(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const (AtomicBool, Waker));
        arc.0.store(true, Ordering::SeqCst);
        arc.1.wake(); // consumes base Waker
        // Arc is dropped here
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const (AtomicBool, Waker));
        arc.0.store(true, Ordering::SeqCst);
        arc.1.wake_by_ref();
        std::mem::forget(arc); // don't drop the Arc
    }

    unsafe fn drop(ptr: *const ()) {
        let _ = Arc::from_raw(ptr as *const (AtomicBool, Waker));
        // dropping Arc here
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    let shared = Arc::new((woken, base));
    unsafe { Waker::from_raw(into_raw(shared)) }
}
