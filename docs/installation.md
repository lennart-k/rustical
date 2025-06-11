# Installation

## Docker

You can start a RustiCal container using the following command:

```sh
docker run \
  -p 4000:4000 \
  -v YOUR_DATA_DIR:/var/lib/rustical/ \
  -v OPTIONAL_YOUR_CONFIG_TOML:/etc/rustical/config.toml \ # (1)!
  -e RUSTICAL__CONFIG_OPTION="asd" \  # (2)!
  ghcr.io/lennart-k/rustical
```

1. Mount config file
2. Alternatively specify configuration using environment variables

## User management

In case you already have an OIDC server set up, see [here](setup/oidc.md) how to set up OIDC login and maybe skip this section.
Otherwise you will have to use the `rustical principals` command.
In Docker you can run this with

```sh
docker run --rm -it -v YOUR_DATA_DIR:/var/lib/rustical/ ghcr.io/lennart-k/rustical rustical principals
```

This is also the place to set up **groups**.
Groups and rooms are also just principals and you can specify them as such using the `--principal-type` parameter.
To assign a user to a group you can use the `rustical membership` command. Being a member to a principal means that you can completely act on their behalf and see their collections.

**Note:** Apple Calendar doesn't play well with the current membership implementation so you might not want to set up groups at the moment.

## Password vs app tokens

The password is optional (if you have configured OpenID Connect) and is only used to log in to the frontend.
Since it's sensitive information, a secure but slow hash algorithm (`argon2`) is chosen.

App tokens are used by your CalDAV/CardDAV client (which can be managed through the frontend).
I recommend to generate random app tokens for each CalDAV/CardDAV client.
Since the app tokens are random they use the faster `pbkdf2` algorithm.

## Configuration

While RustiCal (apart from user management) will work without any configuration you should still know how to configure it. :)

You can either mount a `config.toml` file or use environment variables.

To see the options you can generate a default configuration using

```sh title="Generate default config.toml"
rustical gen-config
```

To see all configuration options available you can browse the [Cargo docs](/rustical/_crate/rustical/config/struct.Config.html).

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

## Manual

```sh
cargo install --locked --git https://github.com/lennart-k/rustical
```
