CREATE TABLE subscriptions (
    id TEXT NOT NULL,
    topic TEXT NOT NULL,
    expiration DATETIME NOT NULL,
    push_resource TEXT NOT NULL,
    PRIMARY KEY (id)
);
