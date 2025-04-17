# RustiCal

a CalDAV/CardDAV server

> [!CAUTION]
> RustiCal is **not production-ready!**
> There can be changes to the database without migrations and there's no guarantee that all endpoints are secured yet.
> If you still want to play around with it in its current state, absolutely feel free to do so but know that not even I use it productively yet.

## Features

- easy to backup, everything saved in one SQLite database
- [WebDAV Push](https://github.com/bitfireAT/webdav-push/) support, so near-instant synchronisation to DAVx5
- lightweight (the container image contains only one binary)
- adequately fast (I'd say blazingly fast™ :fire: if I did the benchmarks to back that claim up)
- deleted calendars are recoverable
- Nextcloud login flow (In DAVx5 you can login through the Nextcloud flow and automatically generate an app token)
- OpenID Connect support (with option to disable password login)

## Getting Started

- Check out the [documentation](https://github.com)

## Installation

### Manual

```sh
cargo install --locked --git https://github.com/lennart-k/rustical
```

### Docker

```sh
docker run \
  -p 4000:4000 \
  -v YOUR_DATA_DIR:/var/lib/rustical/ \
  -v YOUR_CONFIG_TOML:/etc/rustical/config.toml \
  -v YOUR_PRINCIPALS_TOML:/etc/rustical/principals.toml \
  ghcr.io/lennart-k/rustical
```

## Configuration

RustiCal can either be configured using a TOML file or using environment variables.

You can generate a default `config.toml` with

```sh
rustical gen-config
```

> [!WARNING]
> The `rustical gen-config` command generates a random `frontend.secret_key`.
> This secret is used to generate session cookies so if it is leaked an attacker could use it to authenticate to against any endpoint (also when the frontend is disabled).

You'll have to set your database path to something like `/var/lib/rustical/db.sqlite3`.

### Environment variables

The options in `config.toml` can also be configured using environment variables. Names translate the following:

```toml
[data_store.toml]
path = "asd"
```

becomes `RUSTICAL_DATA_STORE__TOML__PATH`.
Every variable is

- uppercase
- prefixed by `RUSTICAL_`
- Dots become `__`

### Users and groups

Next, configure the principals by creating a file specified in `auth.toml.path` (by default `/etc/rustical/principals.toml`) and inserting your principals:

```toml
[[principals]]
id = "user"
displayname = "User"
password = "$argon2id$......."
app_tokens = [
  {id = "1", name = "Token", token = "$pbkdf2-sha256$........"},
]
memberships = ["group:amazing_group"]

[[principals]]
id = "group:amazing_group"
user_type = "group"
displayname = "Amazing group"
```

Password hashes can be generated with

```sh
rustical pwhash
```

### Docker

You can also run the upper commands in Docker with

```sh
docker run --rm ghcr.io/lennart-k/rustical rustical gen-config
docker run -it --rm ghcr.io/lennart-k/rustical rustical pwhash
```

### Password vs app tokens

The password is optional (if you have configured OpenID Connect) and is only used to log in to the frontend.
Since it's sensitive information,
the secure but slow hash algorithm `argon2` is chosen.

App tokens are used by your CalDAV/CardDAV client (which can be managed through the frontend).
I recommend to generate random app tokens for each CalDAV/CardDAV client.
Since the app tokens are random they use the faster `pbkdf2` algorithm.

### WebDAV Push

RustiCal supports [WebDAV Push](https://github.com/bitfireAT/webdav-push/) which can notify compatible clients like DAVx5 about changed calendar/addressbook objects.
Since push messages are currently not encrypted you might potentially want to ensure that users only subscribe through your push server (e.g. [ntfy.sh](https://ntfy.sh/)), you can configure it the following:

```toml
[dav_push]
# Must strictly be the URL origin (so no trailing slashes)
allowed_push_servers = ["https://your-instance-ntfy.sh"]
```
