mod client_mocks;

use sron::request_at_rate;
use tokio::time::Duration;

#[tokio::test]
async fn durations() {
    let client = client_mocks::new_latency_client();

    let request_times = request_at_rate(
        Duration::from_millis(2),
        Duration::from_millis(10),
        client,
        std::iter::repeat_with(|| client_mocks::latency_request(10)),
    )
    .await
    .expect("Successful client runs");

    assert_eq!(request_times.len(), 5);
    assert!(request_times
        .iter()
        .all(|(_, d)| *d > Duration::from_millis(10)));
}

#[tokio::test]
async fn coordinated_omission_durations() {
    let client = client_mocks::new_latency_client();

    let req_10ms = std::iter::repeat_with(|| client_mocks::latency_request(10));
    let req_100ms = std::iter::repeat_with(|| client_mocks::latency_request(100));
    let requests = req_10ms.take(5).chain(req_100ms);

    let request_times = request_at_rate(
        Duration::from_millis(2),
        Duration::from_millis(20),
        client,
        requests,
    )
    .await
    .expect("Successful client runs");

    assert_eq!(request_times.len(), 10);
    assert!(request_times[0].1 >= Duration::from_millis(10));
    assert!(request_times[1].1 >= Duration::from_millis(10));
    assert!(request_times[2].1 >= Duration::from_millis(10));
    assert!(request_times[3].1 >= Duration::from_millis(10));
    assert!(request_times[4].1 >= Duration::from_millis(10));
    assert!(request_times[5].1 >= Duration::from_millis(100));
    assert!(request_times[6].1 >= Duration::from_millis(100));
    assert!(request_times[7].1 >= Duration::from_millis(100));
    assert!(request_times[8].1 >= Duration::from_millis(100));
    assert!(request_times[9].1 >= Duration::from_millis(100));
}
