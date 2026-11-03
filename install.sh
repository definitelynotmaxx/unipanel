#!/usr/bin/env sh
set -e

# UniPanel one-command installer using Docker (builds from source since no public image yet).

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required but not installed: https://docs.docker.com/get-docker/"
  exit 1
fi

echo "Cloning UniPanel repository..."
rm -rf unipanel || true
git clone https://github.com/definitelynotmaxx/unipanel.git unipanel
cd unipanel

echo "Building Docker image locally (this may take several minutes)..."
docker compose build

echo
echo "Starting UniPanel..."
docker compose up -d

echo
echo "UniPanel is running at http://localhost:8080"
echo "Complete the first-run setup (locale + admin account) in your browser."