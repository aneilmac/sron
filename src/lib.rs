use futures::future::join_all;
use hyper::{body::HttpBody, client::connect::Connect};
use hyper::{Client, Request};
use std::error::Error;
use tokio::task::JoinError;
use tokio::time::{sleep_until, Duration, Instant};

/// Over a period of `duration,` sends a request every `period` to `client` from the
/// list of requests in `reqs` until either `reqs` is exhausted or we have run for (at least)
/// `duration` amount of time.
///
/// The resulting output is a vector of tuples of form `(i, d)`, where `i` is the instant
/// a request was sent, and `d` is the duration that request took.
///
/// This method does not suffer from coordinates omission. A high-latency request will not
/// prevent other requests from being sampled.
pub async fn request_at_rate<C, B>(
    period: Duration,
    duration: Duration,
    client: Client<C, B>,
    reqs: impl IntoIterator<Item = Request<B>>,
) -> Result<Vec<(Instant, Duration)>, JoinError>
where
    C: Clone + Send + Sync + Connect + 'static,
    B: Send + HttpBody + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: Into<Box<(dyn Error + Send + Sync)>>,
{
    let start = Instant::now();

    let mut v = Vec::<_>::new();
    let mut i = Duration::ZERO;
    for req in reqs {
        // Don't run for longer than duration.
        if i >= duration {
            break;
        }

        let client = client.clone();
        // For each request, in-order, send a request at the given
        // time-point.
        v.push(tokio::spawn(async move {
            sleep_until(start + i).await;
            let request_start = Instant::now();
            let _ = client.request(req).await;
            (request_start, request_start.elapsed())
        }));
        i += period;
    }

    // Wait for all requests to complete.
    join_all(v).await.into_iter().collect()
}
