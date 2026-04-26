-- Your SQL goes here

CREATE TABLE IF NOT EXISTS public_playlist (
    id VARCHAR NOT NULL PRIMARY KEY,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    thumbnail_url VARCHAR,
    uploader_id VARCHAR NOT NULL REFERENCES channel(id),
    video_count INTEGER
);

CREATE TABLE IF NOT EXISTS playlist_bookmark (
    account_id VARCHAR NOT NULL REFERENCES account(id),
    public_playlist_id VARCHAR NOT NULL REFERENCES public_playlist(id),
    PRIMARY KEY (account_id, public_playlist_id)
)
