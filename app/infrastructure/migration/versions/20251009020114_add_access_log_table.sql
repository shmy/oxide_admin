-- 访问日志表（Access Logs）
CREATE TABLE
    _access_logs (
        id CHAR(24) PRIMARY KEY NOT NULL,
        user_id CHAR(24) NOT NULL,
        method VARCHAR(8) NOT NULL,
        uri VARCHAR(256) NOT NULL,
        user_agent VARCHAR(512),
        ip VARCHAR(45),
        status SMALLINT NOT NULL,
        elapsed BIGINT NOT NULL,
        occurred_at TIMESTAMP NOT NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL
    );

-- 索引
CREATE INDEX idx_access_logs_user_id ON _access_logs (user_id);

CREATE INDEX idx_access_logs_occurred_at ON _access_logs (occurred_at);

CREATE INDEX idx_access_logs_status ON _access_logs (status);

CREATE INDEX idx_access_logs_elapsed ON _access_logs (elapsed);