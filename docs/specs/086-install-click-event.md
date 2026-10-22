# 086 — Measure Install clicks with a named GA4 event

**Status:** Ready. **Origin:** owner request on 2026-10-22 ("Quero um evento pro click no Install
tb"), right after spec 085 shipped the Donate event.

Amends spec 085 R1: the closed list of custom events on the website becomes
`{donate_click, install_click}`, both declared in markup and dispatched by one script.

## Scope

The landing page offers the launcher download twice — the hero button and the button inside the
"Install" section. Both point at `…/com.doomdecklauncher.DoomDeckLauncher.flatpakref` with the
`download` attribute, on the same domain, so GA4's automatic events do not cover them: enhanced
measurement reports outbound clicks only for links *leading away* from the domain, and
`flatpakref` is not among the file extensions its `file_download` option watches. An explicit
event is therefore the only way to count the primary action of the site.

## Requirements

- R1. `install_click` is sent when either install button is activated (pointer or keyboard), and
  `donate_click` keeps the behaviour shipped in 085. Event names are declared in the markup with a
  single attribute, `data-ga-event`, and dispatched by one handler — no per-link code.
- R2. The measured set is closed: exactly these two events on exactly three links (two install
  buttons, one Donate link). No other page, link or event.
- R3. No parameters, identifiers, URL data, `send_to`, `page_location` or `user_id`; one
  `gtag('event', …)` call site, and the event name is the whole payload.
- R4. Every measured link keeps its behaviour: the install buttons keep their `.flatpakref` `href`
  and `download` attribute, the Donate link keeps `target="_blank"`/`rel="noopener"`; the handler
  never calls `preventDefault`, delays navigation or throws when the Google tag is blocked.
- R5. `credits.html` and `404.html` gain no event; loader, `config`, measurement ID, metadata,
  assets and publication guards stay as they are (specs 082/083/085 stay in force).
- R6. Tests assert the declared set, the single call site, the parameter-free payload, the
  preserved link contracts and the script-free fallback; verification is the offline suites plus a
  local render, with no visible window, screen capture, network upload, push, tag or release by the
  worker.
- R7. Property-side reporting (Realtime confirmation, optional key-event marking) is owner work
  after publication.

## Ownership

Own `packaging/flatpak/site/index.html`, the site renderer test suite and
`docs/reports/086-install-click-event.md`. Do not edit `OVERVIEW.md`, `docs/decisions.md`,
`docs/status.md`, `docs/journal.yml`, the renderer, workflows or any Rust file.
