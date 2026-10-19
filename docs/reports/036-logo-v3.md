# 036 — Logo v3: a richer Rose Compass

## Changes

There are three v3 variants of the Rose Compass under `docs/design/logo-v3/`,
each with `emblem.svg` (full tier), `emblem-small.svg` (small tier),
`horizontal.svg` and a contact sheet. They come with a deterministic
`author.py`, a reproducible `export.py` with `--check`, a production-vs-v3
`comparison.png`, and a `README.md` that gives the strengths and risks of each
variant. Production (`docs/design/logo/`, `crates/ddl/ui/assets/logo/`, Steam
art) is untouched (R6). No Rust or Slint code changed.

| Variant | What it adds to D-023 |
| --- | --- |
| **A · Carved Rose** | Craft through carving, with flat fills only. It has a two-slope bronze moulding with rivets, a carved recess wall, a shaded round tracery ring with eight bosses, bevelled bone cardinals, detached bronze ordinals and a faceted ember gem. |
| **B · Stained Glass** (owner's pick, refined in round 2) | Colour through real leaded glass. A lead-came network cuts each bay into an inner quarry, a deep-amber pointed lancet and two ruby side panes. Faceted amber jewels sit in the ring lead, the frame is gradient metal, and the ember is a small faceted amber lozenge. |
| **C · Furnace Rose** | A carved stone plate, backlit. The compass star and oculus are cut through it, a furnace glow shines through, and it adds a bronze dial ring and a hot frame lip. |

All three share the following (R1):

- Every emblem begins with the exact production outer octagon, and `--check`
  enforces it.
- All keep the four cardinal points, the single central ember as the brightest
  element, and the D-023 DDL wordmark contours.
- The palette extends Ember Cathedral with bronze, stone and glass ramps and a
  hot core (`#FFF3D2`), and introduces no new hue families.
- There are no franchise marks, game imagery or franchise lettering.
- Original art is GPL-3.0-or-later. Sheet labels and mock copy are bundled OFL
  Barlow and Pirata One outlines; the marks themselves contain no font
  outlines.

**Author's recommendation:** B best answers "more detail and more colour" while
staying recognisable, and it is the most legible at 32 px. A is the safe
fallback, and C is the bold option. The README has the full strengths and
risks of each.

**Scale tiers (R3).** The full tier is for 128 px and up. The small tier is
for 32–64 px: the app and window icons, and the 56 px header. It keeps the
silhouette and drops the fine ornament. A's small tier reuses the production
lead and petal contours unchanged.

`horizontal.svg` keeps the production 330×128 frame, so it drops into the
154×56 header slot unchanged. It embeds the small tier. Its wordmark is now an
extruded solid (a seamless dark outline and a carved side) on a dark tablet
with octagonal corner cuts. The tablet matches the header colour, so it is
invisible in the app, and it keeps the letters from bleeding into light
grounds. The DDL is centred on the emblem.

## Verification

- **A1 (R2, R4).** The SVGs are in `docs/design/logo-v3/<variant>/`.
  `export.py --check` passes on the final tree and does the following:
  - accepts only the elements `svg`, `defs`, `g`, `path`, `linearGradient`,
    `radialGradient` and `stop`, through an attribute allowlist;
  - rejects any `url()` that is not a local gradient id, and any `data:` or
    `//` reference;
  - asserts the production silhouette for every emblem;
  - checks the dimensions of each PNG and its byte equality after a fresh
    render with resvg 0.47.0.

  Negative tests on scratch copies were all rejected: an injected `<image>`,
  `<text>`, `filter=`, `font-family=`, an external `url(http…)`, a `data:`
  URI, and a moved silhouette vertex. Re-running `author.py` gives a clean
  `git status`, so it is deterministic.

  Byte sizes:

  | Variant | `emblem.svg` | `emblem-small.svg` | `horizontal.svg` |
  | --- | ---: | ---: | ---: |
  | A | 5,979 | 1,630 | 2,906 |
  | B | 7,882 | 4,203 | 5,479 |
  | C | 6,255 | 1,510 | 3,027 |

  The production emblem is 814 B.
- **R2, filters and gradients.** No filters are used. The gradients were
  proven in the app's own renderer with a throwaway headless probe in the
  session scratchpad, not committed:
  - setup: Slint 1.18.1 `MinimalSoftwareWindow` (software renderer), built
    against this repository's `Cargo.lock` (resvg 0.48.1), with the app's
    `svg`/`image-decoders` support;
  - coverage: every emblem at 32, 56, 256 and 512 px, and every lock-up at
    165×64, 330×128 and 660×256;
  - comparison: pixel by pixel against resvg 0.47.0 on `#120C0D`;
  - result: the largest channel difference was 1/255 for the emblems and
    at most 1/255 for the final lock-ups (2/255 on an earlier revision), with
    no channel over 16. The lock-ups were re-run after every wordmark change. The production masters
    show the same rounding. I opened one Slint render to confirm it is not
    blank.
- **A2 (R3, R4).** The contact sheets were inspected at native scale. I read
  crops of the specimen panels, the lock-up band and the header mock, plus 6×
  nearest-neighbour enlargements of native 32, 48 and 64 px renders on dark
  and light. At 32 px:
  - **A** keeps the production silhouette: four separated bone petals, an
    ember with an amber core, and a bevelled bronze rim.
  - **B** reads clearest of all: a bone star on red glass with a bright
    centre.
  - **C** reads as an orange star with a white-hot centre on dark stone. It
    has the lowest contrast of the three but is legible on both grounds.

  The full tier at 32–64 px is shown as a reference, and it is visibly noisier,
  which is why the small tier exists. The header mock is 1280×800, with the
  slot geometry taken from `app-window.slint`. In it, the 56 px emblem and the
  28 px DDL read clearly for all three.

  Contact sheets (1440×2528) and the comparison:
  - `docs/design/logo-v3/carved-rose/contact-sheet.png` (282,808 B)
  - `docs/design/logo-v3/stained-glass/contact-sheet.png` (518,166 B)
  - `docs/design/logo-v3/furnace-rose/contact-sheet.png` (404,876 B)
  - `docs/design/logo-v3/comparison.png` (1200×1080, 151,435 B)
- **A3 (R5).** The iteration notes follow.

## Iteration notes (R5)

**Pass 1.** I critiqued the first renders at 512 px and at native 32/48/64:

1. In A and B, the cardinal points' dark leads overlapped the ordinal blades
   and chipped them into shards. Unlimited mitres also drove the lead tips
   into the frame groove.
2. In B, eight amber roundels competed with the ember, which breaks the
   single-focus rule.
3. In C, the star tips dissolved into the dark because the glow radius was too
   small, and the stone plate was flat.
4. In A's small tier, a dark centre diamond fused the four petals into one
   star. That lost D-023's "separated petals", and the ember dot was smaller
   than production's.
5. In B's small tier, specks of glass showed between the oculus lead and the
   petal bases at 64 px.

**Pass 2.** These changes answered the first critique:

- I detached the ordinals (base 40, tip 70) and capped mitres at 2.5×.
- I rebuilt A's tracery as a 16-sector shaded round moulding with bosses.
- B's roundels became ruby glass with a small highlight, and the lead around
  the oculus grew (r 27, and r 28 in the small tier).
- C got a wider glow, a softly lit stone gradient and chamfered openings.
- A's small tier switched to the production lead and petal contours, plus a
  bevel and an ember ring.

**Pass 3.** A second review found three problems, all in C:

- The dial ring crossed C's cardinal blades and split off loose tip fragments.
  I pulled the star inside the ring and put the bosses on the eight compass
  axes.
- C's 32 px star was dim. I brightened the small-tier glow and widened its
  blades.
- C's shadowed half-blades, black at 42%, went olive and muddy at 512 px. They
  are now deep red at 50%.

**Pass 4 (operator review of the WIP).** The operator flagged the wordmark on
light grounds: there were thin light seams between the outline and the carved
side, and the bone letters seemed to bleed into the light background.

- The seams came from overlapping shifted copies. Each letter is now an exact
  convex sweep with see-through counters, and the outline grew from 3 to
  3.5 units.
- To stop the bleeding, the DDL now sits on a dark tablet that has the
  octagon's 45° cuts and the header's own colour, so it is invisible in the
  app. The DDL is also centred vertically on the emblem.

The operator confirmed it looked better.

## Round 2: owner picked B (coordinator review)

The owner picked **B · Stained Glass** for one more round and kept the solid
DDL wordmark on the dark tablet. The coordinator asked for four changes:

1. **Read as real stained glass.** The 8 large gradient wedges are gone.
   `glass_panes()` builds a lead-came network: in each 45° bay, an inner
   quarry, a lead ring, a pointed lancet on the bisector and two side panes
   whose came runs to the octagon's corner. Cells are inset by half the came
   width, so the dark ground reads as continuous lead. Two glass hues: deep
   amber lancets and ruby or warm-red panes, each with its own radial
   gradient.
2. **Ring dots.** The ruby roundels are gone. Small faceted amber jewels
   (lit and shaded halves in a lead setting) now sit where the ring lead
   crosses the diagonal leads.
3. **Smaller ember.** The round oculus (r 21) became a faceted amber lozenge
   (r 13, lead r 18), back to the D-023 diamond. The bone points lead the
   read.
4. **Lighter on light grounds.** The emblem's outer dark rim is thinner (from
   4 to 2.5 units in the full tier and from 8 to 6 in the small tier). The DDL
   tablet now hugs the letters (about 8 units of margin, from 192×104 to
   182×90) with a wider gap to the emblem. The tablet is shared, so the A and
   C lock-ups changed the same way. The tablet stays `#120C0D`, so the header
   is unchanged and contrast on light is kept.

**Self-critique in round 2:**

- **First render.** The detached bronze ordinals, the jewels and the lancets
  crowded every bay, and a heavy dark disc surrounded the ember. I removed the
  ordinals, so B is back to D-023's four points and the eight lancets carry
  the eight-fold richness. I thinned the cames and brought the glass closer
  to the centre.
- **Jewels.** Placed at the foot of each lancet, they read as sparks stuck to
  it. I moved them to the lead crossings.
- **Small tier.** Alternating amber and ruby wedges made a pinwheel that did
  not match the full tier, and the ember shrank to a dot. I made it
  symmetric: ruby glass with one lancet per bay, and an ember of r 19. The
  first lancets were bright and turned the 32 px mark into a daisy, so I made
  them narrower and a deeper amber. The bone star now dominates at 32 and
  48 px.
- **Centre.** At 128 px, 16 inner quarries sent 16 leads converging on the
  ember. I merged them to one quarry per bay, which calmed the centre.

**Verification in round 2:** `export.py --check` passes. The Slint 1.18.1
software-renderer probe matched resvg 0.47.0 within 1/255 for the B emblems
at 32, 48, 56, 256 and 512 px and for the lock-up at 165×64, 330×128 and
660×256. I inspected the final B sheet at native scale on dark and light,
including 6× enlargements of 32, 48 and 64 px.

## Dependencies

No project dependencies were added. The design-time tools are the same as the
production logo work:

- resvg 0.47.0, installed under `/tmp/ddl-logo-tools`, for export;
- `fonttools==4.60.1` in a venv, for `author.py` only, to outline the bundled
  fonts for labels.

The Slint probe was a throwaway in the scratchpad.

## Open questions and proposals

- **Window and app icon:** `icon.png` is exported at 256 px from `emblem.svg`,
  and the window manager downsamples it to 32 or 48 px. When a variant is
  applied, I propose exporting `icon.png` from `emblem-small.svg`, or shipping
  32, 48 and 256 PNGs, so the small tier is what reaches the window icon.
  Steam icons at 256 and 512 can keep the full tier.
- **Wordmark treatment:** v3 gives the DDL wordmark an extruded solid and a
  dark tablet. The follow-up should decide whether `stacked.svg`,
  `full-name.svg` and the Steam and social compositions adopt them too.
- **Proposed decision entry** (for `docs/decisions.md`, after the owner picks):
  "Logo v3: Rose Compass <variant> (owner pick <date> from `design/logo-v3/`).
  It keeps the D-023 silhouette and wordmark, uses a small tier for 32–64 px,
  and lifts v2's no-gradient rule for identity masters (gradients allowed,
  filters not)."
- **Proposed status and journal line:** "Spec 036 done: three v3 Rose Compass
  variants are ready for owner review in `docs/design/logo-v3/`."

## Spec items left open

None. Applying a chosen variant to production is a follow-up task by design
(R6).
