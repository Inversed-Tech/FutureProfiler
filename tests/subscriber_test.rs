#![cfg(feature = "subscriber")]
#![cfg(feature = "perfetto")]

use future_profiler::{PerfettoLayer, instrument_fut, perfetto_guard};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use tokio::time::{Duration, sleep};
use tracing::Level;
use tracing::instrument;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn perfetto_test1() {
    let perfetto_layer = PerfettoLayer {};
    let fmt_layer = fmt::layer().with_ansi(false);
    tracing_subscriber::registry()
        .with(perfetto_layer)
        .with(fmt_layer)
        .init();

    tracing::debug!("starting perfetto subscriber test");

    let home_dir = std::env::var("HOME").expect("Failed to get home directory");
    let downloads_dir = std::path::Path::new(&home_dir).join("Downloads");
    let trace_file_path = downloads_dir.join("perfetto_subscriber.perfetto_trace");
    let trace_file_path = trace_file_path
        .to_str()
        .expect("Failed to convert path to string")
        .to_string();

    let _guard = perfetto_guard(8192, &trace_file_path).unwrap();

    let mut handles = FuturesUnordered::new();
    for i in 0..16 {
        let handle = tokio::spawn(async move {
            match i % 4 {
                0 => instrument_fut!("scenario1"; scenario4()).await,
                1 => instrument_fut!("scenario2"; scenario4()).await,
                2 => instrument_fut!("scenario3"; scenario4()).await,
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

#[instrument]
async fn scenario1() {
    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;
}

#[instrument]
async fn scenario2() {
    scenario2_1().await;
}

#[instrument]
async fn scenario2_1() {
    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;
}

#[instrument]
async fn scenario3() {
    scenario3_1().await;
    let _ = sum_to(SUM_TO);
    scenario3_1().await;
    let _ = sum_to(SUM_TO);
}

#[instrument]
async fn scenario3_1() {
    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;

    scenario3_2().await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;
}

#[instrument]
async fn scenario3_2() {
    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;

    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(1)).await;
}

#[instrument]
async fn scenario4() {
    for _ in 0..3 {
        scenario4_1().await;
    }
}

#[instrument]
async fn scenario4_1() {
    for _ in 0..2 {
        scenario4_2().await;
    }
}

#[instrument]
async fn scenario4_2() {
    let _ = sum_to(SUM_TO);
    sleep(Duration::from_millis(500)).await;
}
