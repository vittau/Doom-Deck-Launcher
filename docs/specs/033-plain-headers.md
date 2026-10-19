# 033 — Plain headers: drop decorative subtitles

**Status:** Ready (after 031, 032)  ·  **Milestone:** M3  ·  **Depends on:** 012, 030, 031, 032
**Origin:** owner, 2026-10-19 after the first Deck test: "Tire os subtítulos 'de efeito'
supérfluos em headers e elementos de interface. Não precisamos disso, tem cara de 'AI slop'."

## Requirements

- **R1** — **Audit:** list every subtitle, tagline and flavour line in the UI (screen headers such as
  "Curated worlds. Choose your next descent.", "Requirements, content and source.", "Your last
  profile, ready again.", "Build your library and begin a new session.", card and panel
  sublines, dialog sublines) with its string id, and classify each one:
  - **remove** — decorative, or restates the title;
  - **keep** — tells the player a state, a requirement or what to do next (e.g. "DDL will be
    ready when your session ends", a missing base game, an error's next step). Shorten a kept
    line to plain words where it is wordy.
- **R2** — **Remove** every "remove" line and its string. Headers become the title alone; the
  layout closes the gap (no empty band) and keeps spec 030's Discover margin under the header.
- **R3** — **Tone for new copy:** plain, short, factual. Add one line to `docs/design/ui.md` (or the
  current UI contract) stating that rule, so future work does not reintroduce taglines.

## Acceptance criteria

- [ ] **A1** (R1) — the audit table (string id, text, verdict, reason) is in the report.
- [ ] **A2** (R2) — headless renders of every root screen, first run, Mapset detail, Play setup and
  a dialog, inspected; no removed string remains in `crates/ddl/ui/`.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green; README renders refreshed.
