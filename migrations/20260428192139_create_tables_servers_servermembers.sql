-- Включаем расширение для генерации UUID (если еще не включено)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    owner_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon_url TEXT NOT NULL DEFAULT '',
    cover TEXT NOT NULL DEFAULT '',
    is_public BOOL NOT NULL,
    members_count INTEGER NOT NULL DEFAULT 0,
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

-- Функция для INSERT
CREATE OR REPLACE FUNCTION increment_member_count()
RETURNS TRIGGER AS $$ BEGIN
    UPDATE servers SET members_count = members_count + 1 WHERE id = NEW.server_id;
    RETURN NEW;
END;
 $$ LANGUAGE plpgsql;

CREATE TRIGGER trg_increment_member_count
AFTER INSERT ON server_members
FOR EACH ROW EXECUTE FUNCTION increment_member_count();

-- Функция для DELETE
CREATE OR REPLACE FUNCTION decrement_member_count()
RETURNS TRIGGER AS $$ BEGIN
    UPDATE servers SET members_count = members_count - 1 WHERE id = OLD.server_id;
    RETURN OLD;
END;
 $$ LANGUAGE plpgsql;

CREATE TRIGGER trg_decrement_member_count
AFTER DELETE ON server_members
FOR EACH ROW EXECUTE FUNCTION decrement_member_count();