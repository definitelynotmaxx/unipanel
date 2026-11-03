-- M0 foundation schema.
--
-- Conventions:
--   * ids are UUIDv4 strings (text), timestamps are UNIX seconds (integer, UTC)
--   * booleans are integers constrained to 0/1
--   * anything user facing is a message key, never rendered prose

CREATE TABLE users (
    id            TEXT    PRIMARY KEY,
    username      TEXT    NOT NULL,
    display_name  TEXT    NOT NULL,
    password_hash TEXT,
    role          TEXT    NOT NULL DEFAULT 'admin',
    is_active     INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL
);

-- Usernames are compared case-insensitively; the application lowercases them.
CREATE UNIQUE INDEX users_username_unique ON users (username);

CREATE TABLE sessions (
    id           TEXT    PRIMARY KEY,
    user_id      TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash   BLOB    NOT NULL,
    csrf_token   TEXT    NOT NULL,
    created_at   INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    expires_at   INTEGER NOT NULL,
    ip           TEXT,
    user_agent   TEXT
);

CREATE UNIQUE INDEX sessions_token_hash_unique ON sessions (token_hash);
CREATE INDEX sessions_user_id ON sessions (user_id);
CREATE INDEX sessions_expires_at ON sessions (expires_at);

CREATE TABLE settings (
    key        TEXT    PRIMARY KEY,
    value      TEXT    NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Audit log. `event` is a message key such as `audit.auth.login`; `detail` is
-- JSON with message parameters and never contains secrets.
CREATE TABLE audit_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at    INTEGER NOT NULL,
    actor_user_id TEXT REFERENCES users (id) ON DELETE SET NULL,
    actor_label   TEXT,
    event         TEXT    NOT NULL,
    detail        TEXT,
    ip            TEXT,
    success       INTEGER NOT NULL CHECK (success IN (0, 1)),
    request_id    TEXT
);

CREATE INDEX audit_log_created_at ON audit_log (created_at);
CREATE INDEX audit_log_event ON audit_log (event);
CREATE INDEX audit_log_actor ON audit_log (actor_user_id);
