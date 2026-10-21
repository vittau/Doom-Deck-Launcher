#!/usr/bin/env bash
# Published URLs checked after a Pages deployment (spec 063 A5 follow-up).
#
# Prints three lines: the landing page, the install descriptor and the OSTree
# summary. With an empty QUERY the canonical URLs are returned, which is what a
# fresh Cloudflare purge allows; otherwise pass a cache-busting query string.

served_urls() {
  local base="${1%/}" app_id="$2" query="$3"
  printf '%s\n' \
    "${base}/${query}" \
    "${base}/${app_id}.flatpakref${query}" \
    "${base}/repo/summary${query}"
}
