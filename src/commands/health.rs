use crate::config::HttpConfig;
use clap::Parser;
use http::Method;

#[derive(Parser, Debug)]
pub struct HealthArgs {}

/// Healthcheck for running rustical instance
/// Currently just pings to see if it's reachable via HTTP
pub async fn cmd_health(http_config: HttpConfig, _health_args: HealthArgs) -> anyhow::Result<()> {
    let client = reqwest::ClientBuilder::new().build().unwrap();

    let endpoint = format!(
        "http://{host}:{port}/ping",
        host = http_config.host,
        port = http_config.port
    )
    .parse()
    .unwrap();
    let request = reqwest::Request::new(Method::GET, endpoint);

    assert!(client.execute(request).await.unwrap().status().is_success());

    Ok(())
}
