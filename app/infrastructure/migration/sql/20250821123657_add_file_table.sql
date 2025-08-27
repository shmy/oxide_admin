-- Add migration script here
CREATE TABLE
    _files (
        id CHAR(24) PRIMARY KEY NOT NULL,
        path VARCHAR(64) NOT NULL,
        status SMALLINT NOT NULL,
        created_at TIMESTAMP NOT NULL
    );

CREATE INDEX idx_files_path ON _files (path);

CREATE INDEX idx_files_status_created_at ON _files (status, created_at);
