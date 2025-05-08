use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

mod waker;

pub struct AsyncTracer<T, R>
where
    T: Future<Output = R> + Send,
{
    duration: Duration,
    future: Pin<Box<T>>,
}

impl<T, R> AsyncTracer<T, R>
where
    T: Future<Output = R> + Send,
{
    pub fn new(future: T) -> Self {
        Self {
            duration: Duration::ZERO,
            future: Box::pin(future),
        }
    }
}

impl<T, R> Future for AsyncTracer<T, R>
where
    T: Future<Output = R> + Send,
{
    type Output = R;

    /// # Safety
    ///  The `this` variable must not have any data moved out of it.
    ///  It also must not be invalidated.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let was_woken = Arc::new(AtomicBool::new(false));
        let wrapped_waker = waker::waker_with_flag(was_woken.clone(), cx.waker().clone());
        let mut new_cx = Context::from_waker(&wrapped_waker);

        let start = Instant::now();
        let this = unsafe { self.get_unchecked_mut() };
        let r = match this.future.as_mut().poll(&mut new_cx) {
            Poll::Ready(output) => Poll::Ready(output),
            Poll::Pending => Poll::Pending,
        };
        if was_woken.load(Ordering::SeqCst) {
            let elapsed = start.elapsed();
            this.duration += elapsed;
        }
        r
    }
}
