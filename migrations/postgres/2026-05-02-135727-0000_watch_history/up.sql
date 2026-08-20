-- Your SQL goes here
CREATE TABLE IF NOT EXISTS watch_history(
    video_id VARCHAR NOT NULL CONSTRAINT FK__watch_history__video REFERENCES video(id) ON DELETE RESTRICT,
    account_id VARCHAR NOT NULL CONSTRAINT FK__watch_history__account REFERENCES account(id) ON DELETE CASCADE,
    added_date BIGINT NOT NULL,
    watched_state VARCHAR CHECK(watched_state IN ('planned', 'watching', 'completed', 'dropped')) NOT NULL,
    position_millis INTEGER,
    PRIMARY KEY (video_id, account_id)
);

CREATE INDEX watch_history_watched_state ON watch_history (watched_state);
CREATE INDEX watch_history_added_date ON watch_history (added_date);
