mod hole;

use core::task::{Context, Poll};
use futures::future::FutureExt;
use hyper::client::Client;
use hyper::service::Service;
use hyper::Uri;
use hyper::{body::Body, Request};
use std::future::Future;
use std::pin::Pin;
use tokio::time;

/// Builds a new Client object which **only** handles requests with URI
/// `withlatency://<u64>`. Each request takes at least `<u64>` ms to complete.
/// See [latency_request] for conveniently building such a request.
pub fn new_latency_client() -> Client<TimedConnector, Body> {
    Client::builder().build(TimedConnector::new())
}

/// Given `latency`, returns a request with URI `withlatency://[latency]`.
/// This request will take the client `[latency]` ms to handle.
pub fn latency_request(latency: u64) -> Request<Body> {
    Request::builder()
        .uri(format!("withlatency://{}", latency))
        .body(Default::default())
        .unwrap()
}

#[derive(Clone)]
pub struct TimedConnector {}

impl TimedConnector {
    fn new() -> TimedConnector {
        TimedConnector {}
    }
}

impl Service<Uri> for TimedConnector {
    type Response = hole::Hole;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, u: Uri) -> Self::Future {
        // Attempts to extract latency from URI, and sleeps for that amount
        // of time.
        let latency;
        if let Some("withlatency") = u.scheme_str() {
            let l: u64 = u
                .host()
                .expect("Host value")
                .parse()
                .expect("Expected u64 as host name.");
            latency = l;
        } else {
            panic!("Only accepts 'changelatency' scheme, use [latency_request] method.")
        }

        let f = time::sleep(time::Duration::from_millis(latency));
        Box::pin(f.map(|()| Ok(Self::Response::new())))
    }
}
