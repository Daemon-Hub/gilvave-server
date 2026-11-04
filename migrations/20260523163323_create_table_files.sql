CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    name TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    message_id UUID REFERENCES messages (id) ON DELETE CASCADE,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_kb INTEGER NOT NULL,
    uploader_id UUID REFERENCES users (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_message_id ON files (message_id);

CREATE INDEX idx_files_created_at ON files (created_at);

CREATE INDEX idx_files_uploader_id ON files (uploader_id);
