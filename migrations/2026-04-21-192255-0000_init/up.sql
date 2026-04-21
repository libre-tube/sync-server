-- IMPORTANT: to successfully create tables, there may not be any trailing comma in the create statement
-- idk who thought that this would be a good idea, but well ...

CREATE TABLE IF NOT EXISTS user(
    id VARCHAR PRIMARY KEY NOT NULL,
    name_hash VARCHAR NOT NULL,
    password_hash VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS channel
(
    id VARCHAR PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    avatar VARCHAR NOT NULL,
    verified BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS subscription
(
    user_id VARCHAR NOT NULL REFERENCES user(id),
    channel_id VARCHAR NOT NULL REFERENCES channel(id),
    PRIMARY KEY (user_id, channel_id)
);

CREATE TABLE IF NOT EXISTS playlist
(
    id VARCHAR PRIMARY KEY NOT NULL,
    user_id VARCHAR NOT NULL REFERENCES user(id),
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    thumbnail_url VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_video
(
    id VARCHAR(11) PRIMARY KEY NOT NULL,
    title VARCHAR NOT NULL,
    upload_date VARCHAR NOT NULL,
    uploader VARCHAR NOT NULL REFERENCES channel(id),
    thumbnail_url VARCHAR NOT NULL,
    duration INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS playlist_video_member
(
    playlist_id VARCHAR NOT NULL REFERENCES playlist(id),
    video_id VARCHAR NOT NULL REFERENCES playlist_video(id),
    PRIMARY KEY (playlist_id, video_id)
)
