CREATE TABLE IF NOT EXISTS matches (
  id INTEGER PRIMARY KEY,
  match_id TEXT NOT NULL UNIQUE,
  patch_major INTEGER NOT NULL,
  patch_minor INTEGER NOT NULL,
  tier TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS matches_by_patch ON matches (patch_major, patch_minor);

CREATE TABLE IF NOT EXISTS participants (
  match INTEGER NOT NULL REFERENCES matches (id) ON DELETE CASCADE,
  participant INTEGER NOT NULL,
  champion TEXT NOT NULL,
  role TEXT NOT NULL,
  PRIMARY KEY (match, participant)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS participants_by_champion ON participants (champion, role);

CREATE TABLE IF NOT EXISTS minute_values (
  match INTEGER NOT NULL REFERENCES matches (id) ON DELETE CASCADE,
  participant INTEGER NOT NULL,
  at_ms INTEGER NOT NULL,
  cs INTEGER NOT NULL,
  total_gold INTEGER NOT NULL,
  level INTEGER NOT NULL,
  PRIMARY KEY (match, participant, at_ms)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS first_kills (
  match INTEGER NOT NULL REFERENCES matches (id) ON DELETE CASCADE,
  objective TEXT NOT NULL,
  at_ms INTEGER NOT NULL,
  PRIMARY KEY (match, objective)
) WITHOUT ROWID;

-- zstd JSON as Match-V5 answered it, less player identity and per-frame stats.
CREATE TABLE IF NOT EXISTS match_payloads (
  match INTEGER PRIMARY KEY REFERENCES matches (id) ON DELETE CASCADE,
  game BLOB NOT NULL,
  timeline BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS pending_matches (
  id INTEGER PRIMARY KEY,
  match_id TEXT NOT NULL UNIQUE,
  tier TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dropped_matches (
  match_id TEXT PRIMARY KEY,
  at_ms INTEGER NOT NULL
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS api_calls (
  at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collector_state (
  key TEXT PRIMARY KEY,
  value
) WITHOUT ROWID;
