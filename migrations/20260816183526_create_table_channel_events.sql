CREATE TYPE event_type AS ENUM (
    'EDIT', 
    'DELETE', 
    'PIN', 
    'UNPIN', 
    'REACTION_ADD', 
    'REACTION_REMOVE'
);

CREATE TABLE channel_events (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    channel_id UUID NOT NULL REFERENCES channels (id) ON DELETE CASCADE,
    message_id UUID NOT NULL,
    type event_type NOT NULL,
    payload TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_channel_events_channel_id ON channel_events (channel_id);

CREATE INDEX idx_channel_events_created_at ON channel_events (created_at);

CREATE INDEX idx_channel_events_type ON channel_events(type);

CREATE INDEX idx_channel_events_channel_created ON channel_events (channel_id, created_at DESC);

CREATE INDEX idx_channel_events_channel_type ON channel_events(channel_id, type);