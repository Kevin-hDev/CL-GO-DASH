#!/bin/bash
set -euo pipefail

OLLAMA_VERSION="${OLLAMA_VERSION:-0.21.1}"
OS="${1:-$(uname -s | tr '[:upper:]' '[:lower:]')}"
ARCH="${2:-$(uname -m)}"

case "${ARCH}" in
  x86_64|amd64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
esac

DEST="resources/ollama-bundle"
BASE_URL="https://github.com/ollama/ollama/releases/download/v${OLLAMA_VERSION}"
SHA_URL="${BASE_URL}/sha256sum.txt"

# Nettoyage complet avant extraction
rm -rf "${DEST}"
mkdir -p "${DEST}"

case "${OS}" in
  darwin)     ARCHIVE="ollama-darwin.tgz" ;;
  linux)      ARCHIVE="ollama-linux-${ARCH}.tar.zst" ;;
  windows|mingw*|msys*|cygwin*) ARCHIVE="ollama-windows-${ARCH}.zip" ;;
  *)          echo "✗ OS non supporté : ${OS}" >&2; exit 1 ;;
esac

echo "→ Téléchargement Ollama v${OLLAMA_VERSION} (${OS}/${ARCH})..."
curl -fSL --progress-bar -o "/tmp/${ARCHIVE}" "${BASE_URL}/${ARCHIVE}"

# Vérification SHA256
echo "→ Vérification intégrité..."
EXPECTED=$(curl -fsSL "${SHA_URL}" | grep "${ARCHIVE}" | awk '{print $1}')
if [ -n "${EXPECTED}" ]; then
  if command -v sha256sum &>/dev/null; then
    ACTUAL=$(sha256sum "/tmp/${ARCHIVE}" | awk '{print $1}')
  else
    ACTUAL=$(shasum -a 256 "/tmp/${ARCHIVE}" | awk '{print $1}')
  fi
  if [ "${ACTUAL}" != "${EXPECTED}" ]; then
    echo "✗ SHA256 invalide — fichier corrompu ou altéré" >&2
    rm -f "/tmp/${ARCHIVE}"
    exit 1
  fi
  echo "✓ SHA256 OK"
fi

# Extraction
case "${OS}" in
  darwin)
    tar -xzf "/tmp/${ARCHIVE}" -C "${DEST}" --strip-components=1
    chmod +x "${DEST}/ollama"
    ;;
  linux)
    if command -v zstd &>/dev/null; then
      zstd -d "/tmp/${ARCHIVE}" --stdout | tar -x -C "${DEST}" --strip-components=1
    else
      tar --zstd -xf "/tmp/${ARCHIVE}" -C "${DEST}" --strip-components=1
    fi
    chmod +x "${DEST}/ollama"
    ;;
  windows|mingw*|msys*|cygwin*)
    unzip -o -q "/tmp/${ARCHIVE}" -d "${DEST}"
    ;;
esac

rm -f "/tmp/${ARCHIVE}"
echo "✓ Ollama v${OLLAMA_VERSION} (${OS}/${ARCH}) → ${DEST}/"
