#[path = "../tests/client_mocks/mod.rs"]
mod client_mocks;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use hyper::Request;
use tokio::runtime::Runtime;
use tokio::time::Duration;

async fn durations(client: client_mocks::LatencyClient, reqs: Vec<Request<hyper::Body>>) {
    let _ = sron::request_at_rate(
        Duration::from_millis(1),
        Duration::from_millis(5),
        Duration::MAX,
        client,
        reqs,
    )
    .await;
}

fn criterion_benchmark(c: &mut Criterion) {
    let client = client_mocks::new_latency_client();
    c.bench_function("durations", |b| {
        b.to_async(Runtime::new().unwrap()).iter_batched(
            || {
                (
                    client.clone(),
                    std::iter::repeat_with(|| client_mocks::latency_request(0))
                        .take(5)
                        .collect::<Vec<_>>(),
                )
            },
            |(client, its)| durations(client, its),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
