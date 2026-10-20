CREATE TABLE themes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    name VARCHAR(64) NOT NULL,
    author VARCHAR(64) NOT NULL,
    description TEXT,
    config JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL
);