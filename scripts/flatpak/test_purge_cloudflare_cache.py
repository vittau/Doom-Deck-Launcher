#!/usr/bin/env python3
"""Offline tests for the scoped Cloudflare purge helper (spec 063 R6/A5)."""

import io
import json
import unittest
import urllib.error

import purge_cloudflare_cache as purge_mod

ZONE = "0123456789abcdef0123456789abcdef"
TOKEN = "test-token-never-print-me"
PREFIX = "www.vitormach.dev/Doom-Deck-Launcher/"


class FakePost:
    """Stand-in for the HTTP call; records every request."""

    def __init__(self, responses):
        self.responses = list(responses)
        self.calls = []

    def __call__(self, url, token, body):
        self.calls.append({"url": url, "token": token, "body": body})
        response = self.responses[min(len(self.calls) - 1, len(self.responses) - 1)]
        if isinstance(response, Exception):
            raise response
        return response


def ok():
    return 200, json.dumps(
        {"success": True, "errors": [], "result": {"id": "purge-id"}}
    ).encode()


def rejected(message="Unable to purge, rate limit reached"):
    return 200, json.dumps(
        {"success": False, "errors": [{"code": 1134, "message": message}], "result": None}
    ).encode()


def run(*, site_url=purge_mod.SITE_URL, zone=ZONE, token=TOKEN, responses=None):
    post = FakePost(responses or [ok()])
    sleeps = []
    out, err = io.StringIO(), io.StringIO()
    code = purge_mod.run(
        site_url=site_url,
        zone=zone,
        token=token,
        post=post,
        sleep=sleeps.append,
        out=out,
        err=err,
    )
    return code, post, sleeps, out.getvalue(), err.getvalue()


class PrefixScopeTests(unittest.TestCase):
    def test_default_prefix_has_no_scheme(self):
        self.assertEqual(purge_mod.site_prefix(purge_mod.SITE_URL), PREFIX)

    def test_trailing_slash_is_added(self):
        self.assertEqual(
            purge_mod.site_prefix(purge_mod.SITE_URL.rstrip("/")), PREFIX
        )

    def test_other_prefix_is_rejected(self):
        for url in (
            "https://www.vitormach.dev/Other/",
            "https://example.com/Doom-Deck-Launcher/",
            "http://www.vitormach.dev/Doom-Deck-Launcher/",
            "https://www.vitormach.dev/",
        ):
            with self.subTest(url=url):
                with self.assertRaises(purge_mod.ConfigError):
                    purge_mod.site_prefix(url)

    def test_payload_is_one_prefix_only(self):
        payload = purge_mod.build_payload(PREFIX)
        self.assertEqual(payload, {"prefixes": [PREFIX]})
        self.assertEqual(list(payload), ["prefixes"])
        encoded = json.dumps(payload)
        self.assertNotIn("purge_everything", encoded)
        self.assertNotIn("files", encoded)
        self.assertNotIn("hosts", encoded)
        self.assertNotIn("https://", encoded)


class ConfigurationTests(unittest.TestCase):
    def test_missing_both_skips_without_calling_the_api(self):
        code, post, _, out, err = run(zone="", token="")
        self.assertEqual(code, 0)
        self.assertEqual(post.calls, [])
        self.assertIn("skipped", out)
        self.assertEqual(err, "")

    def test_missing_zone_fails(self):
        code, post, _, _, err = run(zone="", token=TOKEN)
        self.assertEqual(code, 2)
        self.assertEqual(post.calls, [])
        self.assertIn(purge_mod.ZONE_ENV, err)

    def test_missing_token_fails(self):
        code, post, _, _, err = run(zone=ZONE, token="")
        self.assertEqual(code, 2)
        self.assertEqual(post.calls, [])
        self.assertIn(purge_mod.TOKEN_ENV, err)

    def test_invalid_zone_fails_before_the_request(self):
        code, post, _, _, err = run(zone="not-a-zone", token=TOKEN)
        self.assertEqual(code, 2)
        self.assertEqual(post.calls, [])
        self.assertIn("zone", err)


class RequestTests(unittest.TestCase):
    def test_success_sends_one_scoped_prefix(self):
        code, post, _, out, err = run()
        self.assertEqual(code, 0)
        self.assertEqual(len(post.calls), 1)
        call = post.calls[0]
        self.assertEqual(
            call["url"],
            f"{purge_mod.API_BASE}/zones/{ZONE}/purge_cache",
        )
        self.assertEqual(json.loads(call["body"]), {"prefixes": [PREFIX]})
        self.assertIn(PREFIX, out)
        self.assertEqual(err, "")

    def test_token_is_never_printed(self):
        for responses in ([ok()], [rejected()], [(400, b'{"success":false}')]):
            with self.subTest(responses=responses):
                _, _, _, out, err = run(responses=responses)
                self.assertNotIn(TOKEN, out)
                self.assertNotIn(TOKEN, err)

    def test_api_rejection_fails_with_the_api_message(self):
        code, _, _, _, err = run(responses=[rejected("purge denied")])
        self.assertEqual(code, 1)
        self.assertIn("purge denied", err)
        self.assertIn("HTTP 200", err)

    def test_client_error_is_not_retried(self):
        code, post, sleeps, _, err = run(responses=[(400, b"bad request")])
        self.assertEqual(code, 1)
        self.assertEqual(len(post.calls), 1)
        self.assertEqual(sleeps, [])
        self.assertIn("HTTP 400", err)

    def test_rate_limit_is_retried_once(self):
        code, post, sleeps, _, _ = run(responses=[(429, b"{}"), ok()])
        self.assertEqual(code, 0)
        self.assertEqual(len(post.calls), 2)
        self.assertEqual(len(sleeps), 1)

    def test_server_errors_stop_after_the_bounded_attempts(self):
        code, post, _, _, err = run(responses=[(500, b"boom")])
        self.assertEqual(code, 1)
        self.assertEqual(len(post.calls), purge_mod.MAX_ATTEMPTS)
        self.assertIn("HTTP 500", err)

    def test_network_error_is_retried(self):
        code, post, _, _, _ = run(
            responses=[urllib.error.URLError("connection reset"), ok()]
        )
        self.assertEqual(code, 0)
        self.assertEqual(len(post.calls), 2)


if __name__ == "__main__":
    unittest.main()
