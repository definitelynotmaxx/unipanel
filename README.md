# UniPanel

A self-hosted control panel. One Rust binary, embedded React frontend, SQLite database.

## Quick start

    docker run -d -p 8080:8080 -v /var/lib/unipanel:/var/lib/unipanel unipanel:latest

Then open <http://localhost:8080> and follow the first-run setup (locale + admin account).

## Development

This repository contains both the Rust workspace and the embedded SPA.

- Rust toolchain: install via [rustup](https://rustup.rs). This project is tested on `x86_64-pc-windows-gnu` (MinGW) and Linux aarch64.
- Node: 20+ (tested with v22).

Build & run locally:

    cargo build -p unipanel-cli
    cd web && npm install && npm run build && cd ..
    ./target/debug/unipanel serve

Run tests:

    cargo test --workspace
    cd web && npm run test && npm run typecheck && npm run lint

## Configuration

All configuration is via environment variables prefixed `UNIPANEL_` (see `.env.example`).

## License

MIT — see [LICENSE](./LICENSE).