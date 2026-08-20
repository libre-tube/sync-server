-- Your SQL goes here
DROP TABLE IF EXISTS playlist CASCADE;
DROP TABLE IF EXISTS playlist_video_member CASCADE;

CREATE TABLE playlist(
	id VARCHAR NOT NULL,
	account_id VARCHAR NOT NULL,
	title VARCHAR NOT NULL,
	description VARCHAR NOT NULL,
	thumbnail_url VARCHAR,
	PRIMARY KEY(id,account_id),
	CONSTRAINT FK__playlist__account FOREIGN KEY(account_id) REFERENCES account(id) ON DELETE CASCADE
);
CREATE TABLE playlist_video_member(
	account_id VARCHAR NOT NULL,
	playlist_id VARCHAR NOT NULL,
	video_id VARCHAR NOT NULL,
	PRIMARY KEY(account_id,playlist_id,	video_id),
	CONSTRAINT FK__playlist_video_member__account FOREIGN KEY(account_id) REFERENCES account(id) ON DELETE CASCADE,
	CONSTRAINT FK__playlist_video_member__playlist FOREIGN KEY(playlist_id, account_id) REFERENCES playlist(id,	account_id) ON DELETE CASCADE,
	CONSTRAINT FK__playlist_video_member__video FOREIGN KEY(video_id) REFERENCES video(id) ON DELETE RESTRICT
);
