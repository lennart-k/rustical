# RustiCal

a CalDAV/CardDAV server

> [!WARNING]
  RustiCal is **not production-ready!**
  While I've started migrating to RustiCal and becoming more confident,
  please know that bugs and rough edges will still occur.
  Concretely, if you are using Apple Calendar you will want to stay away from assigning groups to users.
  If you still want to play around with it in its current state, absolutely feel free to do so and to open up an issue if something is not working. :)

## Features

- easy to backup, everything saved in one SQLite database
- ~~[WebDAV Push](https://github.com/bitfireAT/webdav-push/) support, so near-instant synchronisation to DAVx5~~ (currently broken)
- lightweight (the container image contains only one binary)
- adequately fast (I'd love to say blazingly fast™ :fire: but I don't have any benchmarks)
- deleted calendars are recoverable
- Nextcloud login flow (In DAVx5 you can login through the Nextcloud flow and automatically generate an app token)
- Apple configuration profiles (skip copy-pasting passwords and instead generate the configuration in the frontend)
- OpenID Connect support (with option to disable password login)

## Getting Started

- Check out the [documentation](https://lennart-k.github.io/rustical/installation/)

## Tested Clients

- DAVx5,
- GNOME Accounts, GNOME Calendar, GNOME Contacts
- Evolution
- Apple Calendar (known issue: If a user is member of multiple groups then Apple Calendar just randomly selects a calendar home)
