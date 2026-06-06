use std::{path::PathBuf, str::FromStr};

use anyhow::anyhow;
use reqwest::Url;
use rustical_caldav::CalDavConfig;
use rustical_frontend::FrontendConfig;
use rustical_oidc::OidcConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct HttpConfig {
    pub bind: Option<String>,
    // host, port are deprecated
    pub host: Option<String>,
    pub port: Option<u16>,
    pub session_cookie_samesite_strict: bool,
    pub payload_limit_mb: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpBindConfig {
    Tcp(String),
    Unix(PathBuf),
}

impl FromStr for HttpBindConfig {
    type Err = anyhow::Error;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        // This is incredibly janky but given the comprehensive tests I think we can justify it
        let Ok(url) = address.parse::<Url>() else {
            return Ok(Self::Tcp(address.to_string()));
        };

        Ok(match url.scheme() {
            "unix" => {
                if url.host_str().is_some() || url.port().is_some() {
                    return Err(anyhow!(
                        "Invalid URL in http.bind config: unix: url cannot contain a host. You probably used double slashes, use unix:///absolute/path or unix:/absolute/path"
                    ));
                }
                Self::Unix(url.path().parse()?)
            }
            "http" => {
                if url.path() != "/" {
                    return Err(anyhow!(
                        "Invalid URL in http.bind config: http URL cannot contain a path"
                    ));
                }
                let Some(host) = url.host_str() else {
                    return Err(anyhow!("Invalid URL in http.bind config: host missing"));
                };
                let Some(port) = url.port() else {
                    return Err(anyhow!(
                        "Error in http.bind config: Please explicitly specify a port"
                    ));
                };
                Self::Tcp(format!("{host}:{port}"))
            }
            scheme => {
                // localhost:1234 will become scheme=localhost, path=1234
                if let Ok(port) = url.path().parse::<u16>()
                    && address == format!("{scheme}:{port}")
                {
                    return Ok(Self::Tcp(address.to_string()));
                }

                return Err(anyhow!(
                    "Error in http.bind config: Invalid schema: {scheme}. If it is a hostname explicitly specify a port such as {scheme}:4000"
                ));
            }
        })
    }
}

#[cfg(test)]
mod bind_config {
    use crate::config::HttpBindConfig;
    use rstest::rstest;
    use std::str::FromStr;

    #[rstest]
    #[case("unix:///run/rustical/socket", HttpBindConfig::Unix("/run/rustical/socket".parse().unwrap()))]
    #[case("http://[::]:4000", HttpBindConfig::Tcp("[::]:4000".to_string()))]
    #[case("[::]:4000", HttpBindConfig::Tcp("[::]:4000".to_string()))]
    #[case("example.com:4000", HttpBindConfig::Tcp("example.com:4000".to_string()))]
    #[case("172.10.10.1:4000", HttpBindConfig::Tcp("172.10.10.1:4000".to_string()))]
    #[case("localhost:1234", HttpBindConfig::Tcp("localhost:1234".to_string()))]
    #[case("http://localhost:1234", HttpBindConfig::Tcp("localhost:1234".to_string()))]
    // Unix relative paths
    #[case(
        "unix:asd/asd",
        HttpBindConfig::Unix("asd/asd".parse().unwrap())
    )]
    #[case(
        "unix:asd",
        HttpBindConfig::Unix("asd".parse().unwrap())
    )]
    // Unix absolute path
    #[case(
        "unix:/asd",
        HttpBindConfig::Unix("/asd".parse().unwrap())
    )]
    #[case(
        "unix:/asd/asd",
        HttpBindConfig::Unix("/asd/asd".parse().unwrap())
    )]
    fn test_parse_http_bind_valid(#[case] address: &str, #[case] out: HttpBindConfig) {
        assert_eq!(HttpBindConfig::from_str(address).unwrap(), out);
    }

    #[rstest]
    #[case("unix://asd/run/rustical/socket")]
    #[case("http://[::]:4000/asdlkj")]
    #[case("http://localhost")]
    #[case("https://localhost:4000")]
    #[case("localhost:1234/asd")]
    #[case("unix://hallo:123/run/rustical/socket")]
    fn test_parse_http_bind_invalid(#[case] address: &str) {
        assert!(HttpBindConfig::from_str(address).is_err());
    }

    #[rstest]
    #[case(
        "unix://:123/run/rustical/socket",
        HttpBindConfig::Tcp("unix://:123/run/rustical/socket".to_string())
    )]
    #[case("localhost", HttpBindConfig::Tcp("localhost".to_string()))]
    fn test_parse_http_bind_invalid_but_will_fail_anyway(
        #[case] address: &str,
        #[case] out: HttpBindConfig,
    ) {
        assert_eq!(HttpBindConfig::from_str(address).unwrap(), out);
    }
}

impl HttpConfig {
    fn address(&self) -> anyhow::Result<String> {
        if let Some(ref host) = self.host {
            let port = self.port.unwrap_or(4000);
            tracing::warn!(
                "Using http.host/port is deprecated and will be removed in the future. Please instead use http.bind"
            );
            return Ok(format!("{host}:{port}"));
        }

        if let Some(port) = self.port {
            let host = self.host.as_deref().unwrap_or("[::]");
            tracing::warn!(
                "Using http.host/port is deprecated and will be removed in the future. Please instead use http.bind"
            );
            return Ok(format!("{host}:{port}"));
        }

        if let Some(ref address) = self.bind {
            return Ok(address.clone());
        }

        Err(anyhow!(
            "http.bind is not configured (this should not happen since it has a default)"
        ))
    }

    pub fn bind_config(&self) -> anyhow::Result<HttpBindConfig> {
        HttpBindConfig::from_str(&self.address()?)
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind: Some("[::]:4000".to_owned()),
            host: None,
            port: None,
            session_cookie_samesite_strict: false,
            payload_limit_mb: 4,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SqliteDataStoreConfig {
    pub db_url: String,
    #[serde(default = "default_true")]
    pub run_repairs: bool,
    #[serde(default = "default_true")]
    pub skip_broken: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum DataStoreConfig {
    Sqlite(SqliteDataStoreConfig),
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct TracingConfig {
    pub opentelemetry: bool,
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct DavPushConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    // Allowed Push servers, accepts any by default
    // Specify as URL origins
    pub allowed_push_servers: Option<Vec<String>>,
}

impl Default for DavPushConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_push_servers: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct NextcloudLoginConfig {
    pub enabled: bool,
}

impl Default for NextcloudLoginConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub data_store: DataStoreConfig,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub frontend: FrontendConfig,
    #[serde(default)]
    pub oidc: Option<OidcConfig>,
    #[serde(default)]
    pub tracing: TracingConfig,
    #[serde(default)]
    pub dav_push: DavPushConfig,
    #[serde(default)]
    pub nextcloud_login: NextcloudLoginConfig,
    #[serde(default)]
    pub caldav: CalDavConfig,
}
