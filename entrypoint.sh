#!/bin/sh

PUID=${PUID:-1000}
PGID=${PGID:-1000}

if [ "$(id -u)" -ne 0 ]; then
    exec "$@"
fi

if ! getent group rustical > /dev/null 2>&1; then
    addgroup -g "$PGID" rustical
fi

if ! id rustical > /dev/null 2>&1; then
    if ! getent passwd "$PUID" > /dev/null 2>&1; then
        adduser -D -u "$PUID" -G rustical rustical
    fi
fi

mkdir -p /var/lib/rustical
find /var/lib/rustical \( ! -user "$PUID" -o ! -group "$PGID" \) -exec chown "${PUID}:${PGID}" {} +

exec su-exec "$PUID:$PGID" "$@"
