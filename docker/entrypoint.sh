#!/bin/sh
set -e

MODELS_DIR="${MODELS_DIR:-/app/models}"
RUNTIMES_DIR="${RUNTIMES_DIR:-/app/runtimes}"
DATA_DIR="${DATA_DIR:-/app/data}"

mkdir -p "${MODELS_DIR}" "${RUNTIMES_DIR}" "${DATA_DIR}"

exec "$@"

