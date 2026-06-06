use crate::config::{HttpBindConfig, HttpConfig};
use clap::Parser;
use http::Method;

#[derive(Parser, Debug, Default)]
pub struct HealthArgs {}

/// Healthcheck for running rustical instance
/// Currently just pings to see if it's reachable via HTTP
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn cmd_health(http_config: HttpConfig, _health_args: HealthArgs) -> anyhow::Result<()> {
    let client = reqwest::ClientBuilder::new().build().unwrap();

    let HttpBindConfig::Tcp(address) = http_config.bind_config()? else {
        todo!("Listening on UNIX sockets not implemented yet");
    };
    let endpoint = format!("http://{address}/ping").parse().unwrap();
    let request = reqwest::Request::new(Method::GET, endpoint);

    assert!(client.execute(request).await.unwrap().status().is_success());

    Ok(())
}
