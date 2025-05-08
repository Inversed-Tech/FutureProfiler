use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

mod custom_waker;

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
        let wrapped_waker = custom_waker::with_flag(was_woken.clone(), cx.waker().clone());
        let mut new_cx = Context::from_waker(&wrapped_waker);

        let start = Instant::now();
        let this = unsafe { self.get_unchecked_mut() };
        let r = this.future.as_mut().poll(&mut new_cx);
        if !matches!(r, Poll::Pending) || was_woken.load(Ordering::SeqCst) {
            let elapsed = start.elapsed();
            this.duration += elapsed;
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::Poll;

    #[tokio::test]
    async fn test_async_tracer_duration() {
        let future = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            std::thread::sleep(Duration::from_millis(101));
            42
        };

        let mut tracer = AsyncTracer::new(future);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        match Pin::new(&mut tracer).poll(&mut cx) {
            Poll::Pending => {}
            Poll::Ready(_) => panic!("Future should not be ready yet"),
        }

        tokio::time::sleep(Duration::from_millis(150)).await;

        match Pin::new(&mut tracer).poll(&mut cx) {
            Poll::Ready(output) => assert_eq!(output, 42),
            Poll::Pending => panic!("Future should be ready now"),
        }

        assert!(tracer.duration <= Duration::from_millis(103));
        assert!(tracer.duration >= Duration::from_millis(101));
    }
}
