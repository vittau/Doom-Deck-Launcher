# Visual identity v2 — phase A

**Proposed**, for owner selection under spec 012 R1–R5 / A1. No production code or interaction behaviour changes. All three directions use the same production spec 010 skeleton: 104 px header; title/subtitle band; content at y=230–708; Home 784/384 px panels; setup 680/496 px panels; footer at y=736. Setup options scroll while the launch summary stays fixed. Continue and Play occupy the production coordinates and have one 4 px amber ring with a 4 px dark separation. Active tabs use an underline, selected options use a label and edge, neither is a second focus ring.

These are illustrative installed-library states, not evidence of an actual installation or gameplay verification. Home's spare panel area holds SIGIL II and AUGER;ZENITH context without changing the panel split. Setup depicts the exclusive add-on choice in the existing options column; None is selected, hence exactly one numbered content file. X/Y badges are demonstrated on each style sheet; production screen footers omit actions unavailable in that state. L1/R1 are root-only and Start opens Menu. No profile CRUD, favourites or reordering is introduced.

## Ember Cathedral

[Home](ember-cathedral/home.png) · [Play setup](ember-cathedral/play-setup-auger.png) · [Style sheet](ember-cathedral/style-sheet.png)

A carved oxblood wall and fine bronze borders make the launcher feel like an entrance to a hostile cathedral. Bone-coloured blackletter titles establish the infernal tone immediately; restrained ornament and plain, generous Barlow text keep the content readable. Original lava-cathedral and scarlet-eclipse covers belong to an invented visual world, rather than reproducing the mapsets' published artwork. The amber primary action gives the warm palette a clear navigation anchor.

Palette: canvas `#120C0D`, panel `#211719`, raised `#352527`, text `#F8EAD5`, muted `#D0BCA7`, focus `#FFD078`, decorative ember `#B8583F`, edge `#80533B`.

Display: **Pirata One Regular**, Rodrigo Fuenzalida and Nicolas Massi; body: **Barlow Regular**, The Barlow Project Authors. Both are unmodified SIL OFL 1.1 fonts, verified from the downloaded upstream notices in `fonts/`. See the pinned source links below. Display 30–48 px; body 22 px, metadata 20 px. Style-sheet swatch captions and concept-art plaques are supplementary 16–18 px text.

Slint implementation: bundle the wall and cover PNG tiles, crop the generated border into corner/edge pieces for a 9-slice frame, and use opaque fills for interiors. The mockup compositor uses a stretched source frame only at its thin perimeter; production should preserve corner scale with nine slices. One image per cover, no live blur, shadow, texture shader or glow. The original seal is a small PNG. Runtime budget is approximately 1.2 MB of cropped art plus 130 KB of fonts; exact packed sizes depend on PNG cropping and compression. Keep the OFL notices beside the font assets.
