CREATE TABLE calendars (
  principal TEXT NOT NULL,
  id TEXT NOT NULL,
  displayname TEXT,
  description TEXT,
  'order' INT DEFAULT 0 NOT NULL,
  color TEXT,
  timezone TEXT NOT NULL,
  deleted_at DATETIME,
  PRIMARY KEY (principal, id)
);

CREATE TABLE events (
  principal TEXT NOT NULL,
  cid TEXT NOT NULL,
  uid TEXT NOT NULL,
  ics TEXT NOT NULL,
  deleted_at DATETIME,
  PRIMARY KEY (principal, cid, uid),
  FOREIGN KEY (principal, cid) REFERENCES calendars(principal, id)
);

