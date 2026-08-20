-- Your SQL goes here

CREATE TABLE IF NOT EXISTS public_playlist (
    id VARCHAR NOT NULL PRIMARY KEY,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    thumbnail_url VARCHAR,
    uploader_id VARCHAR NOT NULL CONSTRAINT FK__public_playlist__channel REFERENCES channel(id) ON DELETE RESTRICT,
    video_count INTEGER
);

CREATE TABLE IF NOT EXISTS playlist_bookmark (
    account_id VARCHAR NOT NULL CONSTRAINT FK__playlist_bookmark__account REFERENCES account(id) ON DELETE CASCADE,
    public_playlist_id VARCHAR NOT NULL CONSTRAINT FK__playlist_bookmark__public_playlist REFERENCES public_playlist(id) ON DELETE RESTRICT,
    PRIMARY KEY (account_id, public_playlist_id)
)
