CREATE TABLE user_blocks (
    blocker_id  UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    blocked_id  UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (blocker_id, blocked_id),
    CONSTRAINT chk_user_blocks_no_self CHECK (blocker_id <> blocked_id)
);

CREATE INDEX idx_user_blocks_blocked_id ON user_blocks (blocked_id);