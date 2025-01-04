# RustiCal

a CalDAV/CardDAV server

> [!CAUTION]
> RustiCal is **not production-ready!**
> There can be changes to the database without migrations and there's no guarantee that all endpoints are secured yet.
> If you still want to play around with it in its current state, absolutely feel free to do so but know that not even I use it productively yet.

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
  -v YOUR_DATA_DIRECTORY:YOUR_DATA_DIRECTORY \
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

### Password vs app tokens

The password is meant as a password you use to log in to the frontend.
Since it's sensitive information, the secure but slow hash algorithm `argon2` is chosen.

I recommend to generate random app tokens for each CalDAV/CardDAV client.
These can use the faster `pbkdf2` algorithm.

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
