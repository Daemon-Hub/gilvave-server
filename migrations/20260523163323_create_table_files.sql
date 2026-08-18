CREATE TABLE files (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    message_id UUID NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    -- Тип содержимого: "image/png", "video/mp4", "application/pdf" и т.д.
    -- Нужно для фронтенда, чтобы знать, как рендерить (тег <img> или ссылка на скачивание)
    -- mime_type TEXT NOT NULL,
    size_kb INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_message_id ON files (message_id);

CREATE INDEX idx_files_created_at ON files (created_at);