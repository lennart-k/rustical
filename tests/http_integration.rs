// This integration test checks whether the HTTP server works by actually running rustical in a new
// thread.
use common::rustical_process;
use http::StatusCode;
use std::time::Duration;

mod common;

pub async fn test_runner<O, F>(inner: F)
where
    O: IntoFuture<Output = ()>,
    // <O as IntoFuture>::IntoFuture: UnwindSafe,
    F: FnOnce(String) -> O,
{
    // Start RustiCal process
    let (token, port, main_process, start_notify) = rustical_process();
    let origin = format!("http://localhost:{port}");

    // Wait for RustiCal server to listen
    tokio::time::timeout(Duration::new(2, 0), start_notify.notified())
        .await
        .unwrap();

    // We use catch_unwind to make sure we'll always correctly stop RustiCal
    // Otherwise, our process would just run indefinitely
    inner(origin).into_future().await;

    // Signal RustiCal to stop
    token.cancel();
    main_process.join().unwrap();
}

#[tokio::test]
async fn test_ping() {
    test_runner(async |origin| {
        let resp = reqwest::get(origin.clone() + "/ping").await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Ensure that path normalisation works as intended
        let resp = reqwest::get(origin + "/ping/").await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    })
    .await
}
