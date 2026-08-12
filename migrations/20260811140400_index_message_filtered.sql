-- Add migration script here
CREATE INDEX idx_messages_channel_covering 
ON messages(channel_id, created_at DESC) 
INCLUDE (author_name, content);