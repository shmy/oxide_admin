-- Add migration script here
CREATE TABLE
    _roles (
        id CHAR(24) PRIMARY KEY NOT NULL,
        name VARCHAR(32) UNIQUE NOT NULL,
        privileged BOOLEAN NOT NULL,
        menus INTEGER[] NOT NULL,
        permissions INTEGER[] NOT NULL,
        enabled BOOLEAN NOT NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

CREATE INDEX idx_roles_name ON _roles (name);

CREATE INDEX index_users_permissions ON _roles USING GIN (permissions);

CREATE INDEX index_users_menus ON _roles USING GIN (menus);

CREATE INDEX idx_roles_enabled ON _roles (enabled);
