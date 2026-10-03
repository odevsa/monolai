#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

SOURCE_SVG="${ROOT_DIR}/frontend/src/lib/assets/icon-dark.svg"

# Target OS-specific asset directories inside packaging/
LINUX_ASSETS="${ROOT_DIR}/packaging/linux/assets"
MACOS_ASSETS="${ROOT_DIR}/packaging/macos/assets"
WINDOWS_ASSETS="${ROOT_DIR}/packaging/windows/assets"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "${TMP_DIR}"' EXIT

if [ ! -f "${SOURCE_SVG}" ]; then
  echo "Error: Source SVG not found at ${SOURCE_SVG}" >&2
  exit 1
fi

echo "==> Rendering icon resolutions from ${SOURCE_SVG}..."
SIZES=(16 24 32 48 64 128 256 512 1024)

for size in "${SIZES[@]}"; do
  out_png="${TMP_DIR}/icon-${size}.png"
  if command -v magick &>/dev/null; then
    magick -background none "${SOURCE_SVG}" -resize "${size}x${size}" "${out_png}"
  elif command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w "${size}" -h "${size}" -a -f png -o "${out_png}" "${SOURCE_SVG}"
  elif command -v convert &>/dev/null; then
    convert -background none "${SOURCE_SVG}" -resize "${size}x${size}" "${out_png}"
  elif command -v inkscape &>/dev/null; then
    inkscape -w "${size}" -h "${size}" -o "${out_png}" "${SOURCE_SVG}"
  else
    echo "Error: No SVG rasterizer found (magick, convert, rsvg-convert, or inkscape required)." >&2
    exit 1
  fi
done

# =============================================================================
# 1. Linux Assets (packaging/linux/assets)
# =============================================================================
echo "==> Generating Linux assets in packaging/linux/assets/..."
HICOLOR_DIR="${LINUX_ASSETS}/icons/hicolor"
mkdir -p "${HICOLOR_DIR}"

for size in 16 24 32 48 64 128 256 512; do
  hicolor_target="${HICOLOR_DIR}/${size}x${size}/apps"
  mkdir -p "${hicolor_target}"
  cp "${TMP_DIR}/icon-${size}.png" "${hicolor_target}/monolai.png"
done

mkdir -p "${HICOLOR_DIR}/scalable/apps"
cp "${SOURCE_SVG}" "${HICOLOR_DIR}/scalable/apps/monolai.svg"
cp "${TMP_DIR}/icon-256.png" "${LINUX_ASSETS}/monolai.png"

# =============================================================================
# 2. macOS Assets (packaging/macos/assets)
# =============================================================================
echo "==> Generating macOS assets in packaging/macos/assets/..."
mkdir -p "${MACOS_ASSETS}"
python3 "${SCRIPT_DIR}/make_icns.py" "${MACOS_ASSETS}/monolai.icns" \
  "${TMP_DIR}/icon-16.png" \
  "${TMP_DIR}/icon-32.png" \
  "${TMP_DIR}/icon-64.png" \
  "${TMP_DIR}/icon-128.png" \
  "${TMP_DIR}/icon-256.png" \
  "${TMP_DIR}/icon-512.png" \
  "${TMP_DIR}/icon-1024.png"

# =============================================================================
# 3. Windows Assets (packaging/windows/assets)
# =============================================================================
echo "==> Generating Windows assets in packaging/windows/assets/..."
mkdir -p "${WINDOWS_ASSETS}"
if command -v magick &>/dev/null; then
  magick "${TMP_DIR}/icon-16.png" \
         "${TMP_DIR}/icon-24.png" \
         "${TMP_DIR}/icon-32.png" \
         "${TMP_DIR}/icon-48.png" \
         "${TMP_DIR}/icon-64.png" \
         "${TMP_DIR}/icon-128.png" \
         "${TMP_DIR}/icon-256.png" \
         "${WINDOWS_ASSETS}/monolai.ico"
elif command -v convert &>/dev/null; then
  convert "${TMP_DIR}/icon-16.png" \
          "${TMP_DIR}/icon-24.png" \
          "${TMP_DIR}/icon-32.png" \
          "${TMP_DIR}/icon-48.png" \
          "${TMP_DIR}/icon-64.png" \
          "${TMP_DIR}/icon-128.png" \
          "${TMP_DIR}/icon-256.png" \
          "${WINDOWS_ASSETS}/monolai.ico"
fi

if [ -f "${WINDOWS_ASSETS}/monolai.ico" ]; then
  cp "${WINDOWS_ASSETS}/monolai.ico" "${ROOT_DIR}/desktop/assets/monolai.ico"
fi

echo "==> All OS-specific assets successfully generated in packaging/<os>/assets/!"
