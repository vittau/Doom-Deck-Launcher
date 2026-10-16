# Visual identity v2 — phase A

**Proposed**, for owner selection under spec 012 R1–R5 / A1. No production code or interaction behaviour changes. All three directions use the same production spec 010 skeleton: 104 px header; title/subtitle band; content at y=230–708; Home 784/384 px panels; setup 680/496 px panels; footer at y=736. Setup options scroll while the launch summary stays fixed. Continue and Play occupy the production coordinates and have one 4 px amber ring with a 4 px dark separation. Active tabs use an underline, selected options use a label and edge, neither is a second focus ring.

These are illustrative installed-library states, not evidence of an actual installation or gameplay verification. Home's spare panel area holds SIGIL II and AUGER;ZENITH context without changing the panel split. Setup depicts the exclusive add-on choice in the existing options column; None is selected, hence exactly one numbered content file. X/Y badges are demonstrated on each style sheet; production screen footers omit actions unavailable in that state. L1/R1 are root-only and Start opens Menu. No profile CRUD, favourites or reordering is introduced.

## Ember Cathedral

[Home](ember-cathedral/home.png) · [Play setup](ember-cathedral/play-setup-auger.png) · [Style sheet](ember-cathedral/style-sheet.png)

A carved oxblood wall and fine bronze borders make the launcher feel like an entrance to a hostile cathedral. Bone-coloured blackletter titles establish the infernal tone immediately; restrained ornament and plain, generous Barlow text keep the content readable. Original lava-cathedral and scarlet-eclipse covers belong to an invented visual world, rather than reproducing the mapsets' published artwork. The amber primary action gives the warm palette a clear navigation anchor.

Palette: canvas `#120C0D`, panel `#211719`, raised `#352527`, text `#F8EAD5`, muted `#D0BCA7`, focus `#FFD078`, decorative ember `#B8583F`, edge `#80533B`.

Display: **Pirata One Regular**, Rodrigo Fuenzalida and Nicolas Massi; body: **Barlow Regular**, The Barlow Project Authors. Both are unmodified SIL OFL 1.1 fonts, verified from the downloaded upstream notices in `fonts/`. See the pinned source links below. Display 30–48 px; body 22 px, metadata 20 px. Style-sheet swatch captions and concept-art plaques are supplementary 16–18 px text.

Slint implementation: bundle the wall and cover PNG tiles, crop the generated border into corner/edge pieces for a 9-slice frame, and use opaque fills for interiors. The mockup compositor uses a stretched source frame only at its thin perimeter; production should preserve corner scale with nine slices. One image per cover, no live blur, shadow, texture shader or glow. The original seal is a small PNG. Runtime budget is approximately 1.2 MB of cropped art plus 130 KB of fonts; exact packed sizes depend on PNG cropping and compression. Keep the OFL notices beside the font assets.

## Bastion Terminal

[Home](bastion-terminal/home.png) · [Play setup](bastion-terminal/play-setup-auger.png) · [Style sheet](bastion-terminal/style-sheet.png)

Military olive enamel, brass instrument edges and red signal lamps evoke the human machinery at the edge of an infernal world. Russo One contributes broad, industrial display lettering without imitating a franchise wordmark; familiar Inter keeps filenames and controls effortless to scan. Reactor and foundry imagery makes this the most technological option, with small rivets and restrained warning marks giving the panels material character. Compared with Cathedral, the mood is utilitarian and mechanical rather than ceremonial.

Palette: canvas `#10130F`, panel `#1D241D`, raised `#303A2C`, text `#F4F1DA`, muted `#C6CBB5`, focus `#FFD078`, decorative signal `#C36446`, edge `#74754B`.

Display: **Russo One Regular**, Jovanny Lemonad; body: **Inter Variable 4.1**, The Inter Project Authors. Both unmodified SIL OFL 1.1, confirmed against their bundled source notices. Display 30–48 px; body 22 px, metadata 20 px. Source and licence links are pinned below.

Slint implementation: olive wall PNG, cropped brass 9-slice border with rivets, opaque panel/row fills, simple one-pixel bevels and amber focus rectangles. Warning stripes and signal lamps remain baked into art. No runtime lighting. Estimated cropped art 1.2 MB plus 0.95 MB fonts; the larger variable Inter file can be replaced by upstream static weights in implementation after review. Preserve font licences and reserved names.

## Cinder 1993

[Home](cinder-1993/home.png) · [Play setup](cinder-1993/play-setup-auger.png) · [Style sheet](cinder-1993/style-sheet.png)

Warm umber masonry, oxidized copper and fiery pixel-painted vistas recall an early shooter menu without borrowing its lettering or assets. Bold condensed Teko titles have a baked light-and-shadow bevel; Barlow remains smooth and readable at handheld scale. This direction trades Cathedral's ornamental elegance and Bastion's engineered precision for earthy, tactile nostalgia. Square controls and pale amber focus preserve the production hierarchy while the coarse material treatment supplies the period character.

Palette: canvas `#17100B`, panel `#2A2016`, raised `#403221`, text `#FFF0CC`, muted `#D5C3A1`, focus `#FFD078`, display copper `#E18A63`, edge `#98734C`. Display bevel highlights `#F2B18A`; decorative shadow `#6F3423`.

Display: **Teko**, The Teko Project Authors, unmodified variable font rendered at weight 700; body: **Barlow Regular**, The Barlow Project Authors. Both SIL OFL 1.1, confirmed from source notices. Display 30–48 px; body 22 px, metadata 20 px. The font is a contemporary condensed face, not a DOOM-derived or game bitmap font. No font file is modified to create the bevel.

Slint implementation: brown wall PNG, copper frame cropped as a 9-slice, original emblem, three cover PNGs and opaque brown surfaces. Bake the decorative display bevel into title images if software-renderer text layering proves expensive; keep accessible/translatable title text separately. Prefer static upstream Teko Bold or a reviewed instance for production Slint font loading. No animation or palette shader. Approximate cropped art 1.3 MB plus 125 KB fonts; PNG compression and static font selection determine the final bundle.
