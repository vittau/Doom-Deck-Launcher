#!/usr/bin/env python3
"""Purge the DoomDeckLauncher cache prefix on Cloudflare after a publication.

Spec 063 R6. The owner asked for a single scoped prefix purge instead of
enumerating files: it covers the whole DDL tree and nothing else.

Safety rules:
* exactly one prefix, ``www.vitormach.dev/Doom-Deck-Launcher/``;
* never ``purge_everything``, never a host or zone purge;
* the token is read from the environment and never printed, logged or passed
  as an argument;
* a bounded number of attempts, retrying only rate limits and server errors;
* a partial configuration fails, while both settings missing skips the purge.

Official API: https://developers.cloudflare.com/api/resources/cache/methods/purge/
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request

API_BASE = "https://api.cloudflare.com/client/v4"
SITE_URL = "https://www.vitormach.dev/Doom-Deck-Launcher/"
ALLOWED_PREFIX = "www.vitormach.dev/Doom-Deck-Launcher/"
TOKEN_ENV = "CLOUDFLARE_CACHE_PURGE_TOKEN"
ZONE_ENV = "CLOUDFLARE_ZONE_ID"
MAX_ATTEMPTS = 3
RETRY_STATUS = frozenset({429, 500, 502, 503, 504})
TIMEOUT_SECONDS = 20
MAX_RESPONSE_BYTES = 8192
ZONE_PATTERN = re.compile(r"^[0-9a-fA-F]{32}$")


class ConfigError(Exception):
    """Invalid or incomplete configuration; nothing was sent."""


class PurgeError(Exception):
    """Cloudflare rejected the purge or every attempt failed."""


def site_prefix(site_url: str) -> str:
    """Return the host+path prefix for ``site_url`` without the scheme."""
    if not site_url.startswith("https://"):
        raise ConfigError("the site URL must use https")
    host_path = site_url[len("https://") :]
    if not host_path.endswith("/"):
        host_path += "/"
    if host_path != ALLOWED_PREFIX:
        raise ConfigError(
            f"refusing to purge {host_path!r}: only {ALLOWED_PREFIX!r} is allowed"
        )
    return host_path


def build_payload(prefix: str) -> dict:
    """Cloudflare prefix payload: one entry, no scheme, no query or fragment."""
    if prefix != ALLOWED_PREFIX:
        raise ConfigError(f"refusing to purge {prefix!r}")
    return {"prefixes": [prefix]}


def post_json(url: str, token: str, body: bytes) -> tuple[int, bytes]:
    """POST ``body``; return (status, at most MAX_RESPONSE_BYTES+1).

    HTTP errors are data, not exceptions. Reading is capped so a hostile or
    broken response cannot grow memory without bound.
    """
    request = urllib.request.Request(
        url,
        data=body,
        method="POST",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
            return response.status, response.read(MAX_RESPONSE_BYTES + 1)
    except urllib.error.HTTPError as error:
        return error.code, error.read(MAX_RESPONSE_BYTES + 1)


def redact(text: str, token: str) -> str:
    """Remove the credential from anything that may reach stderr."""
    text = str(text)
    if token:
        text = text.replace(token, "[REDACTED]")
    return text


def _api_messages(raw: bytes, token: str) -> str:
    """Sanitized error codes/messages; never the raw body."""
    try:
        payload = json.loads(raw.decode("utf-8", "replace") or "{}")
    except json.JSONDecodeError:
        return "non-JSON response"
    if not isinstance(payload, dict):
        return "unexpected response shape"
    parts = []
    for entry in payload.get("errors") or []:
        if not isinstance(entry, dict):
            continue
        code = entry.get("code")
        message = entry.get("message")
        if code is not None and message:
            parts.append(f"{code}: {message}")
        elif message:
            parts.append(str(message))
    return redact("; ".join(parts) or "no error details", token)[:300]


def purge(
    zone: str,
    token: str,
    payload: dict,
    *,
    post=post_json,
    sleep=time.sleep,
    attempts: int = MAX_ATTEMPTS,
) -> None:
    """Send the purge, retrying only transient failures."""
    url = f"{API_BASE}/zones/{zone}/purge_cache"
    body = json.dumps(payload).encode("utf-8")
    last_error = "no attempt made"
    for attempt in range(1, attempts + 1):
        retryable = False
        try:
            status, raw = post(url, token, body)
        except (urllib.error.URLError, TimeoutError, OSError) as error:
            last_error = redact(f"network error: {error}", token)[:300]
            retryable = True
        else:
            if len(raw) > MAX_RESPONSE_BYTES:
                last_error = (
                    f"HTTP {status}: response exceeded {MAX_RESPONSE_BYTES} bytes"
                )
            else:
                try:
                    decoded = json.loads(raw.decode("utf-8", "replace") or "{}")
                except json.JSONDecodeError:
                    decoded = None
                if (
                    status == 200
                    and isinstance(decoded, dict)
                    and decoded.get("success") is True
                ):
                    return
                last_error = f"HTTP {status}: {_api_messages(raw, token)}"
                retryable = status in RETRY_STATUS
        if attempt == attempts or not retryable:
            break
        sleep(attempt * 2)
    raise PurgeError(last_error)


def run(
    *,
    site_url: str,
    zone: str,
    token: str,
    post=post_json,
    sleep=time.sleep,
    out=None,
    err=None,
) -> int:
    """CLI body, separated from argument parsing so tests stay offline."""
    out = out or sys.stdout
    err = err or sys.stderr

    if not token and not zone:
        print(
            f"Cloudflare purge skipped: neither {TOKEN_ENV} nor {ZONE_ENV} is set",
            file=out,
        )
        return 0
    missing = []
    if not token:
        missing.append(TOKEN_ENV)
    if not zone:
        missing.append(ZONE_ENV)
    if missing:
        print(
            "Cloudflare purge misconfigured: missing " + " and ".join(missing),
            file=err,
        )
        return 2

    try:
        if not ZONE_PATTERN.match(zone):
            raise ConfigError(f"{ZONE_ENV} is not a 32-character zone id")
        prefix = site_prefix(site_url)
        payload = build_payload(prefix)
        purge(zone, token, payload, post=post, sleep=sleep)
    except ConfigError as error:
        print(f"Cloudflare purge misconfigured: {error}", file=err)
        return 2
    except PurgeError as error:
        print(f"Cloudflare purge failed: {error}", file=err)
        return 1

    print(f"Cloudflare purge accepted for prefix {prefix}", file=out)
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--site-url",
        default=SITE_URL,
        help="published site URL; only the DDL prefix is accepted",
    )
    arguments = parser.parse_args(argv)
    return run(
        site_url=arguments.site_url,
        zone=os.environ.get(ZONE_ENV, "").strip(),
        token=os.environ.get(TOKEN_ENV, "").strip(),
    )


if __name__ == "__main__":
    sys.exit(main())
