CREATE TYPE channel_type AS ENUM (
    'TEXT', 
    'VOICE'
);

CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    server_id UUID REFERENCES servers (id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    type channel_type DEFAULT 'TEXT' NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_channels_server_id ON channels (server_id);