# Architecture Decision Records

Short notes recording decisions made while building UniPanel. Append-only; new decisions
start here and reference this directory from pull requests.

1. [0001-messages.md](./0001-messages.md) — Backend stores message keys + params, never rendered prose
2. [0002-sqlite-timestamps.md](./0002-sqlite-timestamps.md) — UNIX seconds in SQLite columns
3. [0003-session-csrf.md](./0003-session-csrf.md) — Cookie-based sessions with double-submit CSRF
4. [0004-master-key.md](./0004-master-key.md) — Master key location and auto-generation
5. [0005-codegen.md](./0005-codegen.md) — OpenAPI → TypeScript codegen