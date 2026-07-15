#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

HELPER_BINARY="target/release/cl-go-dash-helper"
STAGE="target/cef-runtime/macos"
HELPERS=(
  "CL-GO Helper"
  "CL-GO Helper (GPU)"
  "CL-GO Helper (Renderer)"
  "CL-GO Helper (Plugin)"
  "CL-GO Helper (Alerts)"
)
INPUTS=(
  "Cargo.toml"
  "Cargo.lock"
  "build.rs"
  "src/bin/cl-go-dash-helper.rs"
)

CACHE_VALID=true
if [[ ! -x "$HELPER_BINARY" \
  || ! -d "$STAGE/Chromium Embedded Framework.framework" \
  || ! -s "$STAGE/LICENSE.txt" ]]; then
  CACHE_VALID=false
fi

for helper in "${HELPERS[@]}"; do
  if [[ ! -x "$STAGE/helpers/$helper.app/Contents/MacOS/$helper" ]]; then
    CACHE_VALID=false
    break
  fi
done

if [[ "$CACHE_VALID" == true ]]; then
  for input in "${INPUTS[@]}" resources/cef/macos/helpers/*/Contents/Info.plist; do
    if [[ ! -f "$input" || "$input" -nt "$HELPER_BINARY" ]]; then
      CACHE_VALID=false
      break
    fi
  done
fi

if [[ "$CACHE_VALID" == true ]]; then
  exit 0
fi

bash scripts/prepare-cef.sh
