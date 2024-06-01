CREATE TABLE calendars (
  id TEXT PRIMARY KEY NOT NULL,
  owner TEXT NOT NULL,
  name TEXT,
  description TEXT,
  'order' INT DEFAULT 0 NOT NULL,
  color TEXT,
  timezone TEXT NOT NULL
);

CREATE TABLE events (
  uid TEXT NOT NULL,
  cid TEXT NOT NULL,
  ics TEXT NOT NULL,
  PRIMARY KEY (cid, uid),
  FOREIGN KEY (cid) REFERENCES calendars(id)
);

