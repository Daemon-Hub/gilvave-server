CREATE TABLE profile (
    user_id UUID PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    bio TEXT NOT NULL DEFAULT '',
    avatar TEXT NOT NULL DEFAULT '',
    settings JSONB NOT NULL DEFAULT '{}'::jsonb
);