# Rustical (WIP)

a calendar server

## Installation

## Todo

- [ ] CalDAV
  - [ ] Support for VTODO, VJOURNAL
  - [ ] Proper filtering for REPORT method
  - [ ] ICS parsing
    - [x] Datetime parsing
  - [x] Implement PROPPATCH
- [ ] Auth (There currently is no authentication at all in place for some routes)
  - [ ] Access control
  - [ ] preparation for different principal types (groups)
  - [ ] authentication rewrite? (argon2 is very slow for each request)
  - [ ] OIDC support
- [ ] CardDAV
- [ ] Packaging
  - [x] Ensure cargo install works
  - [ ] Docker image
  - [ ] Releases
- [ ] Locking
- [ ] Web UI
- [ ] Testing such that I'm confident enough to use it myself :)
- [ ] WebDAV sync extension [RFC 6578](https://www.rfc-editor.org/rfc/rfc6578)
  - [ ] implement getctag [see](https://github.com/apple/ccs-calendarserver/blob/master/doc/Extensions/caldav-ctag.txt)
- [ ] Ensure proper routing
- [x] Trash bin
  - [x] Hiding calendars instead of deleting them
  - [ ] Restore endpoint

## Relevant RFCs

- Versioning Extensions to WebDAV: [RFC 3253](https://datatracker.ietf.org/doc/html/rfc3253)
  - provides the REPORT method
- Calendaring Extensions to WebDAV (CalDAV): [RFC 4791](https://datatracker.ietf.org/doc/html/rfc4791)
- Scheduling Extensions to CalDAV: [RFC 6638](https://datatracker.ietf.org/doc/html/rfc6638)
  - not sure yet whether to implement this
- Collection Synchronization WebDAV [RFC 6578](https://datatracker.ietf.org/doc/html/rfc6578)
  - We need to implement sync-token, etc.
- This is important for more efficient synchronisation
