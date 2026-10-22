# 104 — Continuous browsing update publication

## Current state

Publication is being prepared; **0.0.14 is not yet claimed published**.
Reviewed continuous scrolling and favorite-first Library behavior was merged as
`724a99f` after the focused evidence in report102.

Screenshot work (spec103) is complete and integrated as `5e31a91`: **12 of the 26**
baseline recipes gained a verified in-game JPEG and **14 remain honestly unresolved**.
All 12 assets are within the asset bounds, reproduce byte-for-byte from their preserved
originals with `scripts/lean-image.sh`, and the merged tree passed one offline
`ddl catalog lint catalog/` (`Catalog lint passed (offline)`). The merge changed only the
12 recipe screenshot blocks, the new `crates/ddl/ui/assets/screenshots/<id>/` files,
spec103 status and report103 (38 files, no Rust/UI/version/workflow change). The integration
branch tree is identical to the verified task branch.

Workspace/lockfile package versions and checked-in AppStream now name `0.0.14`.
The intended new immutable prerelease is `v0.0.14-browse`; previous tags stay intact.

## Screenshot coverage

Added: caustic-industry, doom-ii-the-way-id-did, going-down-turbo,
hell-revealed-3-pt-1, jolly-junkyard, no-end-in-sight, plutonia-2, rekkr, scythe-ii,
terminal-voltage, the-10x10-project and tnt-revilution (1,323,209 bytes total).

Unresolved: deathless, headless-chicken, japanese-valentines, lunatic, memento-mori,
memento-mori-ii, micro-slaughter-community-project, mindblood-genesis,
santas-outback-bender, spaceballs-the-quest-for-more-dbp-maps, speed-of-doom, sunder,
tomatomania and vanguard. Exact links, candidates, dimensions and blockers are recorded
in report103; no image was generated, enlarged or substituted from another mapset.

## Verification and publication

Version/helper checks passed locally: 27 dependency-free Python helper tests,
the served-URL shell tests, XML parsing, workspace/lockfile/AppStream equality
at 0.0.14, `git diff --check`, and unmodified `appstreamcli validate --no-net`
(successful with one pedantic hint). The metainfo helper was run locally with
the UTC date 2026-10-22; the existing 0.0.13 and older releases were preserved.

Pending: integrated ordinary Linux CI, paired release/Flatpak runs, canonical provenance
and checksum checks, and the tag-specific saved-state fixture. The screenshot batch review
and its single offline catalog lint are complete. No exhaustive test replay or network e2e
is planned for this UI/asset/metadata update. Exact results will replace this preparation
record as they become available.

## Remaining physical checks

**Needs Deck Validation:** controller repeat/scroll feel, real trackpad/wheel/drag
inertia, actual Discover displayed version and upgrade preservation of the owner's
library/settings. See the CB checks in `docs/deck-testing.md`.

No dependencies or protected project status/decision/journal/overview files changed.
