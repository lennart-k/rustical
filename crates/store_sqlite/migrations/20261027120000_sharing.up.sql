-- Sharing a calendar or an addressbook with another principal (user or group).
-- A share row is both the grant and the sharee's view of the collection: `name` is the
-- collection's id in the sharee's home; NULL displayname/color/order means the owner's value.
CREATE TABLE calendar_shares (
    owner TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    sharee TEXT NOT NULL,
    name TEXT NOT NULL,
    access TEXT NOT NULL CHECK (access IN ('read', 'write')),
    displayname TEXT,
    color TEXT,
    "order" INT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_by TEXT,
    PRIMARY KEY (owner, cal_id, sharee),
    CONSTRAINT uq_calendar_share_name UNIQUE (sharee, name),
    CONSTRAINT fk_calendar_share_calendar FOREIGN KEY (owner, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE,
    CONSTRAINT fk_calendar_share_sharee FOREIGN KEY (sharee)
    REFERENCES principals (id) ON DELETE CASCADE
);

CREATE TABLE addressbook_shares (
    owner TEXT NOT NULL,
    addressbook_id TEXT NOT NULL,
    sharee TEXT NOT NULL,
    name TEXT NOT NULL,
    access TEXT NOT NULL CHECK (access IN ('read', 'write')),
    displayname TEXT,
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_by TEXT,
    PRIMARY KEY (owner, addressbook_id, sharee),
    CONSTRAINT uq_addressbook_share_name UNIQUE (sharee, name),
    CONSTRAINT fk_addressbook_share_addressbook FOREIGN KEY (owner, addressbook_id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE,
    CONSTRAINT fk_addressbook_share_sharee FOREIGN KEY (sharee)
    REFERENCES principals (id) ON DELETE CASCADE
);

-- How objects with CLASS:PRIVATE / CLASS:CONFIDENTIAL of an owner appear in shares.
-- Levels: 0 hidden, 1 busy, 2 title, 3 title and location, 4 full.
-- No row means (1, 0). CONFIDENTIAL never shows more than PRIVATE.
CREATE TABLE principal_visibility (
    principal TEXT NOT NULL,
    private INTEGER NOT NULL DEFAULT 1 CHECK (private BETWEEN 0 AND 4),
    confidential INTEGER NOT NULL DEFAULT 0 CHECK (confidential BETWEEN 0 AND 4),
    PRIMARY KEY (principal),
    CHECK (confidential <= private),
    CONSTRAINT fk_principal_visibility_principal FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE CASCADE
);

-- E-mail address of a principal (lowercase), to share by address.
CREATE TABLE principal_emails (
    principal TEXT NOT NULL,
    email TEXT NOT NULL,
    PRIMARY KEY (principal),
    CONSTRAINT uq_principal_email UNIQUE (email),
    CONSTRAINT fk_principal_email_principal FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE CASCADE
);
