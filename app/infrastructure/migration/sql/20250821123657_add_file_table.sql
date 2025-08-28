-- Add migration script here
CREATE TABLE
    _files (
        id CHAR(24) PRIMARY KEY NOT NULL,
        path VARCHAR(64) NOT NULL,
        used BOOLEAN NOT NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX idx_files_path ON _files (path);

CREATE INDEX idx_files_used_created_at ON _files (used, created_at);
