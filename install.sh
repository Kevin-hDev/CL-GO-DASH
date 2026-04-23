#!/bin/bash
set -uo pipefail

REPO="Kevin-hDev/CL-GO-DASH"
APP_NAME="CL-GO"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

info()  { printf "\033[1;34m→\033[0m %s\n" "$1"; }
ok()    { printf "\033[1;32m✓\033[0m %s\n" "$1"; }
fail()  { printf "\033[1;31m✗\033[0m %s\n" "$1" >&2; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"

info "Détection : ${OS} ${ARCH}"

case "${OS}" in
  Darwin) PLATFORM="macos"; EXT=".dmg" ;;
  Linux)  PLATFORM="linux"; EXT=".AppImage" ;;
  *)      fail "OS non supporté : ${OS}. Sur Windows, télécharge le .msi depuis GitHub." ;;
esac

if ! command -v curl &>/dev/null; then
  fail "curl est requis. Installe-le et relance."
fi

info "Récupération de la dernière version..."
RELEASE_JSON=$(curl -fsSL -H "User-Agent: CL-GO-Installer" "${API_URL}" 2>/dev/null) || fail "Impossible de contacter GitHub."

VERSION=$(echo "${RELEASE_JSON}" | grep -o '"tag_name" *: *"[^"]*"' | head -1 | sed 's/.*": *"//;s/"//' | sed 's/^v//') || true
if [ -z "${VERSION}" ]; then
  fail "Aucune release trouvée."
fi

info "Version disponible : v${VERSION}"

ASSET_URL=$(echo "${RELEASE_JSON}" | tr ',' '\n' | grep "browser_download_url" | grep "${EXT}" | head -1 | sed 's/.*"browser_download_url" *: *"//;s/".*//') || true
if [ -z "${ASSET_URL}" ]; then
  fail "Pas de fichier ${EXT} dans la release v${VERSION}."
fi

info "Téléchargement de ${APP_NAME} v${VERSION}..."
TMP_DIR=$(mktemp -d)
TMP_FILE="${TMP_DIR}/${APP_NAME}-update${EXT}"
curl -fSL --progress-bar -o "${TMP_FILE}" "${ASSET_URL}" || fail "Échec du téléchargement."

if [ "${PLATFORM}" = "macos" ]; then
  info "Installation dans /Applications..."
  VOL=$(hdiutil attach "${TMP_FILE}" -nobrowse -noverify 2>/dev/null | grep "/Volumes/" | sed 's/.*\(\/Volumes\/[^ ]*\).*/\1/' | head -1) || true
  if [ -z "${VOL}" ]; then
    fail "Impossible de monter le DMG."
  fi
  if [ -d "${VOL}/${APP_NAME}.app" ]; then
    rm -rf "/Applications/${APP_NAME}.app"
    cp -Rf "${VOL}/${APP_NAME}.app" "/Applications/${APP_NAME}.app"
  else
    hdiutil detach "${VOL}" -quiet 2>/dev/null || true
    fail "${APP_NAME}.app introuvable dans le DMG."
  fi
  hdiutil detach "${VOL}" -quiet 2>/dev/null || true
  rm -rf "${TMP_DIR}"
  ok "${APP_NAME} v${VERSION} installé dans /Applications/"
  info "Lancement..."
  open "/Applications/${APP_NAME}.app"

elif [ "${PLATFORM}" = "linux" ]; then
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "${INSTALL_DIR}"
  DEST="${INSTALL_DIR}/${APP_NAME}.AppImage"
  mv "${TMP_FILE}" "${DEST}"
  chmod +x "${DEST}"
  rm -rf "${TMP_DIR}"
  ok "${APP_NAME} v${VERSION} installé dans ${DEST}"
  info "Lancement..."
  "${DEST}" &
fi

ok "Installation terminée."
