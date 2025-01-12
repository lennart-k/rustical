CREATE TABLE calendars (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    synctoken INTEGER DEFAULT 0 NOT NULL,
    displayname TEXT,
    description TEXT,
    "order" INT DEFAULT 0 NOT NULL,
    color TEXT,
    timezone TEXT,
    timezone_id TEXT,
    deleted_at DATETIME,
    subscription_url TEXT,
    push_topic TEXT UNIQUE NOT NULL,
    PRIMARY KEY (principal, id)
);

CREATE TABLE calendarobjects (
    principal TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    id TEXT NOT NULL,
    ics TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    PRIMARY KEY (principal, cal_id, id),
    FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);

CREATE TABLE calendarobjectchangelog (
    principal TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    object_id TEXT NOT NULL,
    "operation" INTEGER NOT NULL,
    synctoken INTEGER DEFAULT 0 NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (principal, cal_id, synctoken, created_at),
    FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);
