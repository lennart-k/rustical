CREATE TABLE principals (
    id TEXT NOT NULL,
    displayname TEXT,
    principal_type TEXT NOT NULL,
    password_hash TEXT,
    PRIMARY KEY (id)
);

CREATE TABLE app_tokens (
    id TEXT NOT NULL,
    principal TEXT NOT NULL,
    token TEXT NOT NULL,
    displayname TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE CASCADE
);

CREATE TABLE memberships (
    principal TEXT NOT NULL,
    member_of TEXT NOT NULL,
    PRIMARY KEY (principal, member_of),
    CONSTRAINT fk_membership_principal
    FOREIGN KEY (principal) REFERENCES principals (id) ON DELETE CASCADE,
    CONSTRAINT fk_membership_member_of
    FOREIGN KEY (member_of) REFERENCES principals (id) ON DELETE CASCADE
);

CREATE TABLE calendars (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    synctoken BIGINT DEFAULT 0 NOT NULL,
    displayname TEXT,
    description TEXT,
    "order" BIGINT DEFAULT 0 NOT NULL,
    color TEXT,
    timezone_id TEXT,
    deleted_at TIMESTAMP,
    subscription_url TEXT,
    push_topic TEXT UNIQUE NOT NULL,
    comp_event BOOLEAN NOT NULL,
    comp_todo BOOLEAN NOT NULL,
    comp_journal BOOLEAN NOT NULL,
    PRIMARY KEY (principal, id),
    CONSTRAINT fk_calendar_principal FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE RESTRICT
);

CREATE TABLE calendarobjects (
    principal TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    id TEXT NOT NULL,
    "uid" TEXT NOT NULL,
    ics TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    first_occurence DATE,
    last_occurence DATE,
    etag TEXT,
    object_type INTEGER NOT NULL,
    CONSTRAINT pk_calendarobject_id PRIMARY KEY (principal, cal_id, id),
    CONSTRAINT uq_calendarobject_uid UNIQUE (principal, cal_id, "uid"),
    CONSTRAINT fk_calendarobject_calendar FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);

CREATE INDEX idx_calobjs_first_occurence ON calendarobjects (first_occurence);
CREATE INDEX idx_calobjs_last_occurence ON calendarobjects (last_occurence);
CREATE INDEX idx_calobjs_etag ON calendarobjects (etag);
CREATE INDEX idx_calobjs_obj_type ON calendarobjects (object_type);
CREATE INDEX idx_calobjs_uid ON calendarobjects (principal, cal_id, "uid");

CREATE TABLE calendarobjectchangelog (
    principal TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    "operation" INTEGER NOT NULL,
    synctoken BIGINT DEFAULT 0 NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (principal, cal_id, synctoken, created_at),
    FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);

CREATE INDEX idx_calobj_log_cal ON calendarobjectchangelog (cal_id);

CREATE TABLE addressbooks (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    synctoken BIGINT DEFAULT 0 NOT NULL,
    displayname TEXT,
    description TEXT,
    deleted_at TIMESTAMP,
    push_topic TEXT UNIQUE NOT NULL,
    PRIMARY KEY (principal, id),
    CONSTRAINT fk_addressbook_principal FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE RESTRICT
);

CREATE TABLE addressobjects (
    principal TEXT NOT NULL,
    addressbook_id TEXT NOT NULL,
    id TEXT NOT NULL,
    vcf TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    PRIMARY KEY (principal, addressbook_id, id),
    FOREIGN KEY (principal, addressbook_id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
);

CREATE TABLE addressobjectchangelog (
    principal TEXT NOT NULL,
    addressbook_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    "operation" INTEGER NOT NULL,
    synctoken BIGINT DEFAULT 0 NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (principal, addressbook_id, synctoken, created_at),
    FOREIGN KEY (principal, addressbook_id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
);

CREATE INDEX idx_addrobj_log_cal ON addressobjectchangelog (addressbook_id);

CREATE TABLE birthday_calendars (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    displayname TEXT,
    description TEXT,
    "order" BIGINT DEFAULT 0 NOT NULL,
    color TEXT,
    timezone_id TEXT,
    deleted_at TIMESTAMP,
    push_topic TEXT NOT NULL,
    PRIMARY KEY (principal, id),
    CONSTRAINT fk_birthdays_addressbooks FOREIGN KEY (principal, id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
);

CREATE TABLE davpush_subscriptions (
    id TEXT NOT NULL,
    topic TEXT NOT NULL,
    expiration TIMESTAMP NOT NULL,
    push_resource TEXT NOT NULL,
    public_key TEXT NOT NULL,
    public_key_type TEXT NOT NULL,
    auth_secret TEXT NOT NULL,
    PRIMARY KEY (id)
);
