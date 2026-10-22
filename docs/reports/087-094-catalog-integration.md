# Catalog expansion and browsing integration — specs 087–094

**Status:** Integration in progress; final combined checks pending.
**Date:** 2026-10-22. **Coordinator branch:** `verify/087-pwads`.

## Result and scope

The requested scope is 118 curated recipes: the previous 92 plus 25 downloadable
mapsets and the player-supplied No Rest for the Living expansion. Linear Doom is
assessed separately and is not in the playable catalog.

Four supervised OpenCode workers used isolated Orca worktrees, with DeepSeek
V4.1 Flash OpenCode Go / high confirmed in each terminal. Existing conversations
handled additions and review fixes; no nested agents or fifth code worker were
started. The coordinator reviewed source claims, merged on the verification
branch, and independently checked downloaded archive/member identities.

| Worker | New mapsets | Detailed evidence |
| --- | --- | --- |
| A | Scythe II; Plutonia 2; TNT: Revilution; Going Down: Turbo; The 10x10 Project; Speed of Doom | [Research](../research/reference-087-a.md), [report](087-a.md) |
| B | Doom the Way id Did; No End in Sight; Doom II the Way id Did; Deathless; Sunder; REKKR | [Research](../research/reference-087-b.md), [report](087-b.md) |
| C | Arrival; Vanguard; Lunatic; Overboard; Headless Chicken; Micro Slaughter Community Project | [Research](../research/reference-087-c.md), [report](087-c.md) |
| D | UAC Ultra; Japanese Community Project; Stardate 20X6; Ozonia; Abandon | [Research](../research/reference-087-d.md), [report](087-d.md) |
| C, reused | Swim With The Whales; Haste | [Whales report](092-swim-with-the-whales.md), [Haste report](093-haste.md) |
| A, reused | No Rest for the Living, using the player's own file | [Research](../research/no-rest-for-the-living.md), [report](094-no-rest-for-the-living.md) |

## Review decisions and limits

- **Confirmed:** the coordinator independently read all 25 downloaded archives,
  recomputed SHA-256 and sizes, and matched every declared package/member pin.
  Credits/readmes travel with installs. Original Going Down remains separate
  from Going Down: Turbo. Plutonia 2 requires Plutonia; TNT: Revilution requires
  TNT. REKKR is the original free PWAD edition over the player's Ultimate Doom.
- **Confirmed:** the supplied `NERVE.WAD` is the BFG/XBLA PWAD variant: 3,819,855
  bytes, 108 lumps, nine maps; SHA-256
  `2d234de8700ee403375fda278e50ea17739b36703b3316e0429b05aca4458fa9`.
  It was read as data, never executed or committed. The recipe requires Doom II
  and an imported main component, with no download package. The Steam/KEX
  expansion variant has published identification evidence but no locally
  inspected SHA-256; imports retain the existing honest unknown-hash warning.
- **Confirmed:** `gimmick` has label Gimmick and vocabulary order 75, following
  Challenging. The 10x10 Project earns it for its ten-texture/ten-flat constraint.
  Linear Doom's intended classification is recorded in its assessment, with no
  unsupported playable entry or speculative additional assignments.
- **Confirmed:** Sunder is pinned to the author's V.2407 Dropbox build, with 22
  map slots. It is not the newer V.2512 OneDrive build. That limitation is
  explicit in its recipe/research; no host-policy expansion was made.
- **Confirmed:** Linear Doom's author publishes v1.2 as a 7z archive. Tested live
  ModDB URLs returned HTTP 403 and no usable direct file target was observed.
  Without archive bytes, DSDA compatibility cannot be established; the author's
  GZDoom test statement proves neither a dependency nor DSDA support. DDL's
  current extractor does not accept 7z. A local archive would allow inspection,
  but would not by itself solve automatic download or archive support. See the
  [source assessment](../research/linear-doom.md).
- **Needs Deck Validation:** actual gameplay, progression, music and total
  conversion behavior. Sunder/Abandon performance is unmeasured; no FPS or
  battery claim is made. Technical archive/engine facts stay in verification,
  while product descriptions explain the maps and experience.

## Browsing and compatibility

Spec 090 adds an optional validated original-release date with real year/month/
day precision. Discovery categories, Favorites and installed PWAD Library lists
use ascending chronology; favorites do not promote newer entries. Unknown dates
sort last, with deterministic title/id ties. Partial dates retain their precision
instead of fabricating January 1. Recent-play history keeps its own semantics.

Community Projects excludes recipes tagged Doomer Boards while retaining their
dedicated category, other earned tags and installed/profile state. The accurate
underlying community-projects tags are preserved.

The catalog capability gate must match the new application reader so older
clients retain their old catalog rather than silently dropping dated/import-only
recipes. No application release, remote catalog publication or deployment is
part of this local integration.

## Verification

Workers recorded passing focused offline tests, offline/fetch lint for every
downloadable addition, formatting and core Clippy. Screenshots and provenance
were reviewed from local image files without opening a visible application or
capturing the desktop. Source/archive audits are read-only and tests have no
network access.

Final combined formatting, workspace Clippy, workspace tests, complete catalog
lint, chronological metadata audit and required-import headless checks remain
pending while the last capability changes are integrated. Linux CI/e2e and a
physical Deck playthrough have not been run against these unpublished commits.

## Dependencies and remaining work

No new dependency or engine is introduced. Finish spec 090/094 implementation,
review its navigation/import tests, run the combined checks, update this report
with exact results, then advance local main and clean up settled workers.
