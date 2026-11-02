CREATE TABLE friends (
    user_id1 UUID REFERENCES users (id) ON DELETE CASCADE,
    user_id2 UUID REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id1, user_id2),
    CONSTRAINT chk_friends_canonical_order CHECK (user_id1 < user_id2)
);

CREATE INDEX idx_user_id2 ON friends (user_id2);