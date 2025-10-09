CREATE TABLE
    _departments (
        id CHAR(24) PRIMARY KEY NOT NULL,
        name VARCHAR(256) NOT NULL,
        code VARCHAR(64) NOT NULL,
        parent_code VARCHAR(64),
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX idx_departments_parent_code ON _departments (parent_code);