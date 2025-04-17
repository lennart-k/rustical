# RustiCal

a CalDAV/CardDAV server

!!! warning
    RustiCal is **not production-ready!**
    There can be changes to the database without migrations and there's no guarantee that all endpoints are secured yet.
    If you still want to play around with it in its current state, absolutely feel free to do so but know that not even I use it productively yet.

## Features

- easy to backup, everything saved in one SQLite database
- [WebDAV Push](https://github.com/bitfireAT/webdav-push/) support, so near-instant synchronisation to DAVx5
- lightweight (the container image contains only one binary)
- adequately fast (I'd say blazingly fast™ :fire: if I did the benchmarks to back that claim up)
- deleted calendars are recoverable
- Nextcloud login flow (In DAVx5 you can login through the Nextcloud flow and automatically generate an app token)
- [OpenID Connect](setup/oidc.md) support (with option to disable password login)
