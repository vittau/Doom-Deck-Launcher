# 086 — Install click event

## Outcome

Both "Install on Steam Deck" buttons on the landing page now send `install_click`, and the Donate
link keeps sending `donate_click` (spec 085). The two previous single-purpose handlers became one
mechanism: a link that should be measured declares `data-ga-event="<name>"`, and one small script
wires every such link. Nothing else about the page changed — still one Google tag, one measurement
ID, no parameters, no identifier, no new request, and the links behave exactly as before with
JavaScript disabled or the tag blocked.

## Changes

| File | Change |
| --- | --- |
| `packaging/flatpak/site/index.html` | `data-ga-event="install_click"` on both install buttons and `data-ga-event="donate_click"` on the Donate link; the donate-only listener replaced by one handler for every `[data-ga-event]` link |
| `packaging/flatpak/site/test_render.py` | `test_donate_click_sends_one_named_event_without_parameters` became `test_named_action_events_cover_donate_and_install` |
| `docs/specs/085-donate-click-event.md` | Amendment note pointing at this spec |
| `docs/specs/086-install-click-event.md` | This spec |

The page script is now:

```html
<script>
  (function () {
    var links = document.querySelectorAll('[data-ga-event]');
    Array.prototype.forEach.call(links, function (link) {
      link.addEventListener('click', function () {
        if (typeof gtag === 'function') {
          gtag('event', link.getAttribute('data-ga-event'));
        }
      });
    });
  })();
</script>
```

**Why names live in markup.** A reader of the HTML sees which links are measured and what each one
is called, without reading JavaScript; adding a third measured link is one attribute, not another
handler. The test also pins this: the script must not contain either event name.

**Why install needs its own event.** GA4's automatic events do not cover these buttons: enhanced
measurement reports `click` for links *leading away from the domain*, and its `file_download`
option only watches a fixed extension list (`pdf`, `exe`, `pkg`, `zip`, …) that does not include
`flatpakref` ([Enhanced measurement events](https://support.google.com/analytics/answer/9216061)).
Without `install_click` the site's primary action would be invisible beyond the `page_view`.

**Two buttons, one event.** The hero button and the Install-section button are the same action, so
they share the name. Container clicks do not double-count: one click produces one event. A visitor
who clicks both buttons produces two — this counts clicks, not installs, and the report says so
below.

## Requirement evidence

| Spec requirement | Where it is satisfied |
| --- | --- |
| R1 — `install_click` on both buttons, `donate_click` preserved, names in markup | Three `data-ga-event` attributes in `index.html`; one `addEventListener('click', …)` handler for all of them |
| R2 — closed set of two events on three links | Test reads every `data-ga-event` in document order and requires exactly `["install_click", "donate_click", "install_click"]`, all on anchors |
| R3 — no parameters, one call site | Test requires exactly one `gtag('event'` in the page, the call `gtag('event', link.getAttribute('data-ga-event'))` once, no `{` after the event name, and no `user_id`/`page_location`/`send_to`/`link_url` after it |
| R4 — link contracts preserved | Test asserts both install anchors keep `download` and a `.flatpakref` `href` with `class="button primary"`, the Donate anchor keeps `href`/`target="_blank"`/`rel="noopener"`; handler free of `preventDefault`; the install attributes still present with every `<script>` stripped |
| R5 — other pages untouched | `assets/credits.html` and `404.html` asserted free of `gtag('event'`, `data-ga-event` and both event names; the 082 tag test still requires one loader and one `config` per page |
| R6 — tests and local verification | 42 site tests, 23 renderer-workflow tests and `test_served_urls.sh` pass; local render and `node --check` (below) |
| R7 — property-side reporting | Owner action; see follow-ups |

## Verification (local, headless)

| Check | Result |
| --- | --- |
| `python3 -m unittest discover -s packaging/flatpak/site` | **42 passed** (one test rewritten, none added or removed) |
| `python3 -m unittest discover -s scripts/flatpak -p 'test_*.py'` | **23 passed** (unchanged) |
| `bash scripts/flatpak/test_served_urls.sh` | passed |
| Real render (`render-site.py --version 0.0.11 --date 2026-10-22`) | `index.html` has **2** `install_click` and **1** `donate_click` declarations; `credits.html`/`404.html` have none |
| `node --check` on the extracted handler | clean |
| `git diff --check` | clean |

No visible window, screen capture, network upload or Rust build (no Rust file changed).

## Publication

The Donate event (spec 085) is already live: the "Project website" run for `c6e4d5c` succeeded and
`https://doomdecklauncher.com/` serves the page containing `donate_click`. This spec's change
reaches the site through the same workflow after it is merged to `main`.

**Confirmed:** the owner observed both `donate_click` and `install_click` arriving in Google
Analytics on 2026-10-22, so the markup, the handler and the property all agree in practice, not
just in the tests.

## Not done / owner follow-ups

- **Reception is owner-verified.** After publication, click each button once and confirm
  `install_click` (twice) and `donate_click` (once) in Realtime or DebugView.
- **Optional key event.** Admin → Events → `install_click` → *Mark as key event* makes downloads
  of the launcher read as the site's conversion next to `donate_click`.
- **Clicks, not installs.** `install_click` measures intent: the browser accepts the `.flatpakref`
  download, and whether Discover installs it is outside the website's visibility. Do not read it
  as an install count.
- **Unmeasured on purpose:** the `flatpak install` command snippets in the FAQ, the GitHub
  Releases archive link, the screenshots and the navigation are not measured. Each would be a new
  decision, and the `llms.txt`/crawler surface must stay free of tracking.
- **Decisions proposal (not edited here):** a recorded decision would read *"Website analytics
  stays page-view only plus the closed set `donate_click`/`install_click`, declared in markup;
  any further custom event needs its own spec."* `docs/decisions.md` remains the source of truth
  and is coordinator-owned.

## Limitations and open questions

- Tests assert markup, declarations and handler source; they do not run a browser. Real delivery
  still depends on the visitor's browser allowing the Google tag, so absolute counts undercount.
- A visitor with the tag blocked produces no event even though the download happens — the page is
  not measured, the download is not broken.
- No consent layer exists (spec 082 excluded it); if one is ever added, both events are covered by
  it automatically because they go through the same tag.
