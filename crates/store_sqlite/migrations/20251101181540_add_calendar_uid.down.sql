DROP INDEX idx_calobjs_uid;
ALTER TABLE calendarobjects RENAME TO calendarobjects_old;

CREATE TABLE calendarobjects (
    principal TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    id TEXT NOT NULL, -- filename
    ics TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,

    -- For more efficient calendar-queries
    first_occurence DATE,
    last_occurence DATE,
    etag TEXT,
    object_type INTEGER NOT NULL,  -- VEVENT(0)/VTODO(1)/VJOURNAL(2)

    CONSTRAINT pk_calendarobject_id PRIMARY KEY (principal, cal_id, id),
    CONSTRAINT fk_calendarobject_calendar FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);

INSERT INTO calendarobjects (
    principal,
    cal_id,
    id,
    ics,
    updated_at,
    deleted_at,
    first_occurence,
    last_occurence,
    etag,
    object_type
) SELECT
    principal,
    cal_id,
    id,
    ics,
    updated_at,
    deleted_at,
    first_occurence,
    last_occurence,
    etag,
    object_type
FROM calendarobjects_old;

DROP TABLE calendarobjects_old;
