CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL REFERENCES channels (id) ON DELETE CASCADE,
    author_id UUID REFERENCES users (id) ON DELETE SET NULL,
    author_name VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    edited BOOLEAN DEFAULT FALSE NOT NULL,
    -- Идентификатор сообщения, на которое ответили
    reply_to_id UUID,
    -- Информация о месте откуда было переслано это сообщение
    forwarded_from JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_channel_id ON messages (channel_id);

CREATE INDEX idx_messages_author_id ON messages (author_id);

CREATE INDEX idx_messages_channel_covering ON messages (channel_id, created_at DESC) INCLUDE (author_name, content);