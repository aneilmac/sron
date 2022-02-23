use clap::Parser;
use hyper::{body::Body, client::Client, Request};
use sron::request_at_rate;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tokio::time::Duration;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, default_value_t = 33)]
    period: u64,

    #[clap(short, long, default_value_t = 300000)]
    duration: u64,

    #[clap(short, long)]
    urls: PathBuf,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    let client: Client<_, Body> = Client::builder().build_http();

    let mut file = File::open(args.urls).expect("Existing file.");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Valid file contents.");
    let urls = contents.split('\n').map(|u| {
        Request::builder()
            .uri(u)
            .body(Default::default())
            .expect("Valid request")
    });

    let results = request_at_rate(
        Duration::from_millis(args.period),
        Duration::from_millis(args.duration),
        client,
        urls.cycle(),
    )
    .await;

    let results = results.expect("Not all requests successfully completed.");

    for (_, d) in results {
        println!("{}", d.as_micros());
    }
}
