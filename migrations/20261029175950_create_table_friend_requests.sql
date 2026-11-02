CREATE TABLE friend_requests (
    sender_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (sender_id, receiver_id),
    CONSTRAINT chk_friend_requests_no_self CHECK (sender_id <> receiver_id)
);

CREATE INDEX idx_friend_requests_receiver_id ON friend_requests (receiver_id);