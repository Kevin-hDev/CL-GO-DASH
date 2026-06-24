#!/bin/bash
set -uo pipefail

REPO="Kevin-hDev/CL-GO-DASH"
APP_NAME="CL-GO"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

info()  { printf "\033[1;34m→\033[0m %s\n" "$1"; }
ok()    { printf "\033[1;32m✓\033[0m %s\n" "$1"; }
fail()  { printf "\033[1;31m✗\033[0m %s\n" "$1" >&2; exit 1; }

asset_extension_for_platform() {
  case "$1" in
    macos) printf ".dmg" ;;
    linux) printf ".deb" ;;
    *) return 1 ;;
  esac
}

linux_arch_supported() {
  case "$1" in
    x86_64|amd64) return 0 ;;
    *) return 1 ;;
  esac
}

run_as_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  else
    command -v sudo &>/dev/null || fail "sudo est requis pour installer le paquet .deb."
    sudo "$@"
  fi
}

install_macos() {
  local tmp_file="$1"
  local default_dir="/Applications"
  local custom_dir=""
  local install_dir=""

  printf "\n\033[1;33m📁 Répertoire d'installation : %s\033[0m\n" "${default_dir}"
  printf "   Appuie sur Entrée pour accepter, ou tape un autre chemin : "
  read -r custom_dir < /dev/tty 2>/dev/null || custom_dir=""
  if [ -n "${custom_dir}" ]; then
    install_dir="${custom_dir}"
  else
    install_dir="${default_dir}"
  fi

  case "${install_dir}" in
    ~/*) install_dir="${HOME}/${install_dir#\~/}" ;;
    ~)   install_dir="${HOME}" ;;
  esac

  mkdir -p "${install_dir}" 2>/dev/null || fail "Impossible de créer ${install_dir}. Vérifie les permissions."
  info "Installation dans ${install_dir}..."
  VOL=$(hdiutil attach "${tmp_file}" -nobrowse -noverify 2>/dev/null | grep "/Volumes/" | sed 's/.*\(\/Volumes\/[^ ]*\).*/\1/' | head -1)
  [ -z "${VOL}" ] && fail "Impossible de monter le DMG."

  if [ -d "${VOL}/${APP_NAME}.app" ]; then
    rm -rf "${install_dir}/${APP_NAME}.app"
    cp -Rf "${VOL}/${APP_NAME}.app" "${install_dir}/${APP_NAME}.app"
  else
    hdiutil detach "${VOL}" -quiet 2>/dev/null
    fail "${APP_NAME}.app introuvable dans le DMG."
  fi
  hdiutil detach "${VOL}" -quiet 2>/dev/null
  ok "${APP_NAME} v${VERSION} installé dans ${install_dir}/"
  info "Lancement..."
  open "${install_dir}/${APP_NAME}.app"
}

install_linux_deb() {
  local tmp_file="$1"
  info "Installation du paquet .deb..."
  run_as_root apt-get install -y "${tmp_file}" || fail "Échec de l'installation du paquet .deb."
  ok "${APP_NAME} v${VERSION} installé via le paquet .deb"
  info "Lancement..."
  cl-go-dash >/dev/null 2>&1 &
}

main() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  info "Détection : ${OS} ${ARCH}"

  case "${OS}" in
    Darwin) PLATFORM="macos" ;;
    Linux)  PLATFORM="linux" ;;
    *)      fail "OS non supporté : ${OS}. Sur Windows, lance install.ps1 ou télécharge le fichier -setup.exe depuis GitHub." ;;
  esac

  EXT="$(asset_extension_for_platform "${PLATFORM}")"
  command -v curl &>/dev/null || fail "curl est requis. Installe-le et relance."

  if [ "${PLATFORM}" = "linux" ]; then
    linux_arch_supported "${ARCH}" || fail "Linux ARM n'est pas supporté par ce script. Télécharge un paquet compatible depuis GitHub."
    command -v apt-get &>/dev/null || fail "Ce script Linux installe le paquet .deb et nécessite apt-get."
  fi

  info "Récupération de la dernière version..."
  RELEASE_JSON=$(curl -fsSL -H "User-Agent: CL-GO-Installer" "${API_URL}" 2>/dev/null) || fail "Impossible de contacter GitHub."

  VERSION=$(printf '%s' "${RELEASE_JSON}" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' | head -1)
  [ -z "${VERSION}" ] && fail "Aucune release trouvée."

  ASSET_URL=$(printf '%s' "${RELEASE_JSON}" | sed -n 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*'"${EXT}"'\)".*/\1/p' | head -1)
  [ -z "${ASSET_URL}" ] && fail "Pas de fichier ${EXT} dans la release v${VERSION}."

  info "Téléchargement de ${APP_NAME} v${VERSION}..."
  TMP_DIR=$(mktemp -d)
  TMP_FILE="${TMP_DIR}/${APP_NAME}-update${EXT}"
  curl -fSL --progress-bar -o "${TMP_FILE}" "${ASSET_URL}" || fail "Échec du téléchargement."

  if [ "${PLATFORM}" = "macos" ]; then
    install_macos "${TMP_FILE}"
  else
    install_linux_deb "${TMP_FILE}"
  fi

  rm -rf "${TMP_DIR}"
  ok "Installation terminée."
}

if [ "${CL_GO_INSTALLER_TEST_MODE:-0}" != "1" ]; then
  main "$@"
fi
