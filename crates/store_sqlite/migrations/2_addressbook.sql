CREATE TABLE addressbooks (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    synctoken INTEGER DEFAULT 0 NOT NULL,
    displayname TEXT,
    description TEXT,
    deleted_at DATETIME,
    PRIMARY KEY (principal, id)
);

CREATE TABLE addressobjects (
    principal TEXT NOT NULL,
    addressbook_id TEXT NOT NULL,
    id TEXT NOT NULL,
    vcf TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    PRIMARY KEY (principal, addressbook_id, id),
    FOREIGN KEY (principal, addressbook_id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
);

CREATE TABLE addressobjectchangelog (
    -- The actual sync token is the SQLite field 'ROWID'
    principal TEXT NOT NULL,
    addressbook_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    operation INTEGER NOT NULL,
    synctoken INTEGER DEFAULT 0 NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (principal, addressbook_id, created_at),
    FOREIGN KEY (principal, addressbook_id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
);
