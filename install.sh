#!/usr/bin/env sh
set -e

# Determines the best installation method for UniPanel
if command -v docker >/dev/null 2>&1; then
  echo "Docker detected — pulling UniPanel image and starting ... "
  docker pull ghcr.io/definitelynotmaxx/unipanel:latest
  docker run -d --name unipanel \
      -p 8080:8080 \
      -v unipanel_data:/var/lib/unipanel \
      --restart unless-stopped \
      ghcr.io/definitelynotmaxx/unipanel:latest
  echo
  echo "UniPanel is starting at http://localhost:8080"
  echo "Complete the first-run setup there (locale + admin account)."
else
  echo "Docker is not installed. Please install Docker first: https://docs.docker.com/get-docker/"
  echo "Or download a pre-built binary from the releases page."
  exit 1
fi