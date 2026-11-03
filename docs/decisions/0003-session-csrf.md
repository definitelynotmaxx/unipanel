# ADR 0003 — Cookie-based sessions with double-submit CSRF

## Status
Accepted

## Context
The SPA is embedded in the binary so the API is same-origin. We still need to be
resilient to CSRF even with same-origin.

## Decision
- Authenticate with a random 32-byte token stored in the `unipanel_session` cookie;
  only the SHA-256 hash is persisted.
- A separate per-session `unipanel_csrf` cookie plus `X-CSRF-Token` header are
  compared in constant time for unsafe methods.
- Anonymous visitors receive a CSRF cookie via `/bootstrap`, enough for setup/login.

## Consequences
- HttpOnly + SameSite cookies keep the bearer token out of reach from JavaScript.
- No separate token in localStorage; the session cookie is the sole credential.
- Browser refreshes don't invalidate the session.