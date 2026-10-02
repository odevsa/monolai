#!/bin/sh
set -e

# Ensure directories exist
mkdir -p "${DATA_DIR:-/app/data}" "${NOTES_DIR:-/app/notes}"

# Execute command
exec "$@"

