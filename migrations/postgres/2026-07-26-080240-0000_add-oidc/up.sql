-- make password hash nullable
ALTER TABLE account ALTER COLUMN password_hash DROP NOT NULL;
-- add oidc sub
ALTER TABLE account ADD oidc_sub VARCHAR NULL DEFAULT NULL;
