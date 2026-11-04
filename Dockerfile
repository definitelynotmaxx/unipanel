# Multi-stage, multi-arch build: amd64 + arm64.
# Optimize build cache: incremental cargo with cache mount to /usr/local/cargo/registry and /target

# ---------- frontend build ----------
FROM --platform=$BUILDPLATFORM node:22-alpine AS web
WORKDIR /web
COPY web/package.json web/package-lock.json* web/.npmrc* ./
RUN npm ci --no-audit --no-fund 2>/dev/null || npm install --no-audit --no-fund
COPY web ./
RUN npm run build

# ---------- rust build ----------
FROM rust:1.91-alpine AS rust
WORKDIR /src
RUN apk add --no-cache build-base musl-dev pkgconfig
# Separate dependency layer for cache efficiency
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY --from=web /web/dist ./web/dist
# Mount cache dirs, fetch first, then build
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo fetch
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build -p unipanel-cli --release
RUN mkdir -p /out && cp /src/target/release/unipanel /out/unipanel

# ---------- final image ----------
FROM alpine:3.20 AS runtime
WORKDIR /app
RUN apk add --no-cache ca-certificates tini wget
COPY --from=rust /out/unipanel /usr/local/bin/unipanel
ENV UNIPANEL_DATA_DIR=/var/lib/unipanel
ENV UNIPANEL_BIND=0.0.0.0:8080
RUN mkdir -p /var/lib/unipanel
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["wget", "-qO-", "http://127.0.0.1:8080/healthz"]
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["unipanel", "serve"]