use crate::config::{HttpBind, HttpConfig};
use clap::Parser;
use http::Method;

#[derive(Parser, Debug, Default)]
pub struct HealthArgs {}

/// Healthcheck for running rustical instance
/// Currently just pings to see if it's reachable via HTTP
#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn cmd_health(http_config: HttpConfig, _health_args: HealthArgs) -> anyhow::Result<()> {
    let client = reqwest::ClientBuilder::new();
    let client = match http_config.bind {
        HttpBind::Unix { ref path } => client.unix_socket(path.as_path()),
        HttpBind::Tcp(_) => client,
    };

    let client = client.build().unwrap();

    let (host, port) = match http_config.bind {
        HttpBind::Tcp(tcp) => (tcp.host, tcp.port),
        HttpBind::Unix { .. } => ("unix".to_owned(), 80),
    };

    let endpoint = format!("http://{host}:{port}/ping").parse().unwrap();
    let request = reqwest::Request::new(Method::GET, endpoint);

    assert!(client.execute(request).await.unwrap().status().is_success());

    Ok(())
}
