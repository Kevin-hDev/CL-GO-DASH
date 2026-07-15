#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

EXPECTED_ARCHIVE="cef_binary_150.0.10+g8042e43+chromium-150.0.7871.101_macosarm64_minimal.tar.bz2"
EXPECTED_SHA1="e73f7ce767420791b1965e15816a955d88cf1f9a"
HELPERS=(
  "CL-GO Helper"
  "CL-GO Helper (GPU)"
  "CL-GO Helper (Renderer)"
  "CL-GO Helper (Plugin)"
  "CL-GO Helper (Alerts)"
)
SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:--}"
if [[ -z "$SIGNING_IDENTITY" || ${#SIGNING_IDENTITY} -gt 256 \
  || "$SIGNING_IDENTITY" == *$'\n'* || "$SIGNING_IDENTITY" == *$'\r'* \
  || "$SIGNING_IDENTITY" == *$'\t'* ]]; then
  echo "CEF signing identity is invalid" >&2
  exit 1
fi

"${CARGO:-cargo}" build --release --bin cl-go-dash-helper

CEF_DIR=""
CEF_ARCHIVE=""
CANDIDATES=(target/release/build/cef-dll-sys-*/out/cef_macos_aarch64)
if (( ${#CANDIDATES[@]} > 16 )); then
  echo "CEF runtime preparation failed" >&2
  exit 1
fi
for candidate in "${CANDIDATES[@]}"; do
  framework="$candidate/Chromium Embedded Framework.framework"
  archive="$(dirname "$candidate")/$EXPECTED_ARCHIVE"
  if [[ ! -d "$framework" || ! -f "$archive" ]]; then
    continue
  fi
  actual_sha1="$(shasum -a 1 "$archive" | awk '{print $1}')"
  if [[ "$actual_sha1" == "$EXPECTED_SHA1" ]]; then
    CEF_DIR="$candidate"
    CEF_ARCHIVE="$archive"
    break
  fi
done
if [[ -z "$CEF_DIR" ]]; then
  echo "CEF runtime verification failed" >&2
  exit 1
fi

STAGE="target/cef-runtime/macos"
rm -rf -- "$STAGE"
mkdir -p "$STAGE"
ditto "$CEF_DIR/Chromium Embedded Framework.framework" \
  "$STAGE/Chromium Embedded Framework.framework"
ditto "resources/cef/macos/helpers" "$STAGE/helpers"
LICENSE_MEMBER="${EXPECTED_ARCHIVE%.tar.bz2}/LICENSE.txt"
tar -xOf "$CEF_ARCHIVE" "$LICENSE_MEMBER" > "$STAGE/LICENSE.txt"
if [[ ! -s "$STAGE/LICENSE.txt" ]] || (( $(wc -c < "$STAGE/LICENSE.txt") > 2097152 )); then
  echo "CEF license verification failed" >&2
  exit 1
fi

for helper in "${HELPERS[@]}"; do
  app="$STAGE/helpers/$helper.app"
  destination="$STAGE/helpers/$helper.app/Contents/MacOS/$helper"
  mkdir -p "$(dirname "$destination")"
  install -m 755 "target/release/cl-go-dash-helper" "$destination"
  codesign --force --sign "$SIGNING_IDENTITY" "$destination"
  codesign --force --sign "$SIGNING_IDENTITY" "$app"
done
