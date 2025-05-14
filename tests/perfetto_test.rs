#![cfg(feature = "perfetto")]
#![cfg(not(feature = "subscriber"))]
use future_profiler::{instrument_fut, perfetto_guard};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use tokio::time::{Duration, sleep};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn perfetto_test1() {
    tracing::debug!("starting perfetto tracing test");

    let home_dir = std::env::var("HOME").expect("Failed to get home directory");
    let downloads_dir = std::path::Path::new(&home_dir).join("Downloads");
    let trace_file_path = downloads_dir.join("perfetto_profiler.perfetto_trace");
    let trace_file_path = trace_file_path
        .to_str()
        .expect("Failed to convert path to string")
        .to_string();

    let _guard = perfetto_guard(8192, &trace_file_path).unwrap();

    let mut handles = FuturesUnordered::new();
    for i in 0..16 {
        let handle = tokio::spawn(async move {
            match i % 4 {
                0 => instrument_fut!("scenario1"; scenario1()).await,
                1 => instrument_fut!("scenario2"; scenario2()).await,
                2 => instrument_fut!("scenario3"; scenario3()).await,
                3 => instrument_fut!("scenario4"; scenario4()).await,
                _ => unreachable!(),
            }
        });
        handles.push(handle);
    }

    while let Some(_) = handles.next().await {}
    tracing::debug!("ending perfetto tracing test");
}

const SUM_TO: usize = 10000000;

fn sum_to(x: usize) -> usize {
    (0..x).sum::<usize>()
}

async fn scenario1() {
    instrument_fut!("do_work"; async {
        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;

        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;

        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;
    })
    .await;
}

async fn scenario2() {
    instrument_fut!("scenario2_1"; async {
        scenario2_1().await;
    })
    .await;
}

async fn scenario2_1() {
    instrument_fut!("do_work"; async {
        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;

        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;

        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;
    })
    .await;
}

async fn scenario3() {
    instrument_fut!("scenario3_1"; async { scenario3_1().await;}).await;
    let _ = sum_to(SUM_TO);

    instrument_fut!("scenario3_1"; async { scenario3_1().await;}).await;
    let _ = sum_to(SUM_TO);
}

async fn scenario3_1() {
    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;
    instrument_fut!("scenario3_2"; async { scenario3_2().await;}).await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;
}

async fn scenario3_2() {
    instrument_fut!("do_work"; async {
        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;

        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(1)).await;
    })
    .await;
}

async fn scenario4() {
    for _ in 0..3 {
        instrument_fut!("scenario4_1"; async { scenario4_1().await;} ).await;
    }
}

async fn scenario4_1() {
    for _ in 0..2 {
        instrument_fut!("scenario4_2"; async { scenario4_2().await;}).await;
    }
}

async fn scenario4_2() {
    instrument_fut!( "do_work"; async {
        let _ = sum_to(SUM_TO);
        sleep(Duration::from_millis(500)).await;
    })
    .await;
}
