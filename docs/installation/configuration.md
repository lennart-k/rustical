# Configuration

While RustiCal (apart from user management) will work without any configuration you should still know how to configure it. :)

You can either mount a `config.toml` file or use environment variables (recommended).

To see the options you can generate a default configuration using

```sh title="Generate default config.toml"
rustical gen-config
```

To see all configuration options available you can browse the [Cargo docs](/rustical/_crate/rustical/config/struct.Config.html).

## Environment variables

The options in `config.toml` can also be configured using environment variables.
Names translate the following:

```toml title="Example config.toml"
[data_store.toml]
path = "asd"
```

becomes `RUSTICAL_DATA_STORE__TOML__PATH`.
Every variable is

- uppercase
- prefixed by `RUSTICAL_`
- Dots become `__`
- Arrays are JSON-encoded

## Admin settings

The admin frontend is disabled by default and gets enabled by setting an `admin_group`:

```toml
[frontend.admin]
# Name of the admin group, every principal in that group gets admin privileges
admin_group = "admin_group"
# One of "no", "once" or "always"; "once" is set to "no" in this file after the first start
bootstrap_admins = "once"

# Principals listed here are created on startup if they do not exist yet.
[[frontend.admin.initial_admins]]
id = "admin"
displayname = "Administrator"
# Use either `password` or `password_hash`. A plaintext `password` is hashed on first start and replaced with `password_hash` in this file.
# Better: precalculate the hash using `rustical hash-password`.
password = "change-me"
```

A principal is an admin if it is a direct member of `admin_group`; if `admin_group` is unset,
nobody has admin access. The group is created if it does not exist. Bootstrapping never
modifies existing principals, but it re-adds them to the admin group, so an admin who removed themselves regains access on the next run. If the config file cannot be updated, the server logs a warning and continues.

With OIDC, admin access can also be granted by mapping the admin group through `[oidc.assign_memberships]`, no password needed.

To disable local password login for the whole instance, use:

```toml
[frontend]
allow_password_login = false
```
