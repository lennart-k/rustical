SELECT load_extension("uuid");
SELECT uuid_blob();

CREATE TABLE new_principals (
    -- uuid is an internal identifier to allow the principal
    -- to be renamed in the future
    uuid BLOB PRIMARY KEY,
    -- id is the principl identifier used by public interfaces
    id TEXT UNIQUE NOT NULL,
    displayname TEXT,
    principal_type TEXT NOT NULL,
    password_hash TEXT
) STRICT;

CREATE TABLE new_calendars (
    -- UUID to be saved as 16-byte BLOB (much more efficient than text)
    -- disadvantage: less ergonomic in sqlite shell
    uuid BLOB PRIMARY KEY,  -- a globally unique UUID (not the resource path, reference for calendar objects)
    owner BLOB NOT NULL,  -- only the owner can manage shares
    synctoken INTEGER DEFAULT 0 NOT NULL,
    subscription_url TEXT,
    push_topic TEXT UNIQUE NOT NULL,
    comp_event BOOLEAN NOT NULL,
    comp_todo BOOLEAN NOT NULL,
    comp_journal BOOLEAN NOT NULL,
    timezone_id TEXT,
    CONSTRAINT fk_calendar_owner FOREIGN KEY (owner)
    REFERENCES new_principals (uuid) ON DELETE RESTRICT
) STRICT;

CREATE TABLE calendar_views (
    ref TEXT NOT NULL,  -- the underlying calendar
    principal BLOB NOT NULL,  -- the principal having access to this calendar view
    name TEXT NOT NULL,   -- the path (what previously was the id)
    access TEXT NOT NULL,  -- read, write

    displayname TEXT,
    description TEXT,
    "order" INT DEFAULT 0 NOT NULL,
    color TEXT,
    deleted_at DATETIME,  -- a user can delete its calendar view without deleting it for everyone

    PRIMARY KEY (principal, name),
    CONSTRAINT uniq_cal_principal(ref, principal) UNIQUE,  -- a user should not have mutiple views on a calendar
    CONSTRAINT fk_calendar_view_principal FOREIGN KEY (principal)
    REFERENCES new_principals (uuid) ON DELETE CASCADE,
    CONSTRAINT fk_calendar_view_calendar FOREIGN KEY (ref)
    REFERENCES calendars (id) ON DELETE CASCADE
) STRICT;
