#!/usr/bin/env sh
set -e

IMAGE="ghcr.io/definitelynotmaxx/unipanel:latest"

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required: https://docs.docker.com/get-docker/"
  exit 1
fi

echo "Pulling UniPanel image..."
docker pull "$IMAGE"

echo "Starting UniPanel..."
docker rm -f unipanel 2>/dev/null || true
docker run -d --name unipanel \
  -p 8080:8080 \
  -v unipanel_data:/var/lib/unipanel \
  --restart unless-stopped \
  "$IMAGE"

echo
echo "UniPanel running at http://localhost:8080"
echo "Complete the first-run setup: locale + admin account."