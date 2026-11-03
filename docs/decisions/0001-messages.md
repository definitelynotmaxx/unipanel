# ADR 0001 — Backend stores message keys, not prose

## Status
Accepted

## Context
We display user-facing text in both English and German. Different locales may appear in
the same deployment. Console output is often shown in languages we don't speak.

## Decision
Every user-facing message originates as an `{ error: { key, params } }` (or `message`
on action results) structure. The backend never renders prose. The frontend resolves
keys into rendered strings via translation files.

## Consequences
- All user-facing text is internationalizable by construction.
- Searching logs by error key is stable across locales.
- New translation strings are a pure frontend change.