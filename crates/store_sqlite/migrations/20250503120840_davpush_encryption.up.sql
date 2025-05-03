-- Old subscriptions are useless anyway
DELETE FROM davpush_subscriptions;

-- Now the new columns can also be set NOT NULL
ALTER TABLE davpush_subscriptions ADD public_key TEXT NOT NULL;
ALTER TABLE davpush_subscriptions ADD public_key_type TEXT NOT NULL;
ALTER TABLE davpush_subscriptions ADD auth_secret TEXT NOT NULL;
