-- make avatar url nullable
-- https://stackoverflow.com/questions/4007014/alter-column-in-sqlite
ALTER TABLE channel ALTER COLUMN avatar DROP NOT NULL;
