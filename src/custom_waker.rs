use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{RawWaker, RawWakerVTable, Waker},
};

pub fn with_flag(woken: Arc<AtomicBool>, base: Waker) -> Waker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        let (flag_ptr, base_ptr): (Arc<AtomicBool>, Waker) =
            unsafe { (*(data as *const (Arc<AtomicBool>, Waker))).clone() };
        into_raw_waker(flag_ptr, base_ptr)
    }

    unsafe fn wake(data: *const ()) {
        let (flag, base): &(Arc<AtomicBool>, Waker) =
            unsafe { &*(data as *const (Arc<AtomicBool>, Waker)) };
        flag.store(true, Ordering::SeqCst);
        base.wake_by_ref();
    }

    unsafe fn drop(_: *const ()) {}

    fn into_raw_waker(flag: Arc<AtomicBool>, base: Waker) -> RawWaker {
        let data = Box::into_raw(Box::new((flag, base))) as *const ();
        RawWaker::new(data, &VTABLE)
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

    let raw = into_raw_waker(woken.clone(), base);
    unsafe { Waker::from_raw(raw) }
}
