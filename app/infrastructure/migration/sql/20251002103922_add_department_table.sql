-- Add migration script here
CREATE TABLE
    _departments (
        id CHAR(24) PRIMARY KEY NOT NULL,
        name VARCHAR(256) NOT NULL,
        code VARCHAR(64) NOT NULL,
        parent_id CHAR(24),
        enabled BOOLEAN NOT NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX idx_departments_parent_id ON _departments (parent_id);