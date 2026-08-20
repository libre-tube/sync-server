-- IMPORTANT: to successfully create tables in SQLite, there may not be any trailing comma in the create statement
-- idk who thought that this would be a good idea, but well ...

-- The `user` keyword is reserved in PostgreSQL, so the user table is named `account`.
CREATE TABLE IF NOT EXISTS account
(
    id VARCHAR PRIMARY KEY NOT NULL,
    name_hash VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS channel
(
    id VARCHAR(24) PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    avatar VARCHAR NOT NULL,
    verified BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS subscription
(
    account_id VARCHAR NOT NULL CONSTRAINT FK__subscription__account REFERENCES account(id) ON DELETE CASCADE,
    channel_id VARCHAR NOT NULL CONSTRAINT FK__subscription__channel REFERENCES channel(id) ON DELETE RESTRICT,
    PRIMARY KEY (account_id, channel_id)
);

CREATE TABLE IF NOT EXISTS playlist
(
    id VARCHAR PRIMARY KEY NOT NULL,
    account_id VARCHAR NOT NULL CONSTRAINT FK__playlist__account REFERENCES account(id) ON DELETE CASCADE,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    thumbnail_url VARCHAR
);

CREATE TABLE IF NOT EXISTS video
(
    id VARCHAR(11) PRIMARY KEY NOT NULL,
    title VARCHAR NOT NULL,
    upload_date BIGINT NOT NULL,
    uploader_id VARCHAR NOT NULL CONSTRAINT FK__video__channel REFERENCES channel(id) ON DELETE CASCADE,
    thumbnail_url VARCHAR NOT NULL,
    duration INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_video_member
(
    playlist_id VARCHAR NOT NULL CONSTRAINT FK__playlist_video_member__playlist REFERENCES playlist(id) ON DELETE CASCADE,
    video_id VARCHAR NOT NULL CONSTRAINT FK__playlist_video_member__video REFERENCES video(id) ON DELETE RESTRICT,
    PRIMARY KEY (playlist_id, video_id)
)
