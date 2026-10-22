# 085 — Measure Donate clicks with one named GA4 event

**Status:** Ready. **Origin:** owner asked on 2026-10-22 whether a click on Donate is captured,
and accepted an explicit `donate_click` event. Spec 082 deliberately forbade custom events; this
narrow spec amends that for exactly one event on one link and leaves the rest of 082 in force.

**Amended by spec 086:** the closed set of custom events grew to `{donate_click, install_click}`,
and the per-link `id` used here became the declarative `data-ga-event` attribute. The guarantees
below (no parameters, no navigation change, no event on other pages) still hold.

## Scope

The landing page hero carries the only donation link on the site (spec 083 R7). GA4 already
reports an automatic `page_view` per page (spec 082) and its enhanced measurement may report the
outbound click as a generic `click`; neither is a countable, named action in the property's own
reports. Add one named event so the owner can read donations directly, without adding tracking
surface, parameters, scripts, vendors or identifiers.

## Requirements

- R1. Exactly one custom event, named `donate_click`, is sent when the landing page's Donate link
  is activated (any activation that produces a `click` event: pointer or keyboard). It is sent
  through the `gtag` helper that spec 082 already installs — no new loader, script source,
  measurement ID, vendor or network request.
- R2. The event carries **no parameters**: no user identifier, no URL or `page_location`
  override, no `link_url`, no `send_to`, no `set`. The event name alone identifies the action.
- R3. The link keeps its current behaviour: a normal anchor to `https://buymeacoffee.com/vittau`
  with `target="_blank"` and `rel="noopener"`. The handler must not `preventDefault`, delay or
  otherwise alter navigation, and the link must keep working with JavaScript disabled and with
  the Google tag blocked by an extension or network filter.
- R4. The handler must not throw when the tag is unavailable; failures degrade to no event.
- R5. Nothing else changes: `credits.html` and `404.html` gain no event, the loader, `config`
  call, measurement ID, metadata, assets and publication guards stay as they are.
- R6. Tests cover the new behaviour and the preserved behaviour (one event on the landing page,
  none on the other pages, still no other `gtag('event')`/identifiers, link works without
  scripts). Verification is the existing offline suites plus a local render; no visible window,
  screen capture, network upload, push, tag or release.
- R7. The GA4 property's own configuration (marking `donate_click` as a key event, reporting) is
  an owner action in the Analytics UI, not a code change; after publication the owner confirms
  reception in Realtime or DebugView.

## Ownership

Own `packaging/flatpak/site/index.html`, the site renderer test suite and
`docs/reports/085-donate-click-event.md`. Do not edit `OVERVIEW.md`, `docs/decisions.md`,
`docs/status.md`, `docs/journal.yml`, the renderer, workflows or any Rust file. If the added event
should become a recorded decision, propose the entry in the report instead of editing decisions.
