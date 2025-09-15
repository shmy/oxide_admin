-- Add migration script here
CREATE TABLE
    _jobs (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        kind VARCHAR(32) NOT NULL,
        args TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        reason TEXT,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX IF NOT EXISTS idx_jobs_status ON _jobs (status);