CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    avatar TEXT NOT NULL DEFAULT ''
);

CREATE INDEX idx_users_username ON users (username);

CREATE INDEX idx_users_email ON users (email);