-- This file should undo anything in `up.sql`
DELETE FROM account WHERE oidc_sub IS NOT NULL;   
ALTER TABLE account DROP COLUMN oidc_sub;
ALTER TABLE account ALTER COLUMN password_hash SET NOT NULL; 

