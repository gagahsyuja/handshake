-- Drop indexes
DROP INDEX IF EXISTS idx_email_verifications_code;
DROP INDEX IF EXISTS idx_email_verifications_user_id;
DROP INDEX IF EXISTS idx_users_email;

-- Drop tables
DROP TABLE IF EXISTS email_verifications;
DROP TABLE IF EXISTS users;
