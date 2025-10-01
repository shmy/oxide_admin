-- Add migration script here
-- Add migration script here
CREATE TABLE
    _scheds (
        id CHAR(24) PRIMARY KEY NOT NULL,
        key VARCHAR(128) NOT NULL,
        name VARCHAR(256) NOT NULL,
        schedule VARCHAR(256) NOT NULL,
        succeed BOOLEAN NOT NULL,
        output TEXT,
        run_at TIMESTAMP NOT NULL,
        duration_ms BIGINT NOT NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX idx_scheds_key_created_at ON _scheds (key, created_at DESC);

CREATE INDEX idx_scheds_succeed_created_at ON _scheds (succeed, created_at DESC);