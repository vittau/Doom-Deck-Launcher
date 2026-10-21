#!/usr/bin/env python3
"""Offline tests for the scoped Cloudflare purge helper (spec 063 R6/A5)."""

import io
import json
import unittest
import unittest.mock as mock
import urllib.error

import purge_cloudflare_cache as purge_mod

ZONE = "0123456789abcdef0123456789abcdef"
TOKEN = "test-token-never-print-me"
PREFIX = "doomdecklauncher.com/"


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
            "https://www.vitormach.dev/Doom-Deck-Launcher/",
            "https://www.vitormach.dev/",
            "https://vittau.github.io/Doom-Deck-Launcher/",
            "http://doomdecklauncher.com/",
            "https://www.doomdecklauncher.com/",
            "https://doomdecklauncher.com/other/",
            "https://doomdecklauncher.com/Doom-Deck-Launcher/",
            "https://doomdecklauncher.com/?query=1",
            "https://doomdecklauncher.com/#fragment",
            "https://doomdecklauncher.com:443/",
            "https://user@doomdecklauncher.com/",
            "https://doomdecklauncher.com.evil.example/",
            "https://doomdecklauncher.com//",
            "https://doomdecklauncher.com/%2f",
            "https://doomdecklauncher.com/../",
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

    def test_invalid_prefix_payloads_fail_before_any_request(self):
        for prefix in ("doomdecklauncher.com", "doomdecklauncher.com/*",
                       "doomdecklauncher.com/other/", "https://doomdecklauncher.com/"):
            with self.subTest(prefix=prefix), self.assertRaises(purge_mod.ConfigError):
                purge_mod.build_payload(prefix)

    def test_wrong_site_with_credentials_never_calls_api(self):
        code, post, _, _, _ = run(site_url="https://example.com/")
        self.assertEqual(code, 2)
        self.assertEqual(post.calls, [])


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

    def test_success_must_be_the_literal_boolean_true(self):
        for body in (b'{"success": "true"}', b'{"success": 1}', b'{"result": {}}'):
            with self.subTest(body=body):
                code, _, _, _, err = run(responses=[(200, body)])
                self.assertEqual(code, 1)
                self.assertIn("HTTP 200", err)

    def test_oversize_response_is_rejected_without_retry(self):
        oversized = b"x" * (purge_mod.MAX_RESPONSE_BYTES + 1)
        code, post, sleeps, _, err = run(responses=[(200, oversized)])
        self.assertEqual(code, 1)
        self.assertEqual(len(post.calls), 1)
        self.assertEqual(sleeps, [])
        self.assertIn("exceeded", err)
        self.assertNotIn("x" * 32, err)

    def test_non_json_error_is_sanitized(self):
        body = b"<html>internal detail</html>"
        code, _, _, _, err = run(responses=[(200, body)])
        self.assertEqual(code, 1)
        self.assertIn("non-JSON response", err)
        self.assertNotIn("<html>", err)

    def test_token_in_api_message_is_redacted(self):
        code, _, _, _, err = run(responses=[rejected(f"bad request for {TOKEN}")])
        self.assertEqual(code, 1)
        self.assertIn("[REDACTED]", err)
        self.assertNotIn(TOKEN, err)

    def test_token_in_network_error_is_redacted(self):
        code, post, _, _, err = run(
            responses=[urllib.error.URLError(f"reset while sending {TOKEN}")]
        )
        self.assertEqual(code, 1)
        self.assertEqual(len(post.calls), purge_mod.MAX_ATTEMPTS)
        self.assertIn("[REDACTED]", err)
        self.assertNotIn(TOKEN, err)


class TransportCapTests(unittest.TestCase):
    def test_post_reads_at_most_the_cap(self):
        read_sizes = []

        class Response:
            status = 200

            def read(self, size=-1):
                read_sizes.append(size)
                return b"{}"

            def __enter__(self):
                return self

            def __exit__(self, *exc_info):
                return False

        with mock.patch.object(
            purge_mod.urllib.request, "urlopen", return_value=Response()
        ):
            status, raw = purge_mod.post_json("https://example.invalid", TOKEN, b"{}")
        self.assertEqual(status, 200)
        self.assertEqual(raw, b"{}")
        self.assertEqual(read_sizes, [purge_mod.MAX_RESPONSE_BYTES + 1])


if __name__ == "__main__":
    unittest.main()
