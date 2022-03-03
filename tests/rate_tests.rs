mod client_mocks;
use sron::request_at_rate;
use tokio::time::Duration;

#[tokio::test]
async fn durations() {
    let client = client_mocks::new_latency_client();

    let request_times = request_at_rate(
        Duration::from_millis(2),
        Duration::from_millis(10),
        Duration::MAX,
        client,
        std::iter::repeat_with(|| client_mocks::latency_request(10)),
    )
    .await;

    assert_eq!(request_times.len(), 5);
    assert!(request_times
        .into_iter()
        .all(|(_, d)| d.unwrap() > Duration::from_millis(10)));
}

#[tokio::test]
async fn instants_ordered() {
    let client = client_mocks::new_latency_client();

    let period = Duration::from_millis(2);

    let times = request_at_rate(
        period,
        Duration::from_millis(10),
        Duration::MAX,
        client,
        std::iter::repeat_with(|| client_mocks::latency_request(1)),
    )
    .await;
    assert_eq!(times.len(), 5);

    let mut q = times.iter();
    let _ = q.next();
    let ts = times.iter().zip(q);
    for (a, b) in ts {
        let a = a.0;
        let b = b.0;
        assert!(a < b, "{:?} >= {:?}", b, a);
    }
}

#[tokio::test]
async fn instants_in_period_low_lat() {
    let client = client_mocks::new_latency_client();

    let period = Duration::from_millis(2);
    let reqs = std::iter::repeat_with(|| client_mocks::latency_request(0)).take(5);

    let times = request_at_rate(
        period,
        Duration::from_millis(10),
        Duration::MAX,
        client,
        reqs,
    )
    .await;
    assert_eq!(times.len(), 5);

    let times = times.into_iter().map(|(t, _)| t);

    let ranges = std::iter::successors(Some((Duration::ZERO, period)), |(a, b)| {
        a.checked_add(period)
            .and_then(|a| b.checked_add(period).map(|b| (a, b)))
    });
    for (i, ((lower, upper), t)) in ranges.zip(times).enumerate() {
        println!("{}: {:?} <= {:?} < {:?}", i, lower, t, upper);
        //assert!(lower <= t, "lower bounds check failed with {:?} <= {:?}, idx: {}", lower, t, i);
        //assert!(t < upper, "upper bounds check failed with {:?} < {:?}, idx: {}", t, upper, i);
    }
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
        Duration::MAX,
        client,
        requests,
    )
    .await;

    assert_eq!(request_times.len(), 10);
    assert!(request_times[0].1.unwrap() >= Duration::from_millis(10));
    assert!(request_times[1].1.unwrap() >= Duration::from_millis(10));
    assert!(request_times[2].1.unwrap() >= Duration::from_millis(10));
    assert!(request_times[3].1.unwrap() >= Duration::from_millis(10));
    assert!(request_times[4].1.unwrap() >= Duration::from_millis(10));
    assert!(request_times[5].1.unwrap() >= Duration::from_millis(100));
    assert!(request_times[6].1.unwrap() >= Duration::from_millis(100));
    assert!(request_times[7].1.unwrap() >= Duration::from_millis(100));
    assert!(request_times[8].1.unwrap() >= Duration::from_millis(100));
    assert!(request_times[9].1.unwrap() >= Duration::from_millis(100));
}

#[tokio::test]
async fn timeout() {
    let client = client_mocks::new_latency_client();

    let requests = std::iter::repeat_with(|| client_mocks::latency_request(100));

    let request_times = request_at_rate(
        Duration::from_millis(1),
        Duration::from_millis(5),
        Duration::from_millis(10),
        client,
        requests,
    )
    .await;

    assert_eq!(request_times.len(), 5);
    assert!(request_times[0].1.is_timeout());
    assert!(request_times[1].1.is_timeout());
    assert!(request_times[2].1.is_timeout());
    assert!(request_times[3].1.is_timeout());
    assert!(request_times[4].1.is_timeout());
}
