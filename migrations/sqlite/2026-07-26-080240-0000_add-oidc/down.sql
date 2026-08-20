-- This file should undo anything in `up.sql`
DELETE FROM account WHERE oidc_sub IS NOT NULL;   

ALTER TABLE account DROP COLUMN oidc_sub;
CREATE TABLE account_temp(
    id VARCHAR PRIMARY KEY NOT NULL,
    name_hash VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL
);
INSERT INTO account_temp SELECT * FROM account;
DROP TABLE account;
ALTER TABLE account_temp RENAME TO account;
