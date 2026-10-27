CREATE TABLE principal_visibility (
    principal TEXT NOT NULL,
    private INTEGER NOT NULL DEFAULT 1 CHECK (private BETWEEN 0 AND 4),
    confidential INTEGER NOT NULL DEFAULT 0 CHECK (confidential BETWEEN 0 AND 4),
    PRIMARY KEY (principal),
    CHECK (confidential <= private),
    CONSTRAINT fk_principal_visibility_principal FOREIGN KEY (principal)
    REFERENCES principals (id) ON DELETE CASCADE
);

DROP TABLE calendar_visibility;
