# Rustical (WIP)

a calendar server

## Installation

## Todo

- [ ] CalDAV
  - [ ] Support for VTODO, VJOURNAL
    - [x] VTODO
  - [ ] Proper filtering for REPORT method
    - [x] comp-filter
    - [ ] time-range filter
    - [ ] prop-filter
  - [x] ICS parsing
    - [x] Datetime parsing
  - [x] Implement PROPPATCH
- [ ] CardDAV
- [x] Auth (There currently is no authentication at all in place for some routes)
  - [x] static authentication
  - [ ] Access control
  - [x] preparation for different principal types (groups)
  - [ ] authentication rewrite? (argon2 is very slow for each request)
  - [ ] OIDC support
- [ ] Web UI
- [x] Trash bin
  - [x] Hiding calendars instead of deleting them
  - [ ] Restore endpoint
- [ ] Packaging
  - [x] Ensure cargo install works
  - [ ] Docker image
  - [ ] Releases
- [ ] Locking
- [ ] Testing such that I'm confident enough to use it myself :)
- [x] WebDAV sync extension [RFC 6578](https://www.rfc-editor.org/rfc/rfc6578)
  - [x] implement getctag [see](https://github.com/apple/ccs-calendarserver/blob/master/doc/Extensions/caldav-ctag.txt)
  - [ ] implement WebDAV If header
- [x] Ensure proper routing

## Relevant RFCs

- Versioning Extensions to WebDAV: [RFC 3253](https://datatracker.ietf.org/doc/html/rfc3253)
  - provides the REPORT method
- Calendaring Extensions to WebDAV (CalDAV): [RFC 4791](https://datatracker.ietf.org/doc/html/rfc4791)
- Scheduling Extensions to CalDAV: [RFC 6638](https://datatracker.ietf.org/doc/html/rfc6638)
  - not sure yet whether to implement this
- Collection Synchronization WebDAV [RFC 6578](https://datatracker.ietf.org/doc/html/rfc6578)
  - We need to implement sync-token, etc.
- This is important for more efficient synchronisation

## Sync-token

- a returned version of a resource is marked with a sync token
- the client can send the sync token to find out about changes after this sync token
