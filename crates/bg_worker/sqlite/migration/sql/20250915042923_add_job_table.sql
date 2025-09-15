-- Add migration script here
CREATE TABLE
    _jobs (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        kind VARCHAR(32) NOT NULL,
        args TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );