CREATE TABLE davpush_vapid_key (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    vapid_key TEXT
);
INSERT INTO davpush_vapid_key (id) VALUES (1);
