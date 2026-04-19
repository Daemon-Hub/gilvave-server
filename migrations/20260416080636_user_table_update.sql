-- Add migration script here
ALTER TABLE users
ADD COLUMN email TEXT UNIQUE NOT NULL,
ADD COLUMN is_active BOOLEAN DEFAULT true NOT NULL;


CREATE TABLE refresh_token (
    id UUID PRIMARY KEY,
    user_id UUID UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(128) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
