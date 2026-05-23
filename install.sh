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
  *)      fail "OS non supporté : ${OS}. Sur Windows, lance install.ps1 ou télécharge le fichier -setup.exe depuis GitHub." ;;
esac

command -v curl &>/dev/null || fail "curl est requis. Installe-le et relance."

# --- Dépendances Linux ---
if [ "${PLATFORM}" = "linux" ]; then
  info "Vérification des dépendances..."
  MISSING=""
  if command -v dpkg &>/dev/null; then
    dpkg -s libwebkit2gtk-4.1-0 &>/dev/null || MISSING="${MISSING} libwebkit2gtk-4.1-0"
    dpkg -s libgtk-3-0 &>/dev/null || MISSING="${MISSING} libgtk-3-0"
  elif command -v rpm &>/dev/null; then
    rpm -q webkit2gtk4.1 &>/dev/null || MISSING="${MISSING} webkit2gtk4.1"
    rpm -q gtk3 &>/dev/null || MISSING="${MISSING} gtk3"
  fi
  if [ -n "${MISSING}" ]; then
    info "Installation des dépendances :${MISSING}"
    if command -v apt-get &>/dev/null; then
      sudo apt-get install -y ${MISSING} || fail "Échec de l'installation des dépendances. Lance : sudo apt-get install${MISSING}"
    elif command -v dnf &>/dev/null; then
      sudo dnf install -y ${MISSING} || fail "Échec de l'installation des dépendances."
    else
      fail "Paquets manquants :${MISSING}. Installe-les manuellement et relance."
    fi
    ok "Dépendances installées."
  else
    ok "Dépendances OK."
  fi
fi

# --- Récupération de la dernière version ---
info "Récupération de la dernière version..."
RELEASE_JSON=$(curl -fsSL -H "User-Agent: CL-GO-Installer" "${API_URL}" 2>/dev/null) || fail "Impossible de contacter GitHub."

VERSION=$(printf '%s' "${RELEASE_JSON}" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' | head -1)
[ -z "${VERSION}" ] && fail "Aucune release trouvée."

ASSET_URL=$(printf '%s' "${RELEASE_JSON}" | sed -n 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*'"${EXT}"'\)".*/\1/p' | head -1)
[ -z "${ASSET_URL}" ] && fail "Pas de fichier ${EXT} dans la release v${VERSION}."

# --- Téléchargement ---
info "Téléchargement de ${APP_NAME} v${VERSION}..."
TMP_DIR=$(mktemp -d)
TMP_FILE="${TMP_DIR}/${APP_NAME}-update${EXT}"
curl -fSL --progress-bar -o "${TMP_FILE}" "${ASSET_URL}" || fail "Échec du téléchargement."

# --- Choix du répertoire d'installation ---
if [ "${PLATFORM}" = "macos" ]; then
  DEFAULT_DIR="/Applications"
else
  DEFAULT_DIR="${HOME}/.local/bin"
fi

printf "\n\033[1;33m📁 Répertoire d'installation : %s\033[0m\n" "${DEFAULT_DIR}"
printf "   Appuie sur Entrée pour accepter, ou tape un autre chemin : "
read -r CUSTOM_DIR < /dev/tty 2>/dev/null || CUSTOM_DIR=""
if [ -n "${CUSTOM_DIR}" ]; then
  INSTALL_DIR="${CUSTOM_DIR}"
else
  INSTALL_DIR="${DEFAULT_DIR}"
fi

case "${INSTALL_DIR}" in
  ~/*) INSTALL_DIR="${HOME}/${INSTALL_DIR#\~/}" ;;
  ~)   INSTALL_DIR="${HOME}" ;;
esac

mkdir -p "${INSTALL_DIR}" 2>/dev/null || fail "Impossible de créer ${INSTALL_DIR}. Vérifie les permissions."

# --- Installation ---
if [ "${PLATFORM}" = "macos" ]; then
  info "Installation dans ${INSTALL_DIR}..."
  VOL=$(hdiutil attach "${TMP_FILE}" -nobrowse -noverify 2>/dev/null | grep "/Volumes/" | sed 's/.*\(\/Volumes\/[^ ]*\).*/\1/' | head -1)
  [ -z "${VOL}" ] && fail "Impossible de monter le DMG."

  if [ -d "${VOL}/${APP_NAME}.app" ]; then
    rm -rf "${INSTALL_DIR}/${APP_NAME}.app"
    cp -Rf "${VOL}/${APP_NAME}.app" "${INSTALL_DIR}/${APP_NAME}.app"
  else
    hdiutil detach "${VOL}" -quiet 2>/dev/null
    fail "${APP_NAME}.app introuvable dans le DMG."
  fi
  hdiutil detach "${VOL}" -quiet 2>/dev/null
  rm -rf "${TMP_DIR}"
  ok "${APP_NAME} v${VERSION} installé dans ${INSTALL_DIR}/"
  info "Lancement..."
  open "${INSTALL_DIR}/${APP_NAME}.app"

elif [ "${PLATFORM}" = "linux" ]; then
  DEST="${INSTALL_DIR}/${APP_NAME}.AppImage"
  mv "${TMP_FILE}" "${DEST}"
  chmod +x "${DEST}"
  rm -rf "${TMP_DIR}"
  ok "${APP_NAME} v${VERSION} installé dans ${DEST}"
  info "Lancement..."
  "${DEST}" &
fi

ok "Installation terminée."
