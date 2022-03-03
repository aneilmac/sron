use clap::Parser;
use hyper::{body::Body, client::Client, Request};
use sron::{request_at_rate, Elapsed};
use tokio::time::Duration;

#[derive(clap::ArgEnum, Clone, Debug)]
enum OutputFormat {
    Json,
    Raw,
}

impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::Raw
    }
}

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

    /// Output format
    #[clap(short, long, arg_enum, default_value_t=OutputFormat::default())]
    output_format: OutputFormat,

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

    let timeouts = results.iter().filter(|e| e.1.is_timeout()).count();
    if timeouts > 0 {
        eprintln!(
            "{} requests timed out with latency >= {:?}",
            timeouts, args.timeout
        );
    }

    write_with_format(results, args.output_format);
}

fn write_with_format(
    data: impl IntoIterator<Item = (Duration, Elapsed)>,
    output_format: OutputFormat,
) {
    match output_format {
        OutputFormat::Json => write_json(data),
        OutputFormat::Raw => write_raw(data),
    }
}

fn write_json(data: impl IntoIterator<Item = (Duration, Elapsed)>) {
    use serde_json::json;
    println!(
        "{}",
        json!({
            "results": data.into_iter().map(|(s, d)|
                json!(
                    {
                        "start_time_secs": s.as_secs_f64(),
                        "duration_secs": match d {
                            Elapsed::Success(d) => json!(d.as_secs_f64()),
                            Elapsed::Timeout => serde_json::Value::Null,
                        }
                    })
            ).collect::<Vec<_>>()
        })
    );
}

fn write_raw(data: impl IntoIterator<Item = (Duration, Elapsed)>) {
    for (s, d) in data.into_iter() {
        let d = d.into_inner().map(|v| v.as_micros());
        let d = d.map_or(String::from("TIMEOUT"), |d| d.to_string());
        println!("{},{}", s.as_micros(), d);
    }
}
