-- Включаем расширение для генерации UUID (если еще не включено)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Таблица серверов
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    name VARCHAR(100) NOT NULL,
    owner_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    icon_url TEXT NOT NULL,
    is_public BOOL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_server_members_owner_id ON servers (owner_id);

-- Таблица участников серверов
-- Связывает пользователей и серверы (многие-ко-многим)
CREATE TABLE server_members (
    server_id UUID REFERENCES servers (id) ON DELETE CASCADE,
    user_id UUID REFERENCES users (id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (server_id, user_id)
);

CREATE INDEX idx_server_members_user_id ON server_members (user_id);

-- Типы каналов (Text, Voice)
CREATE TYPE channel_type AS ENUM (
    'TEXT', 
    'VOICE'
);

-- Таблица каналов
CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    server_id UUID REFERENCES servers (id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    type channel_type DEFAULT 'TEXT' NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_channels_server_id ON channels (server_id);

-- Таблица сообщений
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL REFERENCES channels (id) ON DELETE CASCADE,
    author_id UUID REFERENCES users (id) ON DELETE SET NULL,
    author_name VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    edited BOOLEAN DEFAULT FALSE NOT NULL,
    reply_to_id UUID,
    forwarded_from JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Индексы
CREATE INDEX idx_messages_channel_id ON messages (channel_id);

CREATE INDEX idx_messages_author_id ON messages (author_id);

CREATE INDEX idx_messages_channel_covering ON messages (channel_id, created_at DESC) INCLUDE (author_name, content);