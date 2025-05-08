use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

mod metrics;
pub use metrics::*;

pub trait AsyncMetrics {
    fn new() -> Self;
    /// called before poll
    fn prepare(&mut self);
    /// called after poll
    fn update(&mut self);
    /// logs the metrics. it takes as arguments the metrics collected by the AsyncTracer too.
    fn finish(&self, label: &str, wake_time: Duration, sleep_time: Duration);
    /// called when the future is dropped early
    fn error(&self, label: &str);
}

pub struct AsyncTracer<T, R, M>
where
    T: Future<Output = R> + Send,
    M: AsyncMetrics,
{
    label: String,
    // used to calculate sleep_time
    start: Instant,
    wake_time: Duration,
    sleep_time: Option<Duration>,
    user_metrics: M,
    // the future of interest. has to be pinned
    future: Pin<Box<T>>,
}

impl<T, R, M> AsyncTracer<T, R, M>
where
    T: Future<Output = R> + Send,
    M: AsyncMetrics,
{
    pub fn new<S: Into<String>>(label: S, future: T) -> Self {
        Self {
            label: label.into(),
            user_metrics: M::new(),
            start: Instant::now(),
            wake_time: Duration::ZERO,
            sleep_time: None,
            future: Box::pin(future),
        }
    }
}

impl<T, R, M> Drop for AsyncTracer<T, R, M>
where
    T: Future<Output = R> + Send,
    M: AsyncMetrics,
{
    fn drop(&mut self) {
        // if self.sleep_time is None then the future was not polled to completion.
        if let Some(sleep_time) = self.sleep_time {
            self.user_metrics
                .finish(&self.label, self.wake_time, sleep_time);
        } else {
            self.user_metrics.error(&self.label);
        }
    }
}

impl<T, R, M> Future for AsyncTracer<T, R, M>
where
    T: Future<Output = R> + Send,
    M: AsyncMetrics,
{
    type Output = R;

    /// # Safety
    ///  The `this` variable must not have any data moved out of it.
    ///  It also must not be invalidated.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let poll_start = Instant::now();
        let this = unsafe { self.get_unchecked_mut() };

        this.user_metrics.prepare();
        let r = this.future.as_mut().poll(cx);
        let elapsed = poll_start.elapsed();
        this.user_metrics.update();
        this.wake_time += elapsed;

        // update sleep_time when the future is completed. this could be done on drop but
        // if the caller doesn't drop the future, then sleep_time could be misreported.
        if !matches!(r, Poll::Pending) {
            this.sleep_time
                .replace(this.start.elapsed() - this.wake_time);
        }

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

        let mut tracer = AsyncTracer::<_, _, DefaultMetrics>::new("waiter", future);
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

        assert!(tracer.wake_time <= Duration::from_millis(103));
        assert!(tracer.wake_time >= Duration::from_millis(101));
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

        let mut tracer = AsyncTracer::<_, _, DefaultMetrics>::new("waiter", future);
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

        assert!(tracer.wake_time <= Duration::from_millis(142));
        assert!(tracer.wake_time >= Duration::from_millis(141));
    }
}
