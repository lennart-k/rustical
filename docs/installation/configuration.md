# Configuration

While RustiCal (apart from user management) will work without any configuration you should still know how to configure it. :)

You can either mount a `config.toml` file or use environment variables (recommended).

To see the options you can generate a default configuration using

```sh title="Generate default config.toml"
rustical gen-config
rustical gen-config --postgres
```

To see all configuration options available you can browse the [Cargo docs](/rustical/_crate/rustical/config/struct.Config.html).

## Environment variables

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

## PostgreSQL

```toml
[data_store.postgres]
db_url = "postgres://user@localhost/rustical"
run_repairs = true
skip_broken = true
```

`db_url` is a standard PostgreSQL URL. Environment equivalent: `RUSTICAL_DATA_STORE__POSTGRES__DB_URL`.
