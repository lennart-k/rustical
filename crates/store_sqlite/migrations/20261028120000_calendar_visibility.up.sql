-- How objects with CLASS:PRIVATE / CLASS:CONFIDENTIAL of a calendar appear in its shares, per
-- calendar instead of per owner. Levels: 0 hidden, 1 busy, 2 title, 3 title and location, 4 full.
-- No row means (0, 0). CONFIDENTIAL never shows more than PRIVATE.
CREATE TABLE calendar_visibility (
    principal TEXT NOT NULL,
    cal_id TEXT NOT NULL,
    private INTEGER NOT NULL DEFAULT 0 CHECK (private BETWEEN 0 AND 4),
    confidential INTEGER NOT NULL DEFAULT 0 CHECK (confidential BETWEEN 0 AND 4),
    PRIMARY KEY (principal, cal_id),
    CHECK (confidential <= private),
    CONSTRAINT fk_calendar_visibility_calendar FOREIGN KEY (principal, cal_id)
    REFERENCES calendars (principal, id) ON DELETE CASCADE
);

-- The per-owner policy it replaces (no row is carried over: every calendar starts hidden)
DROP TABLE principal_visibility;
