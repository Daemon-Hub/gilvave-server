-- Включаем расширение для генерации UUID (если еще не включено)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Таблица серверов
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    name VARCHAR(100) NOT NULL,
    owner_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE, -- Владелец сервера
    icon_url TEXT, -- Иконка сервера
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Таблица участников серверов
-- Связывает пользователей и серверы (многие-ко-многим)
CREATE TABLE server_members (
    server_id UUID REFERENCES servers (id) ON DELETE CASCADE,
    user_id UUID REFERENCES users (id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (server_id, user_id)
);

-- Типы каналов (Text, Voice)
CREATE TYPE channel_type AS ENUM ('text', 'voice');

-- Таблица каналов
CREATE TABLE channels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    server_id UUID REFERENCES servers (id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    type channel_type DEFAULT 'text',
    position INTEGER DEFAULT 0,
    topic TEXT, -- Описание канала
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Таблица сообщений
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID REFERENCES channels (id) ON DELETE CASCADE,
    author_id UUID REFERENCES users (id) ON DELETE SET NULL, -- Если пользователь удален, сообщение сохраняется, но автор = null
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Индексы
CREATE INDEX idx_messages_channel_id ON messages (channel_id);

CREATE INDEX idx_messages_author_id ON messages (author_id);

CREATE INDEX idx_server_members_user_id ON server_members (user_id);

CREATE INDEX idx_channels_server_id ON channels (server_id);