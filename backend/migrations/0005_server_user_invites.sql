CREATE TABLE IF NOT EXISTS server_user_invites (
  id TEXT PRIMARY KEY,
  server_id TEXT NOT NULL,
  from_user_id TEXT NOT NULL,
  to_user_id TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(server_id, to_user_id),
  FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
  FOREIGN KEY (from_user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (to_user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_server_user_invites_to_user ON server_user_invites(to_user_id);
CREATE INDEX IF NOT EXISTS idx_server_user_invites_server ON server_user_invites(server_id);
