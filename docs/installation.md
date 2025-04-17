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
  -v YOUR_CONFIG_TOML:/etc/rustical/config.toml \
  -v YOUR_PRINCIPALS_TOML:/etc/rustical/principals.toml \
  ghcr.io/lennart-k/rustical
```

## Configuration

### TOML

### Environment variables

The options in `config.toml` can also be configured using environment variables. Names translate the following:

```toml title="config.toml"
[data_store.toml]
path = "asd"
```

becomes `RUSTICAL_DATA_STORE__TOML__PATH`.
Every variable is

- uppercase
- prefixed by `RUSTICAL_`
- Dots become `__`
- Arrays are JSON-encoded
