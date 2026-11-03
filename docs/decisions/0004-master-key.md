# ADR 0004 — Master key location and auto-generation

## Status
Accepted

## Context
Connector secrets are encrypted at rest with ChaCha20-Poly1305. The master key cannot
live inside the database.

## Decision
Resolve the master key in this order:
1. `UNIPANEL_MASTER_KEY` — inline base64 value.
2. `UNIPANEL_MASTER_KEY_FILE` — path to a file storing base64.
3. `{data_dir}/master.key` — file generated on first run with mode 0600 on Unix.

## Consequences
- `docker run` works without extra configuration on first boot.
- Operators can mount or inject a persistent key; losing it means losing access to
  stored secrets, which matches operator-driven backup workflows.