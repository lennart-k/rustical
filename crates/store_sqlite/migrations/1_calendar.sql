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

    -- For more efficient calendar-queries
    first_occurence DATE,
    last_occurence DATE,
    etag TEXT,
    object_type INTEGER NOT NULL,  -- VEVENT(0)/VTODO(1)/VJOURNAL(2)

    PRIMARY KEY (principal, cal_id, id),
    FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);

CREATE INDEX idx_calobjs_first_occurence ON calendarobjects (first_occurence);
CREATE INDEX idx_calobjs_last_occurence ON calendarobjects (last_occurence);
CREATE INDEX idx_calobjs_etag ON calendarobjects (etag);
CREATE INDEX idx_calobjs_obj_type ON calendarobjects (object_type);

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

CREATE INDEX idx_calobj_log_cal ON calendarobjectchangelog (cal_id);
