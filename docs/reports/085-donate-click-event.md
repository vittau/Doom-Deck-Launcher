# 085 — Donate click event

## Outcome

The landing page's Donate link now sends one named GA4 event, `donate_click`, through the `gtag`
helper that spec 082 already installs. Nothing else about the page changed: no new script source,
no measurement ID, no parameters, no identifier, no URL override, no consent layer. The link is
still a plain anchor, and the site works exactly as before with JavaScript disabled or with the
Google tag blocked.

Answering the owner's question directly: before this change the site measured `page_view` only,
and spec 082 explicitly forbade custom events — an outbound click might have appeared as GA4's
automatic generic `click`, but never as a countable action named after a donation. Now it does.

## Changes

| File | Change |
| --- | --- |
| `packaging/flatpak/site/index.html` | `id="donate-link"` on the existing anchor; one small script before `</body>` that calls `gtag('event', 'donate_click')` on click |
| `packaging/flatpak/site/test_render.py` | New test `test_donate_click_sends_one_named_event_without_parameters`; the 082 tag test now allows exactly one event on the landing page and none elsewhere |
| `packaging/flatpak/site/test_discovery.py` | The script-closure count of the landing page went from three to four, with the comment naming the new script |
| `docs/specs/085-donate-click-event.md` | The spec this implements |

The generated block is:

```html
<script>
  (function () {
    var donate = document.getElementById('donate-link');
    if (!donate) return;
    donate.addEventListener('click', function () {
      if (typeof gtag === 'function') gtag('event', 'donate_click');
    });
  })();
</script>
```

**Why a listener instead of `onclick="…"`.** The handler lives in the page's own plain script, not
in an attribute: the markup stays readable, and there is one place to see the whole measurement.
It is placed after the link, so the element exists when it runs.

**Why the event has no parameters.** Spec 085 R2 asks for exactly that. The name identifies the
action, and GA4's automatic outbound `click` already carries `link_url` for anyone who wants the
destination. Sending nothing keeps the payload identical to page-view data in kind.

**Why the link still navigates.** The handler never calls `preventDefault` and never delays the
click; `target="_blank"` opens the donation page in a new tab while the landing page stays loaded,
so the hit is not at risk from an unload. With the loader blocked, `typeof gtag === 'function'` is
false and the click simply does nothing extra.

## Requirement evidence

| Spec requirement | Where it is satisfied |
| --- | --- |
| R1 — exactly one named event on link activation | One call `gtag('event', 'donate_click')` in `index.html`, in the only script that references the link; test asserts the call exists once and appears after `id="donate-link"` and before `</body>` |
| R1 — through the existing `gtag` helper, no new request | The handler contains no `googletagmanager` string; the only loader is the spec-082 one, still exactly one per page |
| R2 — no parameters, identifiers or URL data | Test rejects `{` after `gtag('event',` and any `user_id`, `page_location`, `send_to`, `link_url` after the call |
| R3 — link behaviour unchanged, works without JS | Anchor attributes asserted unchanged (`href`, `target="_blank"`, `rel="noopener"`); handler asserted free of `preventDefault`; the existing donation test still finds the link with every `<script>` stripped |
| R4 — no exception when the tag is unavailable | `typeof gtag === 'function'` guard; the handler also returns early if the link is absent |
| R5 — nothing else changes | `assets/credits.html` and `404.html` asserted free of `gtag('event'` and `donate_click`; metadata, assets, publication guards and the single-ID rule untouched |
| R6 — tests and local verification | 42 site tests, 23 renderer-workflow tests and `test_served_urls.sh` all pass; local render checked (below) |
| R7 — property configuration is the owner's | Not done here; see follow-ups |

Event name validity: Google's [Event naming rules](https://support.google.com/analytics/answer/13316687)
require a letter first and only letters, numbers and underscores, and list `click`, `page_view`,
`file_download` etc. as reserved — `donate_click` meets the rules and is not reserved.

## Verification (local, headless)

| Check | Result |
| --- | --- |
| `python3 -m unittest discover -s packaging/flatpak/site` | **42 passed** (41 existing + 1 new, one existing test amended) |
| `python3 -m unittest discover -s scripts/flatpak -p 'test_*.py'` | **23 passed** (unchanged) |
| `bash scripts/flatpak/test_served_urls.sh` | passed |
| Real render (`render-site.py --version 0.0.11 --date 2026-10-22`) | `index.html` has **1** `donate_click` call, **1** `id="donate-link"`, **0** unresolved `@@TOKEN@@`; `assets/credits.html` and `404.html` have **0** events |
| `node --check` on the extracted handler | clean — the script parses |
| `git diff --check` | clean |

No visible window opened, no screen capture, no network upload, no publication. No Rust file
changed, so no Rust build was needed.

## Not done / owner follow-ups

- **Publication.** The change reaches `https://doomdecklauncher.com/` only through the website
  workflow; the live-source check happens after it runs.
- **Reception is owner-verified.** Confirm `donate_click` in Analytics → Realtime (or DebugView)
  with one real click after publication. This worker can prove the markup and the code, not that
  Google stored the hit.
- **Optional: mark it a key event.** Admin → Events → `donate_click` → *Mark as key event* turns
  the count into a conversion-style metric. That is property configuration, not code.
- **Expected duplication.** Enhanced measurement may still record the same click as a generic
  `click` with `link_url` containing `buymeacoffee.com`. That is Google's automatic behaviour, not
  a second measurement we added; count `donate_click`.

## Limitations and open questions

- The tests validate generated markup and handler source, not browser execution. A click is only
  measurable when the browser allows the Google tag to load: ad blockers and tracking-protection
  features suppress both the automatic and the custom event, so absolute numbers will undercount.
- No consent layer exists (spec 082 excluded it, and this spec does not add one). If consent
  behaviour is ever required, it applies to the whole tag, `donate_click` included.
- **Decision worth recording (proposed, not edited here):** spec 082's "no custom events" rule now
  has exactly one documented exception. A decision entry such as *"Website analytics stays
  page-view only, plus one explicit `donate_click` event with no parameters; any further custom
  event needs its own spec"* would keep `docs/decisions.md` the source of truth.
