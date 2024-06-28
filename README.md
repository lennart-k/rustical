# Rustical (WIP)

a calendar server

## Todo

- [ ] CalDAV
  - [ ] Support for VTODO, VJOURNAL
  - [ ] Proper filtering for REPORT method
  - [ ] ICS parsing
    - [x] Datetime parsing
  - [x] Implement PROPPATCH
- [ ] Auth
  - [ ] Access control
  - [ ] preparation for different principal types (groups)
  - [ ] authentication rewrite? (argon2 is very slow for each request)
  - [ ] OIDC support
- [ ] CardDAV
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
