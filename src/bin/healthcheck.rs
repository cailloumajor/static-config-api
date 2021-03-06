use anyhow::anyhow;
use clap::Parser;
use mime::Mime;
use trillium_async_std::TcpConnector;
use trillium_client::Conn;

use static_config_api::rfc7807::ProblemDetails;

type ClientConn = Conn<'static, TcpConnector>;

#[derive(Parser)]
#[clap(name = "healthcheck")]
#[clap(about = "Static config API health checking")]
struct Args {
    /// Network port to use
    #[clap(action)]
    port: u16,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let url = format!("http://127.0.0.1:{}/health", args.port);
    let mut resp = ClientConn::get(url.as_str()).execute().await?;
    let status = resp.status().unwrap();
    let mut problem: Option<ProblemDetails> = None;
    if let Some(true) = resp
        .response_headers()
        .get_str("content-type")
        .and_then(|t| t.parse::<Mime>().ok())
        .map(|m| m.essence_str() == "application/problem+json")
    {
        problem = resp.response_json().await?;
    };
    status.is_success().then_some(()).ok_or_else(|| {
        if let Some(p) = problem {
            anyhow!("status={} err={}", p.status(), p.title())
        } else {
            anyhow!("status={status} err=unexpected")
        }
    })
}
