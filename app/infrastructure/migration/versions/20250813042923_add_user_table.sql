CREATE TABLE
    _users (
        id CHAR(24) PRIMARY KEY NOT NULL,
        account VARCHAR(32) UNIQUE NOT NULL,
        portrait VARCHAR(128),
        name VARCHAR(32) NOT NULL,
        privileged BOOLEAN NOT NULL,
        password VARCHAR(255) NOT NULL,
        role_ids CHAR(24)[] NOT NULL,
        enabled BOOLEAN NOT NULL,
        refresh_token VARCHAR(21),
        refresh_token_expired_at TIMESTAMP,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX index_users_account ON _users (account);

CREATE INDEX index_users_role_ids ON _users USING GIN (role_ids);

CREATE INDEX index_users_enabled ON _users (enabled);

CREATE INDEX index_users_refresh_token ON _users (refresh_token);
