-- Connector configurations: external panel back-ends such as Pterodactyl,
-- Docker, Proxmox or arbitrary REST APIs.  API keys are stored base64-encoded
-- and encrypted at rest by the crypto crate; the application never reads
-- plain-text keys from the database.

CREATE TABLE connectors (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    kind       TEXT NOT NULL CHECK(kind IN ('pterodactyl', 'docker', 'proxmox', 'rest')),
    api_url    TEXT NOT NULL,
    api_key    TEXT NOT NULL,
    is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_connectors_active ON connectors(is_active);
