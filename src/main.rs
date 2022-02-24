use clap::Parser;
use hyper::{body::Body, client::Client, Request};
use sron::{request_at_rate, Elapsed};
use tokio::time::Duration;

/// Fixed-rate latency tester.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Fire a new request every period.
    #[clap(short, long, parse(try_from_str=duration_str::parse), default_value="30ms")]
    period: Duration,

    /// Total duration to run the tests.
    #[clap(short, long, parse(try_from_str=duration_str::parse), default_value="5m")]
    duration: Duration,

    /// Maximum latency before a request is aborted.
    #[clap(short, long, parse(try_from_str=duration_str::parse), default_value="60s")]
    timeout: Duration,

    /// List of 1 or more Urls to cycle through for each
    #[clap(last = true)]
    urls: Vec<hyper::Uri>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    let client: Client<_, Body> = Client::builder().build_http();

    let urls = args.urls.into_iter().map(|u| {
        Request::builder()
            .uri(u)
            .body(Default::default())
            .expect("Valid request")
    });

    let results = request_at_rate(
        args.period,
        args.duration,
        args.timeout,
        client,
        urls.cycle(),
    )
    .await;

    let results = results.collect::<Vec<_>>();
    let timeouts = results.iter().filter(|e| e.1.is_timeout()).count();
    if timeouts > 0 {
        eprintln!(
            "{} requests timed out with latency >= {:?}",
            timeouts, args.timeout
        );
    }
    for (_, d) in results {
        if let Elapsed::Success(d) = d {
            println!("{}", d.as_micros());
        }
    }
}
