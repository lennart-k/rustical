use crate::config::{HttpBindConfig, HttpConfig};
use clap::Parser;
use http::Method;

#[derive(Parser, Debug, Default)]
pub struct HealthArgs {}

/// Healthcheck for running rustical instance
/// Currently just pings to see if it's reachable via HTTP
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn cmd_health(http_config: HttpConfig, _health_args: HealthArgs) -> anyhow::Result<()> {
    let bind_config = http_config.bind_config()?;
    let mut client_builder = reqwest::ClientBuilder::new();

    let address = match bind_config {
        HttpBindConfig::Tcp(address) => address,
        HttpBindConfig::Unix(path) => {
            client_builder = client_builder.unix_socket(path);
            "rustical".to_string()
        }
    };
    let client = client_builder.build()?;

    let endpoint = format!("http://{address}/ping").parse().unwrap();
    let request = reqwest::Request::new(Method::GET, endpoint);

    assert!(client.execute(request).await.unwrap().status().is_success());

    Ok(())
}
