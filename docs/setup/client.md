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

## DAVx5

You can set up DAVx5 through the Nextcloud login flow. Collections including group collections will automatically be discovered.

## Apple Calendar

You can download a configuration profile from the frontend in the app token section.

**Limitation**: Group collections are not automatically discovered, for these you need to set up separate CalDAV configurations using the corresponding principal homes (but your own user id).

## Evolution

Set up a collection account in the account settings.

**Limitation**: Group collections are not discovered. It seems as if currently you have to add each group collection manually.

## Home Assistant CalDAV integration

As URL specify

```
https://<your-host>/.well-known/caldav
```

For goup collections explicitly specify

```
https://<your-host>/caldav/principal/<principal>
```
