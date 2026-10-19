# 031 — Profiles you can see, name and switch easily

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 018, 030
**Origin:** Deck test 2026-10-19 on `v0.0.6-local`. Saves stayed separate per profile (C4 ✅),
but the owner could not tell from the Mapset page whether a profile existed or how to edit it,
could not type a profile name, and found Previous/Next profile confusing.
**Supersedes** spec 018 R3's preset-name list (owner, 2026-10-19: "Pode tirar os nomes
pré-determinados, deixa só um input pra eu escrever o nome").

## Requirements

- **R1** — **Profile visible on Mapset detail:** the detail page shows the active profile's name and
  its key settings (skill, pistol start, enabled add-ons, extra files count) with an obvious
  **Edit** that opens Play setup on the profile section. Library and Home keep showing the
  profile name as in spec 018 R5.
- **R2** — **Free-text names:** creating or renaming a profile uses one text field (no preset list).
  Names are trimmed, 1–32 characters, unique per mapset (case-insensitive); an invalid name
  shows the reason inline. Existing preset-named profiles keep their names.
- **R3** — **On-screen keyboard:** focusing the name field opens the Steam on-screen keyboard in
  Gaming Mode. Research how a non-Steam shortcut can open it (for example the
  `steam://open/keyboard` URL) from Valve documentation or well-sourced community evidence;
  cite it and label it Confirmed / Proposed / **Needs Deck Validation**. Launch any helper with
  `std::process::Command` and separate arguments, never a shell. If it cannot open, the field
  still works with Steam + X and says so in a one-line hint.
- **R4** — **Switching:** Previous/Next profile wrap around at both ends, and L1/R1 switch profile
  while the profile picker is focused or the profiles sheet is open (they do not switch tabs
  there — spec 030 R1 disables the tab bar on Play setup).
- **R5** — **Clarity:** the profiles sheet reads as one list of profiles with the active one marked
  and actions (New — copy settings, New — defaults, Rename, Delete) grouped below it, instead
  of the current confusing modal.

## Constraints

- App state schema, profile ids and saves layout are unchanged (rename changes only the display
  name). Ember Cathedral look; 1280×800; headless renders only.

## Acceptance criteria

- [ ] **A1** (R1, R5) — headless renders of Mapset detail with a profile summary and the new
  profiles sheet, inspected; a navigation test reaches Edit and lands on the profile section.
- [ ] **A2** (R2) — core tests: trim, length, case-insensitive uniqueness, old preset names kept.
- [ ] **A3** (R3) — a test with a fake launcher asserts the keyboard helper argv when the field
  gains focus; the research note is in `docs/research/` with links.
- [ ] **A4** (R4) — navigation tests: wrap at both ends; L1/R1 switch profile inside the picker and
  sheet.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green. Deck re-check: C4 naming and keyboard
  (Needs Deck Validation).
