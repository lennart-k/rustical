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
- adequately fast
- deleted calendars are recoverable

## Installation

### Manual

```sh
cargo install --git https://github.com/lennart-k/rustical
```

### Docker

```sh
docker run \
  -p 4000:4000 \
  -v YOUR_DATA_DIR:/var/lib/rustical/ \
  -v YOUR_CONFIG_TOML:/etc/rustical/config.toml \
  ghcr.io/lennart-k/rustical
```

## Configuration

You can generate a default `config.toml` with

```sh
rustical gen-config
```

You'll have to set your database path to something like `/var/lib/rustical/db.sqlite3`.
There you also set your username, password, and app tokens.
Password hashes can be generated with

```sh
rustical pwhash
```

#### Docker

You can also run the upper commands in Docker with

```sh
docker run --rm ghcr.io/lennart-k/rustical rustical gen-config
docker run -it --rm ghcr.io/lennart-k/rustical rustical pwhash
```

### Password vs app tokens

The password is meant as a password you use to log in to the frontend.
Since it's sensitive information, the secure but slow hash algorithm `argon2` is chosen.

I recommend to generate random app tokens for each CalDAV/CardDAV client.
These can use the faster `pbkdf2` algorithm.

### WebDAV Push

RustiCal supports [WebDAV Push](https://github.com/bitfireAT/webdav-push/) which can notify compatible clients like DAVx5 about changed calendar/addressbook objects.
Since push messages are currently not encrypted you might potentially want to ensure that users only subscribe through your push server (e.g. [ntfy.sh](https://ntfy.sh/)), you can configure it the following:

```toml
[dav_push]
# Must strictly be the URL origin (so no trailing slashes)
allowed_push_servers = ["https://your-instance-ntfy.sh"]
```

## Relevant RFCs

- Versioning Extensions to WebDAV: [RFC 3253](https://datatracker.ietf.org/doc/html/rfc3253)
  - provides the REPORT method
- Calendaring Extensions to WebDAV (CalDAV): [RFC 4791](https://datatracker.ietf.org/doc/html/rfc4791)
- Scheduling Extensions to CalDAV: [RFC 6638](https://datatracker.ietf.org/doc/html/rfc6638)
  - not sure yet whether to implement this
- Collection Synchronization WebDAV [RFC 6578](https://datatracker.ietf.org/doc/html/rfc6578)
  - We need to implement sync-token, etc.
  - This is important for more efficient synchronisation
- iCalendar [RFC 2445](https://datatracker.ietf.org/doc/html/rfc2445#section-3.10)
