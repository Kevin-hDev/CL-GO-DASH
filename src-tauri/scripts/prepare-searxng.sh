#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCHIVE="$ROOT/resources/searxng-sidecar/source.tar.gz"
SOURCE="$ROOT/resources/searxng-sidecar/source"
SOURCE_TMP=""
WHEELS="$ROOT/resources/searxng-sidecar/wheels"
TMP="$WHEELS.tmp"
STAMP="$WHEELS/.requirements.sha256"
LIMIT_BYTES=$((150 * 1024 * 1024))

cleanup() {
  if [[ -n "$SOURCE_TMP" ]]; then
    rm -r "$SOURCE_TMP"
  fi
}
trap cleanup EXIT

if [[ ! -d "$SOURCE" && -f "$ARCHIVE" ]]; then
  SOURCE_TMP="$(mktemp -d)"
  tar -xzf "$ARCHIVE" -C "$SOURCE_TMP"
  SOURCE="$SOURCE_TMP/source"
fi

if [[ ! -f "$SOURCE/requirements.txt" || ! -f "$SOURCE/setup.py" ]]; then
  echo "SearXNG bundle incomplet" >&2
  exit 1
fi

if find "$SOURCE" \( -name '._*' -o -name '.DS_Store' -o -name '.py' \) -print -quit | grep -q .; then
  echo "SearXNG bundle pollue par des metadonnees macOS" >&2
  exit 1
fi

HASH="$(shasum -a 256 "$SOURCE/requirements.txt" "$SOURCE/setup.py" | shasum -a 256 | cut -d ' ' -f 1)"
if [[ -f "$STAMP" && "$(cat "$STAMP")" == "$HASH" ]] && compgen -G "$WHEELS/*.whl" >/dev/null; then
  exit 0
fi

if command -v uv >/dev/null 2>&1; then
  PYTHON=(uv run python3)
else
  PYTHON=(python3)
fi

rm -r "$TMP" 2>/dev/null || true
mkdir -p "$TMP"
"${PYTHON[@]}" -m pip download \
  --only-binary=:all: \
  --dest "$TMP" \
  -r "$SOURCE/requirements.txt"
"${PYTHON[@]}" -m pip download \
  --only-binary=:all: \
  --dest "$TMP" \
  setuptools wheel

SIZE="$(tar -czf - -C "$TMP" . | wc -c | tr -d ' ')"
rm -r "$WHEELS" 2>/dev/null || true
mkdir -p "$WHEELS"

if [[ "$SIZE" -gt "$LIMIT_BYTES" ]]; then
  echo "download-at-first-launch" > "$WHEELS/.mode"
  rm -r "$TMP" 2>/dev/null || true
  exit 0
fi

mv "$TMP"/*.whl "$WHEELS"/
echo "$HASH" > "$STAMP"
rm -r "$TMP" 2>/dev/null || true
