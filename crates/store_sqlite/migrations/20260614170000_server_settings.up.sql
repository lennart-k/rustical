-- Single-row table for server-wide settings. First column is the persistent
-- DAV Push VAPID keypair (PEM), generated once so the advertised public key
-- stays stable across restarts. Add future settings as further columns.
CREATE TABLE server_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    vapid_key TEXT
);
INSERT INTO server_settings (id) VALUES (1);
