# UniPanel

Self-hosted control panel. One Rust binary, embedded React frontend, SQLite database, MIT license.

## Quick install (recommended)

### With Docker (one line):

```sh
curl -fsSL https://raw.githubusercontent.com/definitelynotmaxx/unipanel/main/install.sh | sh
```

### With Docker Compose:

```sh
# Download the file
curl -fsSL https://raw.githubusercontent.com/definitelynotmaxx/unipanel/main/docker-compose.yml -o docker-compose.yml
# Start
docker compose up -d
# Open browser
# https://localhost:8080  (or :8080)
```

On first launch you'll be prompted for:
1. **Instance language** (English / Deutsch)
2. **Admin account** (username + password + display name + optional instance name)

After setup you land at the login screen.

## Container configuration

Set any of these `ENV` vars (prefix `UNIPANEL_`):

| Var | Default |
|-----|---------|
| `BIND` | `0.0.0.0:8080` |
| `PUBLIC_URL` | (auto) |
| `DATA_DIR` | `/var/lib/unipanel` |
| `TRUST_PROXY` | `false` |
| `MASTER_KEY` | auto-generated |
| `LOG` | `info` |

## Development

- Rust 1.80+ via [rustup](https://rustup.rs)
- Node 22+

```sh
cargo build -p unipanel-cli
cd web && npm ci && npm run build && cd ..
./target/debug/unipanel serve
```

Tests:
```sh
cargo test --workspace
cd web && npm test && npm run typecheck && npm run lint
```

## License

MIT — see `LICENSE`.