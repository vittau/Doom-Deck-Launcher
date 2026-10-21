# 082 — Website analytics with GA4

## Outcome

Added the standard asynchronous Google tag (gtag.js) for `G-BTYE5G8WCM` to both generated pages
— the landing page and the credits page — and to nothing else. No renderer, workflow, manifest,
README or application file changed, so the existing publication guards, metadata, branding and
offline contracts are untouched. All 23 pre-existing site tests still pass, plus one new focused
test; real event reception and publication remain outside this worker.

## Changes

| File | Change |
| --- | --- |
| `packaging/flatpak/site/index.html` | Google tag snippet in `<head>`: one async loader + one `gtag('config')` |
| `packaging/flatpak/site/credits.html` | The same snippet in `<head>` (rendered to `assets/credits.html`) |
| `packaging/flatpak/site/test_render.py` | One new test: `test_google_tag_loads_and_configures_once_per_page` |

The generated block is the documented snippet with its exact code lines:

```html
<!-- Google tag (gtag.js) — website analytics only, configured once in this document. -->
<script async src="https://www.googletagmanager.com/gtag/js?id=G-BTYE5G8WCM"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'G-BTYE5G8WCM');
</script>
```

**Placement.** Google's guide says to place the snippet immediately after the opening `<head>`
tag. The tag sits two lines below it, after `<meta charset>` and the viewport meta, because
keeping the character-set declaration first is the safer ordering for a static document; it is
still ahead of the title, description, canonical, Open Graph tags and stylesheet, so the loader is
requested as early as before. This deviation is deliberate and recorded here rather than hidden.

**One initialization per page.** Each template contains exactly one loader (with `async`), one
`dataLayer` initializer and one `gtag('config', 'G-BTYE5G8WCM')`. The tag is in the template, so
the renderer copies it to the published page; nothing is injected at runtime and there is no
second initialization path. GA4 sends its automatic `page_view` when `config` runs — Google's own
guide states this — so no custom page-view event was added.

**Scope respected.** Only the measurement ID appears; there is no `gtag('event')`, no
`gtag('set')`, no `user_id`, no `page_location`/URL override, no `send_to`, no cookie or consent
code, no new dependency and no application or launcher telemetry. A GA measurement ID is public
configuration, so no secret handling was introduced.

## Requirement evidence

| Spec requirement | Where it is satisfied |
| --- | --- |
| Loading page and credits page carry the tag | Both templates; verified in a local render of the current tree |
| Initialize once per page using the documented gtag.js integration | Snippet verbatim; test asserts one async `googletagmanager.com/gtag/js?id=…` script, one `dataLayer` initializer, one `gtag('js', …)` and one `gtag('config', …)` per page |
| Website traffic only, no launcher telemetry or new application behavior | No file outside `packaging/flatpak/site/` was touched (only the two templates and one test) |
| No custom user identifiers, URL data or custom events | Test rejects `gtag('event'`, `gtag('set'`, `user_id`, `page_location`, `send_to`, and requires the only `G-…` ID in each page to be `G-BTYE5G8WCM` |
| Main content functional without JavaScript | Test strips every `<script>` block and still finds the page's own text; on the credits page the Google tag pair is the only JavaScript present at all |
| Existing metadata, branding and publication guards retained | No other edit: canonical/OG/description/favicon/stylesheet hashes, donation embed, `CNAME` writing and the `--check-publication` guard are unchanged and their tests pass |
| No secrets | Only the public measurement ID is embedded |

## Verification (local, headless)

| Check | Result |
| --- | --- |
| `python3 -m unittest discover -s packaging/flatpak/site` | **24 passed** — the 23 existing checks plus the new one |
| `python3 -m unittest discover -s scripts/flatpak -p 'test_*.py'` | **23 passed** (unchanged) |
| `bash scripts/flatpak/test_served_urls.sh` | passed |
| Local render (`render-site.py --output … --version 0.0.11 --date 2026-10-21`) | `index.html` and `assets/credits.html` each contain **1** loader, **1** `config`, the ID exactly **twice**, and **0** unresolved `@@TOKEN@@` |
| `git diff --check` | clean |

No visible window was opened and no screen capture was taken; the checks are unit tests plus a
local text render. No Rust build was needed because no Rust file changed. Nothing was pushed,
tagged or released.

## Primary source (checked 2026-10-21)

- Google, [Set up the Google tag with gtag.js](https://developers.google.com/tag-platform/gtagjs/install)
  (page last updated 2026-10-13 UTC). Used for: the snippet itself, the "immediately after the
  opening `<head>` tag … on every page you want to measure" placement guidance, the requirement
  that `config` be present, and the statement that Analytics sends `page_view` automatically when
  `config` is called unless `send_page_view` is false.
- Google Analytics help, [Find your Google tag ID](https://support.google.com/analytics/answer/9539598),
  referenced by that guide for where the ID comes from (not re-fetched in detail).

## Not done / owner follow-ups

- **Publication.** The page changes reach the live site only through the independent website
  workflow owned by the coordinator. Until that runs, `https://doomdecklauncher.com/` still serves
  the pre-082 page; the live-source check is post-deployment.
- **Real event reception is an owner check.** This worker can prove the markup is correct and
  present once; it cannot prove Google received a hit. Confirm in Analytics Realtime or DebugView
  after deployment (Tag Assistant is the vendor's alternative for verifying the tag is found).
- **Consent.** Spec 082 explicitly excludes it, so no Consent Mode, banner or cookie logic was
  added. If the owner needs regional consent behavior, that is a separate decision with its own
  primary sources — it is not a defect in this change.
- **Not measured on purpose:** no custom events, no key events, no user identifiers, no URL
  overrides; adding any of them is a new spec.

## Limitations and open questions

- The new test validates generated markup, not network behavior: an ad blocker, browser tracking
  protection or a consent layer can still suppress hits in the field.
- The measurement ID is duplicated in the two templates. That is deliberate — it keeps the
  renderer free of site-specific analytics configuration — and the test fails if the two pages
  ever disagree or if a second ID appears. Moving it to `render-site.py` as a template value is a
  reasonable later refactor if more pages are added.
- Traffic data itself (coverage, sessions, referrals) belongs to the owner's Analytics property
  and was not accessed.
