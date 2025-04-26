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

You'll have to set your database path to something like `/var/lib/rustical/db.sqlite3`.

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

## Users and groups

Next, you will want to set up your principals.
Using the `rustical principals` command you can manage principals and passwords.

Groups and rooms are also just principals and you can specify them as such using the `--principal-type` parameter.
To assign a user to a group you can use the `rustical membership` command. Being a member to a principal means that you can completely act on their behalf and see their collections.

You can also completely skip this and instead configure OpenID Connect.
In that case your user will automatically be created when logging in through the frontend.

## Password vs app tokens

The password is optional (if you have configured OpenID Connect) and is only used to log in to the frontend.
Since it's sensitive information,
the secure but slow hash algorithm `argon2` is chosen.

App tokens are used by your CalDAV/CardDAV client (which can be managed through the frontend).
I recommend to generate random app tokens for each CalDAV/CardDAV client.
Since the app tokens are random they use the faster `pbkdf2` algorithm.

## WebDAV Push

RustiCal supports [WebDAV Push](https://github.com/bitfireAT/webdav-push/) which can notify compatible clients like DAVx5 about changed calendar/addressbook objects.
Since push messages are currently not encrypted you might potentially want to ensure that users only subscribe through your push server (e.g. [ntfy.sh](https://ntfy.sh/)), you can configure it the following:

```toml
[dav_push]
# Must strictly be the URL origin (so no trailing slashes)
allowed_push_servers = ["https://your-instance-ntfy.sh"]
```
