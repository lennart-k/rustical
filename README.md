# RustiCal

a CalDAV/CardDAV server

> [!WARNING]
  RustiCal is under **active development**!
  While I've been successfully using RustiCal productively for some months now and there seems to be a growing user base,
  you'd still be one of the first testers so expect bugs and rough edges.
  If you still want to use it in its current state, absolutely feel free to do so and to open up an issue if something is not working. :)

## Features

- easy to backup, everything saved in one SQLite database
  - also export feature in the frontend
- Import your existing calendars in the frontend
- **[WebDAV Push](https://github.com/bitfireAT/webdav-push/)** support, so near-instant synchronisation to DAVx5
- lightweight (the container image contains only one binary)
- adequately fast (I'd love to say blazingly fast™ :fire: but I don't have any benchmarks)
- deleted calendars are recoverable
- Nextcloud login flow (In DAVx5 you can login through the Nextcloud flow and automatically generate an app token)
- Apple configuration profiles (skip copy-pasting passwords and instead generate the configuration in the frontend)
- **OpenID Connect** support (with option to disable password login)
- Group-based **sharing**

## Getting Started

- Check out the [documentation](https://lennart-k.github.io/rustical/installation/)

## Tested Clients

- DAVx5,
- GNOME Accounts, GNOME Calendar, GNOME Contacts
- Evolution
- Apple Calendar
- Home Assistant integration
- Thunderbird
