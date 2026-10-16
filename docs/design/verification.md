# Design delivery and verification

Task: DDL UI design — design system and screen mockups.

## Result

Created the UI contract, all eight requested screens and two supplementary screens (Library and AUGER;ZENITH setup), a reproducible Pillow renderer, and unmodified Inter 4.1 font with its OFL notice. No files outside `docs/design/` changed; no Rust/Slint code or runtime dependencies added.

## Verification

Pillow 12.3.0 rendered every screen successfully. Automated assertions checked 1280×800 dimensions and the 1,500,000-byte ceiling. Manual review of all ten renders found no clipped text, overlapping chrome, missing focus ring or missing hint bar. Cover placeholders are original neutral geometry labelled “cover art”; no DOOM art, logos or map screenshots are used.

### Contrast

Computed sRGB/WCAG relative-luminance contrast (brightest surface is the worst case).

| Foreground | Canvas `#101418` | Surface `#1B2229` | Raised `#26313B` |
| --- | ---: | ---: | ---: |
| text | 16.40:1 | 14.24:1 | 11.75:1 |
| text-muted | 10.33:1 | 8.97:1 | 7.40:1 |
| focus / warning | 12.27:1 | 10.66:1 | 8.79:1 |
| success | 11.10:1 | 9.64:1 | 7.95:1 |
| danger | 9.85:1 | 8.56:1 | 7.06:1 |

Accent button: `#101418` on `#FFCA70` = 12.27:1.

### Raster inventory

| File | Dimensions | Bytes |
| --- | --- | ---: |
| [discover.png](mockups/discover.png) | 1280×800 | 52,397 |
| [download-install.png](mockups/download-install.png) | 1280×800 | 60,627 |
| [first-run.png](mockups/first-run.png) | 1280×800 | 69,581 |
| [home.png](mockups/home.png) | 1280×800 | 68,909 |
| [library.png](mockups/library.png) | 1280×800 | 52,182 |
| [mapset-detail.png](mockups/mapset-detail.png) | 1280×800 | 70,809 |
| [play-blocked-iwad.png](mockups/play-blocked-iwad.png) | 1280×800 | 91,312 |
| [play-setup-auger.png](mockups/play-setup-auger.png) | 1280×800 | 79,133 |
| [play-setup-sigil.png](mockups/play-setup-sigil.png) | 1280×800 | 87,646 |
| [settings.png](mockups/settings.png) | 1280×800 | 69,792 |

### Required sips output

Command: `sips -g pixelWidth -g pixelHeight docs/design/mockups/*.png`

```text
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/discover.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/download-install.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/first-run.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/home.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/library.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/mapset-detail.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/play-blocked-iwad.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/play-setup-auger.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/play-setup-sigil.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/mockups/settings.png
  pixelWidth: 1280
  pixelHeight: 800
```

## Scope and remaining validation

All 12 prior-art §5 critique items have individual rows in ui.md §7. Every requested screen has purpose, layout, focus behavior, loading/empty/error/offline behavior and copy in §4. D-008 translation, D-009 pointer behavior and D-010/D-014 components/IWAD gates are specified. No application tests run because this delivery contains no application changes; future behavioral tests and Deck checks are enumerated in ui.md §8.

Open design/data questions: current paid SIGIL identities and whether compat stays exposed; reviewed AUGER;ZENITH add-on metadata and complevel; Deck validation of cl3/episodes, pistol start and controller return/hotplug; initial engine/template version and final evidence vocabulary. These are explicit release gates, not inferred facts.

Typeface: Inter 4.1, SIL Open Font License 1.1; source https://github.com/rsms/inter/releases/tag/v4.1 and license https://github.com/rsms/inter/blob/v4.1/LICENSE.txt. The font is bundled here solely to reproduce mockups; an implementation spec must bundle the license with any app font distribution.

## Reproduce

Install Pillow in an isolated environment, then run `python3 docs/design/render_mockups.py`. Renderer uses only assets beside it and writes the ten PNGs; no network access is needed. The temporary Pillow install used for this delivery is removed from the worktree.

Visual follow-up: the wireframes have been re-rendered with B = Back, Start = Menu and the unclear header removed; see [visual-report.md](visual-report.md) for six high-fidelity concepts and updated visual guidance.
