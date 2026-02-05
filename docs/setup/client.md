# Client Setup

## Common

Following resources are available.

```
/.well-known/caldav
# CalDAV root
/caldav
# Principal home
/caldav/principal/<user_id>
# Calendar home
/caldav/principal/<user_id>/<calendar_id>
/caldav/principal/<user_id>/_birthdays_<addressbook_id>

# CalDAV root
/caldav-compat
/caldav-compat/principal...
```

```
/.well-known/carddav
# CardDAV root
/carddav
# Principal home
/carddav/principal/<user_id>
# Addressbook home
/carddav/principal/<user_id>/<addressbook_id>
```

### Authentication

Authenticate with HTTP Basic authentication using your user id and a generated app token.

Note that the user id is not allowed to contain a colon (`:`) character.
Furthermore, the dollar (`$`) character is also forbidden.

#### Principal impersonation

If a user (e.g. alice) is member of a group (i.e. group.amazing) a client can authenticate as the group by specifying the username
`alice$group.amazing`.
This only applies for the caldav/carddav endpoints and is an unfortunate workaround since Apple does not properly adhere to their own spec and ignores its own configuration options. :(

## `/caldav` vs `/caldav-compat` (relevant for group sharing)

To discover shared calendars the `calendar-home-set` property is used to list all principals the user has access to.
However, some clients don't support `calendar-home-set` containing multiple paths (e.g. Apple Calendar).

As a workaround `/caldav-compat` offers the same endpoints as `/caldav` with the only difference being that it does not return all calendar homes in `calendar-home-set`.
This means that clients under this path will probably not auto-discover group calendars so you can instead add them one-by-one using the principal path `/caldav-compat/principal/<principal_id>`.

## DAVx5

You can set up DAVx5 through the Nextcloud login flow. Collections including group collections will automatically be discovered.

## Apple Calendar

You can download a configuration profile from the frontend in the app token section. This will include CalDAV profiles for all groups the user is a member of.

When the memberships of a user change the user is required to download a new configuration profile from the frontend.

This is because Apple does not properly adhere to the CalDAV specification (which they authored).

**Note**: Since Apple Calendar does not properly support the `calendar-home-set` property the `/caldav-compat` endpoints should be used.
That also means that Apple Calendar is not able to automatically discover group collections so in that case you'll have to manually add all principals with `/caldav-compat/principal/<principal_id>`.

## Evolution

Set up a collection account in the account settings.
Evolution correctly uses all calendar homes so group collections work properly.

## Home Assistant CalDAV integration

The underlying library `python-caldav` does not support multiple calendar homes so you should use the `/caldav-compat` endpoints.

As URL specify

```
https://<your-host>/caldav-compat
```

For group collections explicitly specify

```
https://<your-host>/caldav-compat/principal/<principal>
```

## Thunderbird

- Go to `New Account -> Calendar -> On The Network`
- Specify the root path of RustiCal
- Thunderbird will properly discover group calendars
