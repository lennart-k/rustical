CREATE TABLE calendars (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  owner TEXT NOT NULL,
  description TEXT,
  color TEXT,
  timezone TEXT NOT NULL
);

CREATE TABLE events (
  uid TEXT NOT NULL,
  cid TEXT NOT NULL,
  PRIMARY KEY (cid, uid),
  FOREIGN KEY (cid) REFERENCES calendars(uid)
);

