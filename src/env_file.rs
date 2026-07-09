use figment::value::{Dict, Map, Tag, Value};
use figment::{Error, Metadata, Profile, Provider};

/// A [`figment::Provider`] that loads configuration values from files referenced by
/// `*_FILE`-suffixed environment variables (e.g. Docker secrets).
///
/// For example, with `RUSTICAL_OIDC__CLIENT_SECRET_FILE=/run/secrets/oidc_client_secret`
/// the config value `oidc.client_secret` is set to the contents of that file.
/// The file content is used verbatim as a string, only trailing newlines are stripped.
pub struct EnvFile {
    prefix: &'static str,
    split: Option<&'static str>,
}

impl EnvFile {
    #[must_use]
    pub const fn prefixed(prefix: &'static str) -> Self {
        Self {
            prefix,
            split: None,
        }
    }

    #[must_use]
    pub const fn split(mut self, split: &'static str) -> Self {
        self.split = Some(split);
        self
    }
}

fn strip_prefix_ci<'a>(string: &'a str, prefix: &str) -> Option<&'a str> {
    string
        .get(..prefix.len())
        .filter(|start| start.eq_ignore_ascii_case(prefix))
        .map(|_| &string[prefix.len()..])
}

fn strip_suffix_ci<'a>(string: &'a str, suffix: &str) -> Option<&'a str> {
    string
        .len()
        .checked_sub(suffix.len())
        .filter(|&mid| string.is_char_boundary(mid) && string[mid..].eq_ignore_ascii_case(suffix))
        .map(|mid| &string[..mid])
}

fn insert_nested(dict: &mut Dict, keys: &[String], value: Value) {
    let (key, rest) = keys.split_first().expect("keys must not be empty");
    if rest.is_empty() {
        dict.insert(key.clone(), value);
        return;
    }
    let entry = dict
        .entry(key.clone())
        .or_insert_with(|| Value::Dict(Tag::Default, Dict::new()));
    if !matches!(entry, Value::Dict(..)) {
        *entry = Value::Dict(Tag::Default, Dict::new());
    }
    let Value::Dict(_, inner) = entry else {
        unreachable!()
    };
    insert_nested(inner, rest, value);
}

impl Provider for EnvFile {
    fn metadata(&self) -> Metadata {
        Metadata::named(format!(
            "`{prefix}*_FILE` environment variable(s)",
            prefix = self.prefix
        ))
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut dict = Dict::new();
        for (name, path) in std::env::vars() {
            let Some(key) = strip_prefix_ci(&name, self.prefix) else {
                continue;
            };
            let Some(key) = strip_suffix_ci(key, "_FILE") else {
                continue;
            };
            if key.is_empty() {
                continue;
            }
            let contents = std::fs::read_to_string(&path)
                .map_err(|err| Error::from(format!("Cannot read file {path} ({name}): {err}")))?;
            let value = contents.trim_end_matches(['\r', '\n']).to_string();
            let keys: Vec<String> = match self.split {
                Some(split) => key.split(split).map(str::to_lowercase).collect(),
                None => vec![key.to_lowercase()],
            };
            insert_nested(&mut dict, &keys, Value::from(value));
        }
        Ok(Map::from([(Profile::Default, dict)]))
    }
}

#[cfg(test)]
mod tests {
    use super::EnvFile;
    use crate::config::Config;
    use figment::{
        Figment, Jail,
        providers::{Env, Format, Toml},
    };

    fn figment(jail: &Jail) -> Figment {
        let _ = jail;
        Figment::new()
            .merge(
                Env::prefixed("RUSTICAL_")
                    .filter(|key| !key.as_str().to_ascii_lowercase().ends_with("_file"))
                    .split("__"),
            )
            .merge(EnvFile::prefixed("RUSTICAL_").split("__"))
    }

    #[test]
    fn test_config_env_file_secret() {
        Jail::expect_with(|jail| {
            jail.create_file("client_secret", "topsecret\n")?;
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env("RUSTICAL_OIDC__NAME", "Authelia");
            jail.set_env("RUSTICAL_OIDC__ISSUER", "https://auth.rustical.dev");
            jail.set_env("RUSTICAL_OIDC__CLIENT_ID", "rustical");
            jail.set_env("RUSTICAL_OIDC__SCOPES", "[openid]");
            jail.set_env(
                "RUSTICAL_OIDC__CLIENT_SECRET_FILE",
                jail.directory().join("client_secret").to_str().unwrap(),
            );

            let config: Config = figment(jail).extract()?;
            assert_eq!(
                config.oidc.unwrap().client_secret.unwrap().secret(),
                "topsecret"
            );
            Ok(())
        });
    }

    #[test]
    fn test_config_env_file_takes_precedence_over_env() {
        Jail::expect_with(|jail| {
            jail.create_file("client_secret", "from_file")?;
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env("RUSTICAL_OIDC__NAME", "Authelia");
            jail.set_env("RUSTICAL_OIDC__ISSUER", "https://auth.rustical.dev");
            jail.set_env("RUSTICAL_OIDC__CLIENT_ID", "rustical");
            jail.set_env("RUSTICAL_OIDC__SCOPES", "[openid]");
            jail.set_env("RUSTICAL_OIDC__CLIENT_SECRET", "from_env");
            jail.set_env(
                "RUSTICAL_OIDC__CLIENT_SECRET_FILE",
                jail.directory().join("client_secret").to_str().unwrap(),
            );

            let config: Config = figment(jail).extract()?;
            assert_eq!(
                config.oidc.unwrap().client_secret.unwrap().secret(),
                "from_file"
            );
            Ok(())
        });
    }

    #[test]
    fn test_config_env_file_takes_precedence_over_toml() {
        Jail::expect_with(|jail| {
            jail.create_file("client_secret", "from_file")?;
            jail.create_file(
                "config.toml",
                r#"
[data_store.sqlite]
db_url = "/var/lib/rustical/db.sqlite3"

[oidc]
name = "Authelia"
issuer = "https://auth.rustical.dev"
client_id = "rustical"
client_secret = "from_toml"
scopes = ["openid"]
"#,
            )?;
            jail.set_env(
                "RUSTICAL_OIDC__CLIENT_SECRET_FILE",
                jail.directory().join("client_secret").to_str().unwrap(),
            );

            let config: Config = Figment::new()
                .merge(Toml::file(jail.directory().join("config.toml")))
                .merge(figment(jail))
                .extract()?;
            assert_eq!(
                config.oidc.unwrap().client_secret.unwrap().secret(),
                "from_file"
            );
            Ok(())
        });
    }

    #[test]
    fn test_config_env_file_missing_file() {
        Jail::expect_with(|jail| {
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env("RUSTICAL_OIDC__CLIENT_SECRET_FILE", "/does/not/exist");

            assert!(figment(jail).extract::<Config>().is_err());
            Ok(())
        });
    }
}
