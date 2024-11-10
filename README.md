# RustiCal

a CalDAV/CardDAV server

## Warning: State of the project

This software is not stable yet and things are subject to change.

## Installation

### Manual

```sh
cargo install --git https://github.com/lennart-k/rustical
```

### Docker

```sh
docker run -p 4000:4000 -v YOUR_CONFIG_TOML:/etc/rustical/config.toml -v YOUR_DATA_DIRECTORY:YOUR_DATA_DIRECTORY ghcr.io/lennart-k/rustical
```

## Configuration

You can generate a default `config.toml` with

```sh
rustical gen-config
```

There, you can customize your username, password, and app tokens.
Password hashes can be generated with

```sh
rustical pwhash
```

### Password vs app tokens

The password is meant as a password you use to log in to the frontend.
Since it's sensitive information, the secure but slow hash algorithm `argon2` is chosen.

I recommend to generate random app tokens for each CalDAV/CardDAV client.
These can use the faster `pbkdf2` algorithm.

## Todo

- [ ] CalDAV
  - [x] Support for VEVENT, VTODO, VJOURNAL
  - [ ] Proper filtering for REPORT method
    - [x] comp-filter
    - [x] time-range filter
    - [x] good enough to use
    - [ ] prop-filter
  - [x] ICS parsing
    - [x] Datetime parsing
  - [x] Implement PROPPATCH
- [x] CardDAV
- [x] Auth
  - current state: RustiCal should be safe against unauthenticated request, however many routes are not checked for authorization yet
  - [x] static authentication
  - [ ] Access control
  - [x] preparation for different principal types (groups)
  - [x] authentication rewrite? (argon2 is very slow for each request)
    - [x] solved through app tokens
- [ ] Web UI
- [x] Trash bin
  - [x] Hiding calendars instead of deleting them
  - [x] Restore endpoint
- [ ] Packaging
  - [x] Ensure cargo install works
  - [x] Docker image
  - [ ] Releases
- [ ] Testing such that I'm confident enough to use it myself :)
- [x] WebDAV sync extension [RFC 6578](https://www.rfc-editor.org/rfc/rfc6578)
  - [x] implement getctag [see](https://github.com/apple/ccs-calendarserver/blob/master/doc/Extensions/caldav-ctag.txt)
  - [ ] implement WebDAV If header
- [x] Ensure proper routing
- [ ] Onboarding
  - [ ] config generation
  - [ ] usable documentation
  - [ ] usable frontend

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

## Sync-token

- a returned version of a resource is marked with a sync token
- the client can send the sync token to find out about changes after this sync token
