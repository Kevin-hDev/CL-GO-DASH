#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" || $# -lt 1 || $# -gt 65 ]]; then
  echo "CEF development launch failed" >&2
  exit 1
fi

BINARY_INPUT="$1"
shift
if [[ -z "$BINARY_INPUT" || ${#BINARY_INPUT} -gt 4096 \
  || "$BINARY_INPUT" == *$'\n'* || "$BINARY_INPUT" == *$'\r'* \
  || "$BINARY_INPUT" == *$'\t'* ]]; then
  echo "CEF development launch failed" >&2
  exit 1
fi

DEBUG_ROOT="$(cd target/debug && pwd -P)"
BINARY="$(cd "$(dirname "$BINARY_INPUT")" && pwd -P)/$(basename "$BINARY_INPUT")"
if [[ ! -f "$BINARY" || ! -x "$BINARY" || "$BINARY" != "$DEBUG_ROOT"/* ]]; then
  echo "CEF development launch failed" >&2
  exit 1
fi
if [[ "$(basename "$BINARY")" != "cl-go-dash" ]]; then
  exec "$BINARY" "$@"
fi

bash scripts/ensure-cef-dev-runtime.sh

RUNTIME="target/cef-runtime/macos"
FRAMEWORK_SOURCE="$RUNTIME/Chromium Embedded Framework.framework"
HELPERS_SOURCE="$RUNTIME/helpers"
PLIST_SOURCE="resources/cef/macos/dev-app/Info.plist"
APP_MACOS="target/cef-dev/CL-GO Dev.app/Contents/MacOS"
APP_ROOT="$(dirname "$(dirname "$APP_MACOS")")"
APP_FRAMEWORKS="$APP_ROOT/Contents/Frameworks"
APP_EXECUTABLE="$APP_MACOS/cl-go-dash"
if [[ ! -d "$FRAMEWORK_SOURCE" || ! -d "$HELPERS_SOURCE" || ! -f "$PLIST_SOURCE" ]]; then
  echo "CEF development launch failed" >&2
  exit 1
fi

mkdir -p "$APP_MACOS" "$APP_FRAMEWORKS" "$APP_ROOT/Contents/Resources"
ditto "$FRAMEWORK_SOURCE" "$APP_FRAMEWORKS/Chromium Embedded Framework.framework"
ditto "$HELPERS_SOURCE" "$APP_FRAMEWORKS"
install -m 644 "$PLIST_SOURCE" "$APP_ROOT/Contents/Info.plist"
install -m 755 "$BINARY" "$APP_EXECUTABLE"
codesign --force --options runtime --entitlements Entitlements.dev.plist \
  --sign - "$APP_EXECUTABLE" >/dev/null
codesign --force --options runtime --entitlements Entitlements.dev.plist \
  --sign - "$APP_ROOT" >/dev/null

exec "$APP_EXECUTABLE" "$@"
