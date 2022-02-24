use futures::future::join_all;
use hyper::{body::HttpBody, client::connect::Connect};
use hyper::{Client, Request};
use std::error::Error;
use tokio::time::{sleep_until, Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum Elapsed {
    Success(Duration),
    Timeout,
}

impl Elapsed {
    pub fn new(d: Duration) -> Elapsed {
        Elapsed::Success(d)
    }

    pub fn is_timeout(&self) -> bool {
        match self {
            Elapsed::Success(_) => false,
            Elapsed::Timeout => true,
        }
    }

    pub fn unwrap(self) -> Duration {
        match self {
            Elapsed::Success(d) => d,
            Elapsed::Timeout => panic!("Attempting to unwrap an Elapsed::Timeout."),
        }
    }
}

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
    timeout: Duration,
    client: Client<C, B>,
    reqs: impl IntoIterator<Item = Request<B>>,
) -> impl Iterator<Item = (Instant, Elapsed)>
where
    C: Clone + Send + Sync + Connect + 'static,
    B: Send + HttpBody + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: Into<Box<(dyn Error + Send + Sync)>>,
{
    let start = Instant::now();

    let v = std::iter::successors(Some(Duration::ZERO), |d| {
        d.checked_add(period)
            .and_then(|d| if d < duration { Some(d) } else { None })
    })
    .zip(std::iter::repeat(client))
    .zip(reqs.into_iter())
    .map(|((i, client), req)| {
        async move {
            sleep_until(start + i).await;
            let request_start = Instant::now();
            let res = tokio::time::timeout(timeout, client.request(req)).await;
            if res.is_ok() {
                // Request may or may not have completed successfully, but it did not timeout.
                (request_start, Elapsed::new(request_start.elapsed()))
            } else {
                // Request timed out.
                (request_start, Elapsed::Timeout)
            }
        }
    });
    // Wait for all requests to complete.
    join_all(v).await.into_iter()
}
