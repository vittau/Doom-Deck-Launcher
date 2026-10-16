# High-fidelity visual pass

## Delivery

Six 1280×800 final-look concepts are in `visual/`; ten corrected layout references remain in `mockups/`. The UX contract stays intact, with §2 extended for the furnace/steel art direction and §3 corrected so B always means Back, does nothing at root, and never opens Menu. Start opens Menu. The ambiguous Native / DSDA header is removed from all wireframes and replaced by the product name in visual concepts.

## Generation and composition

**Model-generated:** the original three-panel art sheet [furnace-art-sheet.png](assets/furnace-art-sheet.png), generated using the built-in image_gen tool, not a CLI/API fallback. Furnace gateway, blue-black industrial megastructure and fractured obsidian ring are original abstract moods, not actual mapset artwork. No borrowed screenshots, sprites, logos or game textures. This single shared art sheet keeps the six composited screens visually consistent.

**Composited with Pillow:** all six UI screens, exact Inter typography, opaque information surfaces, steel edges, charcoal gradients, component switches, focused controls, bottom hints and DDL split-gate wordmark. No model-generated lettering is used. The wordmark combines original simple polygon brackets/beam with Inter; it is implementable as vector shapes plus text. Text sits on solid panels including the small Original concept art plaque. The owner explicitly authorized Pillow composition of real UI text over generated art.

Reproduction: install Pillow in an isolated environment, then run `python3 docs/design/render_mockups.py` and `python3 docs/design/render_visual.py`. The visual renderer loads the wireframe helpers and screen recipes without executing the wireframe program, then applies the final visual treatment; no network required. Inter 4.1 is bundled under SIL OFL 1.1 with its notice. No runtime dependencies added.

### Exact generation prompt

```text
Use case: ui-mockup / original art layers for a high-fidelity native handheld game launcher named DDL. Create a single landscape 1536x1024 production art sheet, split into THREE equal-width vertical artwork panels with clean straight boundaries and no gutters. No text, no lettering, no UI controls. These are original abstract cover concepts, not actual SIGIL or DOOM artwork. LEFT PANEL: imposing brutalist industrial gateway with nested angular black steel arches descending into a molten amber-red abyss, subtle volumetric orange light, ominous hellish technology, monumental geometric forms, no symbols. MIDDLE PANEL: abstract vertical megastructure at night, dense angular blue-black towers and narrow vermilion and pale cyan emissive channels, futuristic industrial city, no people. RIGHT PANEL: fractured obsidian ring suspended above a vast rust-red chasm, angular architectural ruins, amber sparks, dramatic deep shadows. Visually cohesive premium editorial videogame-launcher art direction, painterly cinematic 3D, crisp geometric silhouettes, restrained detail, charcoal/oxidized copper/amber, dark edges. No game sprites, monsters, weapons, recognizable game architecture, actual mapset art, franchise wordmarks, DOOM logo, id or Bethesda logos, watermarks or copyright text. High quality original artwork suitable for displaying inside cards of a Steam Deck 1280x800 UI. Text and controls will be composited separately with exact typography.
```

## Verification

All six visuals and ten re-rendered wireframes are exactly 1280×800. Renderer assertions check dimensions and <2,000,000 bytes for each visual (<1,500,000 for references). Final visuals use the same measured text/focus palette; worst-case raised-panel contrast remains 11.75:1 primary text, 7.40:1 secondary text and 8.79:1 focus. Copper is decorative only. Original concept art plaques use opaque #090D11 backing. Visual review through local image tools checked the six-screen contact sheet and setup screens; no visible app windows were opened and no screen captures were taken.

| Screen | Bytes |
| --- | ---: |
| [discover.png](visual/discover.png) | 288,064 |
| [download-install.png](visual/download-install.png) | 225,944 |
| [home.png](visual/home.png) | 170,363 |
| [mapset-detail.png](visual/mapset-detail.png) | 272,097 |
| [play-blocked-iwad.png](visual/play-blocked-iwad.png) | 122,349 |
| [play-setup-sigil.png](visual/play-setup-sigil.png) | 118,595 |

### Required visual sips output

`sips -g pixelWidth -g pixelHeight docs/design/visual/*.png`

```text
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/visual/discover.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/visual/download-install.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/visual/home.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/visual/mapset-detail.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/visual/play-blocked-iwad.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/ui-design/docs/design/visual/play-setup-sigil.png
  pixelWidth: 1280
  pixelHeight: 800
```

### Re-rendered wireframe sips output

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

## Remaining gates

No requested screens remain open. Physical Deck LCD readability and interaction validation are still required; artwork and raster composition cannot prove focus behavior or game execution. Existing research gates remain: paid SIGIL identities/compat exposure, AUGER;ZENITH add-on data and compatibility, engine/template evidence vocabulary and Deck checks for episodes/pistol start/hotplug/handoff. Original abstract art is concept-only and must not be mistaken for authentic mapset media.
