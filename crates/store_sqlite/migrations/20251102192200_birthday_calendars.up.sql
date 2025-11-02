CREATE TABLE birthday_calendars (
    principal TEXT NOT NULL,
    id TEXT NOT NULL,
    displayname TEXT,
    description TEXT,
    "order" INT DEFAULT 0 NOT NULL,
    color TEXT,
    timezone_id TEXT,
    deleted_at DATETIME,
    push_topic TEXT NOT NULL,
    PRIMARY KEY (principal, id),
    CONSTRAINT fk_birthdays_addressbooks FOREIGN KEY (principal, id)
    REFERENCES addressbooks (principal, id) ON DELETE CASCADE
    -- birthday calendar stores no meaningful data so we can cascade
);

INSERT INTO birthday_calendars
(principal, id, displayname, deleted_at, push_topic)
SELECT
    principal,
    id,
    displayname || ' birthdays' AS displayname,
    deleted_at,
    push_topic || substr(printf('%d', random()), -4) AS push_topic
    -- jank suffix to ensure that new push_topic is different :D
FROM addressbooks;
