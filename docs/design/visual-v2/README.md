# Visual identity v2 — phase A

**Proposed**, for owner selection under spec 012 R1–R5 / A1. No production code or interaction behaviour changes. All three directions use the same production spec 010 skeleton: 104 px header; title/subtitle band; content at y=230–708; Home 784/384 px panels; setup 680/496 px panels; footer at y=736. Setup options scroll while the launch summary stays fixed. Continue and Play occupy the production coordinates and have one 4 px amber ring with a 4 px dark separation. Active tabs use an underline, selected options use a label and edge, neither is a second focus ring.

These are illustrative installed-library states, not evidence of an actual installation or gameplay verification. Home's spare panel area holds SIGIL II and AUGER;ZENITH context without changing the panel split. Setup depicts the exclusive add-on choice in the existing options column; None is selected, hence exactly one numbered content file. X/Y badges are demonstrated on each style sheet; production screen footers omit actions unavailable in that state. L1/R1 are root-only and Start opens Menu. No profile CRUD, favourites or reordering is introduced.

## Ember Cathedral

[Home](ember-cathedral/home.png) · [Play setup](ember-cathedral/play-setup-auger.png) · [Style sheet](ember-cathedral/style-sheet.png)

A carved oxblood wall and fine bronze borders make the launcher feel like an entrance to a hostile cathedral. Bone-coloured blackletter titles establish the infernal tone immediately; restrained ornament and plain, generous Barlow text keep the content readable. Original lava-cathedral and scarlet-eclipse covers belong to an invented visual world, rather than reproducing the mapsets' published artwork. The amber primary action gives the warm palette a clear navigation anchor.

Palette: canvas `#120C0D`, panel `#211719`, raised `#352527`, text `#F8EAD5`, muted `#D0BCA7`, focus `#FFD078`, decorative ember `#B8583F`, edge `#80533B`.

Display: **Pirata One Regular**, Rodrigo Fuenzalida and Nicolas Massi; body: **Barlow Regular**, The Barlow Project Authors. Both are unmodified SIL OFL 1.1 fonts, verified from the downloaded upstream notices in `fonts/`. See the pinned source links below. Display 30–48 px; body 22 px, metadata 20 px. Style-sheet swatch captions and concept-art plaques are supplementary 16–18 px text.

Slint implementation: bundle the wall and cover PNG tiles, crop the generated border into corner/edge pieces for a 9-slice frame, and use opaque fills for interiors. The mockup compositor uses a stretched source frame only at its thin perimeter; production should preserve corner scale with nine slices. One image per cover, no live blur, shadow, texture shader or glow. The original seal is a small PNG. Runtime budget is approximately 1.2 MB of cropped art plus 160 KB of fonts; exact packed sizes depend on PNG cropping and compression. Keep the OFL notices beside the font assets.

## Bastion Terminal

[Home](bastion-terminal/home.png) · [Play setup](bastion-terminal/play-setup-auger.png) · [Style sheet](bastion-terminal/style-sheet.png)

Military olive enamel, brass instrument edges and red signal lamps evoke the human machinery at the edge of an infernal world. Russo One contributes broad, industrial display lettering without imitating a franchise wordmark; familiar Inter keeps filenames and controls effortless to scan. Reactor and foundry imagery makes this the most technological option, with small rivets and restrained warning marks giving the panels material character. Compared with Cathedral, the mood is utilitarian and mechanical rather than ceremonial.

Palette: canvas `#10130F`, panel `#1D241D`, raised `#303A2C`, text `#F4F1DA`, muted `#C6CBB5`, focus `#FFD078`, decorative signal `#C36446`, edge `#74754B`.

Display: **Russo One Regular**, Jovanny Lemonad; body: **Inter Variable 4.1**, The Inter Project Authors. Both unmodified SIL OFL 1.1, confirmed against their bundled source notices. Display 30–48 px; body 22 px, metadata 20 px. Source and licence links are pinned below.

Slint implementation: olive wall PNG, cropped brass 9-slice border with rivets, opaque panel/row fills, simple one-pixel bevels and amber focus rectangles. Warning stripes and signal lamps remain baked into art. No runtime lighting. Estimated cropped art 1.2 MB plus 0.92 MB fonts; the larger variable Inter file can be replaced by upstream static weights in implementation after review. Preserve font licences and reserved names.

## Cinder 1993

[Home](cinder-1993/home.png) · [Play setup](cinder-1993/play-setup-auger.png) · [Style sheet](cinder-1993/style-sheet.png)

Warm umber masonry, oxidized copper and fiery pixel-painted vistas recall an early shooter menu without borrowing its lettering or assets. Bold condensed Teko titles have a baked light-and-shadow bevel; Barlow remains smooth and readable at handheld scale. This direction trades Cathedral's ornamental elegance and Bastion's engineered precision for earthy, tactile nostalgia. Square controls and pale amber focus preserve the production hierarchy while the coarse material treatment supplies the period character.

Palette: canvas `#17100B`, panel `#2A2016`, raised `#403221`, text `#FFF0CC`, muted `#D5C3A1`, focus `#FFD078`, display copper `#E18A63`, edge `#98734C`. Display bevel highlights `#F2B18A`; decorative shadow `#6F3423`.

Display: **Teko**, The Teko Project Authors, unmodified variable font rendered at weight 700; body: **Barlow Regular**, The Barlow Project Authors. Both SIL OFL 1.1, confirmed from source notices. Display 30–48 px; body 22 px, metadata 20 px. The font is a contemporary condensed face, not a DOOM-derived or game bitmap font. No font file is modified to create the bevel.

Slint implementation: brown wall PNG, copper frame cropped as a 9-slice, original emblem, three cover PNGs and opaque brown surfaces. Bake the decorative display bevel into title images if software-renderer text layering proves expensive; keep accessible/translatable title text separately. Prefer static upstream Teko Bold or a reviewed instance for production Slint font loading. No animation or palette shader. Approximate cropped art 1.3 MB plus 396 KB fonts; PNG compression and static font selection determine the final bundle.

## Verified font sources and embedding

The notices were fetched and read from Google Fonts revision `47d54ca1ebb14fd864695f1d39dc734f39017c33`; the exact unmodified font binaries and notices are in `fonts/`. Inter was copied from the repository's reviewed Inter 4.1 assets and its upstream notice was checked. **Confirmed:** each notice states SIL Open Font License 1.1; its permission clause permits embedding and redistribution with software. Fonts retain OFL and copyright notices, separate from DDL's GPL-3.0-or-later source licence; do not sell them alone or rename reserved font names after modifications.

| Face | Source / actual licence reviewed | Used by |
| --- | --- | --- |
| Pirata One | [Source](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/pirataone), [OFL](https://github.com/google/fonts/blob/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/pirataone/OFL.txt) | Cathedral display |
| Barlow | [Source](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/barlow), [OFL](https://github.com/google/fonts/blob/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/barlow/OFL.txt) | Cathedral / Cinder body |
| Russo One | [Source](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/russoone), [OFL](https://github.com/google/fonts/blob/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/russoone/OFL.txt) | Bastion display |
| Teko | [Source](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/teko), [OFL](https://github.com/google/fonts/blob/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/teko/OFL.txt) | Cinder display |
| Inter 4.1 | [Release](https://github.com/rsms/inter/releases/tag/v4.1), [OFL](https://github.com/rsms/inter/blob/v4.1/LICENSE.txt) | Bastion body |

## Measured contrast

Computed from opaque sRGB using WCAG relative luminance, before rounding. Every main body pair exceeds 4.5:1; display text exceeds 3:1. Decorative material edges and title shadow pixels carry no information and are not used as body-text colours. The bevel's readable face is the copper display pair below; the shadow is supplementary. Texture is excluded from every body text region.

| Direction | Text / panel | Muted / panel | Text / raised | Muted / raised | Canvas / focus button | Focus / canvas | Display / panel |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Ember Cathedral | 14.75:1 | 9.52:1 | 12.26:1 | 7.91:1 | 13.42:1 | 13.42:1 | 14.75:1 |
| Bastion Terminal | 13.95:1 | 9.54:1 | 10.43:1 | 7.14:1 | 12.96:1 | 12.96:1 | 13.95:1 |
| Cinder 1993 | 14.12:1 | 9.23:1 | 10.96:1 | 7.16:1 | 13.04:1 | 13.04:1 | 6.09:1 |

## Layers, provenance and reproduction

**Generated:** three original six-tile art atlases, one per direction, using the built-in image generation tool. Each includes wall material, empty frame, emblem and three separate mood covers for SIGIL, SIGIL II and AUGER;ZENITH. Complete final prompts are retained as `<direction>/prompt.txt`. No image reference input was supplied; no id/Bethesda artwork, UZDeck assets, DOOM wordmark or derived font was used. Source atlases were palette-compressed to 192 colours to meet the PNG budget. These are **Original concept art**, not actual map screenshots or licensed official covers.

**Composited:** every string, header tab, solid panel, row, bevel line, button, badge, focus ring and status label is drawn by `render.py` with the bundled fonts. Wall texture is darkened by a fixed precomposited blend; frames are decorative perimeters around opaque interiors. Tiny lower Home cover thumbnails are also original concept art, as are their main cover counterparts.

Reproduce offline with Python 3 and **Pillow 12.3.0** (design-time dependency only; no Cargo or runtime dependency):

```sh
python3 -m venv /tmp/ddl-visual-venv
/tmp/ddl-visual-venv/bin/pip install Pillow==12.3.0
/tmp/ddl-visual-venv/bin/python docs/design/visual-v2/render.py
sips -g pixelWidth -g pixelHeight docs/design/visual-v2/*/home.png \
  docs/design/visual-v2/*/play-setup-auger.png docs/design/visual-v2/*/style-sheet.png
```

The compositor uses the saved atlas tiles, not an online generation call. Its explicit tile seams match each source image. It asserts filename/control text fits and every output is below 2,000,000 bytes. Full command evidence, byte counts and regeneration checks are in [verification.md](verification.md).

Content: **Confirmed** mapset identity/base games/component names follow [D-010 / D-016](../../decisions.md), [SIGIL research](../../research/reference-sigil.md) and [AUGER upstream evidence](../../research/upstream-sources.md#5-reference-mapset--augerzenith-doomer-boards-project-37). These files link pinned upstream evidence. **Proposed:** complevel 2, explicitly not yet verified. DSDA-Doom 0.30.1 is an illustrative reviewed reference version, not a claim about the latest release. No compatibility claim is implied by artwork or a base-game-found label.

## Verification and remaining gates

R1: nine inspected 1280×800 renders with common layout constants; required content, numbered order and focus are present. R2: each style sheet has palette hex values, fonts, controls, focus, unavailable state, badges and materials. R3: original generated source atlases and prompts retained, no copied inputs; originality is reviewed through provenance and visual inspection rather than an automated copyright test. R4: actual font licence files verified and bundled. R5: ratios computed by the compositor's luminance functions and checked against thresholds; visible text uses solid surfaces. Manual inspection caught and corrected an unsupported check glyph, add-on help overlap and insufficient Teko display weight.

No production Slint code changed, so phase-B interaction/focus integration tests are intentionally deferred. `cargo test -p ddl` regenerates the existing offscreen production references for layout review; it does not implement these concepts. No visible windows were opened or screens captured. **Needs Deck Validation:** handheld legibility and software-renderer performance; neither can be established from these static mockups. A2 remains the owner’s selection; A3 and A4 remain later implementation / hardware work. No changes to protected source-of-truth documents are proposed by this task.

## Recommendation

Choose **Ember Cathedral**. Its carved oxblood architecture and bone blackletter give the strongest immediate infernal identity, while the fine bronze trim feels more deliberate and polished than a wall-texture backdrop alone. Bastion is the clearest industrial alternative; Cinder is the warmest nostalgic option. All three retain the same layout, plain body typography and unambiguous amber focus, so the owner can choose atmosphere without reopening the interaction design.
