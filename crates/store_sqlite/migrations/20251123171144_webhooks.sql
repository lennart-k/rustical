CREATE TABLE webhook_subscriptions (
    id TEXT NOT NULL,
    target_url TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    secret_key TEXT,
    PRIMARY KEY (id)
);