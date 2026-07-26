-- make password hash nullable
-- https://stackoverflow.com/questions/4007014/alter-column-in-sqlite
CREATE TABLE account_temp(
    id VARCHAR PRIMARY KEY NOT NULL,
    name_hash VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NULL
);
INSERT INTO account_temp SELECT * FROM account;
DROP TABLE account;
ALTER TABLE account_temp RENAME TO account;

-- add oidc sub
ALTER TABLE account ADD oidc_sub VARCHAR NULL DEFAULT NULL;
