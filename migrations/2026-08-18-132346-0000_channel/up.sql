-- make avatar url nullable
-- https://stackoverflow.com/questions/4007014/alter-column-in-sqlite
CREATE TABLE channel_temp
(
    id VARCHAR(24) PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    avatar VARCHAR NULL,
    verified BOOLEAN NOT NULL
);
INSERT INTO channel_temp SELECT * FROM channel;
DROP TABLE channel;
ALTER TABLE channel_temp RENAME TO channel;
