use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct AsyncTracer<T, R>
where
    T: Future<Output = R> + Send,
{
    // used to measure the total time that this future existed
    start: Instant,
    // measures the time spent not sleeping
    duration: Duration,
    // the future of interest. has to be pinned
    future: Pin<Box<T>>,
}

impl<T, R> AsyncTracer<T, R>
where
    T: Future<Output = R> + Send,
{
    pub fn new(future: T) -> Self {
        Self {
            start: Instant::now(),
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
        let start = Instant::now();
        let this = unsafe { self.get_unchecked_mut() };
        let r = this.future.as_mut().poll(cx);
        this.duration += start.elapsed();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::Poll;

    #[tokio::test]
    async fn test_async_tracer_duration1() {
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

    #[tokio::test]
    async fn test_async_tracer_duration2() {
        let future = async {
            std::thread::sleep(Duration::from_millis(101));
            tokio::time::sleep(Duration::from_millis(100)).await;
            std::thread::sleep(Duration::from_millis(10));
            tokio::time::sleep(Duration::from_millis(20)).await;
            std::thread::sleep(Duration::from_millis(30));
            42
        };

        let mut tracer = AsyncTracer::new(future);
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        match Pin::new(&mut tracer).poll(&mut cx) {
            Poll::Pending => {}
            Poll::Ready(_) => panic!("Future should not be ready yet"),
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        loop {
            match Pin::new(&mut tracer).poll(&mut cx) {
                Poll::Ready(_output) => break,
                Poll::Pending => {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
        }

        assert!(tracer.duration <= Duration::from_millis(142));
        assert!(tracer.duration >= Duration::from_millis(141));
    }
}
