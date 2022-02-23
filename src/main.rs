use clap::Parser;
use hyper::{body::Body, client::Client, Request};
use sron::{Elapsed, request_at_rate};
use tokio::time::Duration;

/// Fixed-rate latency tester.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Fire a new request every period (ms).
    #[clap(short, long, default_value_t = 33)]
    period: u64,

    /// Total duration to run the tests (ms).
    #[clap(short, long, default_value_t = 300000)]
    duration: u64,

    /// Maximum latency before a request is aborted (ms).
    #[clap(short, long, default_value_t = 60000)]
    timeout: u64,

    #[clap(last = true)]
    urls: Vec<hyper::Uri>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    let client: Client<_, Body> = Client::builder().build_http();

    // let mut file = File::open(args.urls).expect("Existing file.");
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)
    //     .expect("Valid file contents.");
    let urls = args.urls.into_iter().map(|u| {
        Request::builder()
            .uri(u)
            .body(Default::default())
            .expect("Valid request")
    });

    let results = request_at_rate(
        Duration::from_millis(args.period),
        Duration::from_millis(args.duration),
        Duration::from_millis(args.timeout),
        client,
        urls.cycle(),
    )
    .await;

    let results = results.expect("Not all requests successfully completed.");

    let timeouts = results.iter().filter(|e| e.1.is_timeout()).count();
    if timeouts > 0 {
        eprintln!("{} requests timed out with latency >= {}ms", timeouts, args.timeout);
    }
    for (_, d) in results {
        if let Elapsed::Success(d) = d {
            println!("{}", d.as_micros());
        }
    }
}
