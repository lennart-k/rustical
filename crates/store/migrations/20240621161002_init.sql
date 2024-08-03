CREATE TABLE calendars (
  principal TEXT NOT NULL,
  id TEXT NOT NULL,
  synctoken INTEGER DEFAULT 0 NOT NULL,
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
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  deleted_at DATETIME,
  PRIMARY KEY (principal, cid, uid),
  FOREIGN KEY (principal, cid) REFERENCES calendars(principal, id)
);

CREATE TABLE eventchangelog (
  -- The actual sync token is the SQLite field 'ROWID'
  principal TEXT NOT NULL,
  cid TEXT NOT NULL,
  uid TEXT NOT NULL,
  operation INTEGER NOT NULL,
  synctoken INTEGER DEFAULT 0 NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (principal, cid, created_at),
  FOREIGN KEY (principal, cid) REFERENCES calendars(principal, id) ON DELETE CASCADE
);
