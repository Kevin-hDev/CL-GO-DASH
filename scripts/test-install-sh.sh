#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CL_GO_INSTALLER_TEST_MODE=1

# shellcheck source=../install.sh
. "${ROOT_DIR}/install.sh"

assert_eq() {
  local expected="$1"
  local actual="$2"
  local label="$3"

  if [ "${actual}" != "${expected}" ]; then
    printf "FAIL %s: expected [%s], got [%s]\n" "${label}" "${expected}" "${actual}" >&2
    exit 1
  fi
}

assert_eq ".dmg" "$(asset_extension_for_platform macos)" "macOS asset"
assert_eq ".deb" "$(asset_extension_for_platform linux)" "Linux asset"

linux_arch_supported x86_64
linux_arch_supported amd64
if linux_arch_supported arm64; then
  printf "FAIL Linux ARM should not be accepted by install.sh\n" >&2
  exit 1
fi

printf "install.sh tests OK\n"
