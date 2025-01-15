CREATE TABLE addressbooks (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    synctoken INTEGER DEFAULT 0 NOT NULL,
    displayname TEXT,
    description TEXT,
    deleted_at DATETIME,
    push_topic TEXT UNIQUE NOT NULL,
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
    principal TEXT NOT NULL,
    addressbook_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    "operation" INTEGER NOT NULL,
    synctoken INTEGER DEFAULT 0 NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (principal, addressbook_id, synctoken, created_at),
    FOREIGN KEY (principal, addressbook_id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
);

CREATE INDEX idx_addrobj_log_cal ON addressobjectchangelog (addressbook_id);
