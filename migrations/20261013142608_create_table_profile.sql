CREATE TABLE profile (
    user_id UUID PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    bio TEXT NOT NULL DEFAULT '',
    avatar TEXT NOT NULL DEFAULT '',
    settings JSONB NOT NULL,
);