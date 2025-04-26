CREATE TABLE principals (
    id TEXT NOT NULL,
    displayname TEXT,
    principal_type TEXT NOT NULL,
    password_hash TEXT,
    PRIMARY KEY (id)
);

CREATE TABLE app_tokens (
    id TEXT NOT NULL,
    principal TEXT NOT NULL,
    token TEXT NOT NULL,
    displayname TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE CASCADE
);

CREATE TABLE memberships (
    principal TEXT NOT NULL,
    member_of TEXT NOT NULL,
    PRIMARY KEY (principal, member_of),
    CONSTRAINT fk_membership_principal
    FOREIGN KEY (principal) REFERENCES principals (id) ON DELETE CASCADE,
    CONSTRAINT fk_membership_member_of
    FOREIGN KEY (member_of) REFERENCES principals (id) ON DELETE CASCADE
);
