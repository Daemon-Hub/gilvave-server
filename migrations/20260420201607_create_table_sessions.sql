CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    refresh_token_hash BYTEA NOT NULL,
    device_info JSONB NOT NULL,
    ip_address INET NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ
);

CREATE INDEX idx_sessions_user_id ON sessions (user_id);

CREATE INDEX idx_sessions_refresh_token_hash ON sessions (refresh_token_hash);