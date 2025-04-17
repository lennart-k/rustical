# Installation

## Manual

```sh
cargo install --locked --git https://github.com/lennart-k/rustical
```

## Docker

```sh
docker run \
  -p 4000:4000 \
  -v YOUR_DATA_DIR:/var/lib/rustical/ \
  -v YOUR_PRINCIPALS_TOML:/etc/rustical/principals.toml \
  -v YOUR_CONFIG_TOML:/etc/rustical/config.toml \ # (1)!
  -e RUSTICAL__CONFIG_OPTION="asd" \  # (2)!
  ghcr.io/lennart-k/rustical
```

1. Mount config file
2. Alternatively specify configuration using environment variables

## Configuration

RustiCal can be configured using either a `config.toml` file or environent variables.

To see all configuration options available you can browse the [Cargo docs](/rustical/_crate/rustical/config/struct.Config.html).

### TOML

You can generate a default `config.toml` configuration using

```sh title="Generate default config.toml"
rustical gen-config
```

### Environment variables

The options in `config.toml` can also be configured using environment variables.
Names translate the following:

```toml title="Example config.toml"
[data_store.toml]
path = "asd"
```

becomes `RUSTICAL_DATA_STORE__TOML__PATH`.
Every variable is

- uppercase
- prefixed by `RUSTICAL_`
- Dots become `__`
- Arrays are JSON-encoded
