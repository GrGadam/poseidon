ALTER TABLE users ADD COLUMN email TEXT;
UPDATE users SET email = username || '@poseidon.local' WHERE email IS NULL OR email = '';
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique ON users(email);
