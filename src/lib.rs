use hyper::{body::HttpBody, client::connect::Connect};
use hyper::{Client, Request};
use std::error::Error;
use tokio::time::{Duration, Instant};
use std::collections::LinkedList;

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
/// The resulting output is a vector of tuples of form `(i, d)`, where `i` is the duration
/// since starting the load testing, and `d` is the duration that request took.
///
/// This method does not suffer from coordinates omission. A high-latency request will not
/// prevent other requests from being sampled.
pub async fn request_at_rate<C, B>(
    period: Duration,
    duration: Duration,
    timeout: Duration,
    client: Client<C, B>,
    reqs: impl IntoIterator<Item = Request<B>>,
) -> Vec<(Duration, Elapsed)>
where
    C: Clone + Send + Sync + Connect + 'static,
    B: Send + HttpBody + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: Into<Box<(dyn Error + Send + Sync)>>,
{   
    let mut i = Duration::ZERO;
    let mut interval = tokio::time::interval(period);
    let mut ll = LinkedList::<_>::new();
    let process_start = Instant::now();
    for req in reqs {
        if i >= duration {
            break;
        }
        interval.tick().await;
        let client = client.clone();
        let request_start = Instant::now();
        ll.push_back(tokio::spawn(async move {
            let res = tokio::time::timeout(timeout, client.request(req)).await;
            let end_time = request_start.elapsed();
            let start_time = request_start.duration_since(process_start);
            if res.is_ok() {
                // Request may or may not have completed successfully, but it did not timeout.
                (start_time, Elapsed::new(end_time))
            } else {
                // Request timed out.
                (start_time, Elapsed::Timeout)
            }
        }));
        i += period;
    }
    futures::future::try_join_all(ll).await.expect("No JoinError")
}
