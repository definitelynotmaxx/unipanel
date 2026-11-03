# Multi-stage, multi-arch build: amd64 + arm64.
# Build with:
#   docker buildx build --platform linux/amd64,linux/arm64 -t unipanel:latest .

# ---------- frontend build ----------
FROM --platform=$BUILDPLATFORM node:22-alpine AS web
WORKDIR /web
COPY web/package.json web/package-lock.json* web/.npmrc* ./
RUN npm ci --no-audit --no-fund
COPY web ./ 
RUN npm run build

# ---------- rust build ----------
FROM rust:1.80-alpine AS rust
WORKDIR /src
# install system deps (openssl/sqlite libs are vendored, only toolchain needed)
RUN apk add --no-cache build-base musl-dev
COPY . .
COPY --from=web /web/dist ./web/dist
# pin to workspace; cargo builds the CLI binary which embeds the SPA.
RUN cargo build -p unipanel-cli --release && cp target/release/unipanel /out/unipanel

# ---------- final image ----------
FROM alpine:3.20 AS runtime
WORKDIR /app
RUN apk add --no-cache ca-certificates tini curl
COPY --from=rust /out/unipanel /usr/local/bin/unipanel
ENV UNIPANEL_DATA_DIR=/var/lib/unipanel
ENV UNIPANEL_BIND=0.0.0.0:8080
RUN mkdir -p /var/lib/unipanel
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["wget", "-qO-", "http://127.0.0.1:8080/healthz"]
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["unipanel", "serve"]