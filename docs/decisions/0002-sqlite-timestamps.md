# ADR 0002 — SQLite timestamps stored as UNIX seconds

## Status
Accepted

## Context
SQLx runtime queries did not compile cleanly with SQLite's date/time format when
using macros. We want to avoid encoding ambiguity.

## Decision
All timestamps are stored as `INTEGER` columns holding UNIX epoch seconds. The storage
layer and `unipanel-storage` handle conversion to/from `time::OffsetDateTime`.

## Consequences
- No compile-time macros needed in SQL queries.
- Clear contract between storage and the rest of the codebase.
- Loses sub-second precision (acceptable for an admin panel).