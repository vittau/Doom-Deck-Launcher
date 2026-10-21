#!/usr/bin/env bash
# Offline tests for served_urls (spec 063 A5 follow-up).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/flatpak/served_urls.sh
source "$here/served_urls.sh"

base="https://doomdecklauncher.com"
app_id="com.doomdecklauncher.DoomDeckLauncher"

canonical="$(printf '%s\n' \
  "$base/" \
  "$base/$app_id.flatpakref" \
  "$base/repo/summary")"
busted="$(printf '%s\n' \
  "$base/?v=1dc6e8ef50f9" \
  "$base/$app_id.flatpakref?v=1dc6e8ef50f9" \
  "$base/repo/summary?v=1dc6e8ef50f9")"

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

[ "$(served_urls "$base" "$app_id" "")" = "$canonical" ] \
  || fail "canonical URLs mismatch"
[ "$(served_urls "$base/" "$app_id" "")" = "$canonical" ] \
  || fail "trailing slash on the base URL must not double the slash"
[ "$(served_urls "$base" "$app_id" "?v=1dc6e8ef50f9")" = "$busted" ] \
  || fail "cache-busted URLs mismatch"

# The page must always come first: it is the file compared byte-for-byte.
[ "$(served_urls "$base" "$app_id" "" | head -n 1)" = "$base/" ] \
  || fail "the landing page must be the first URL"

echo "served_urls tests ok"
