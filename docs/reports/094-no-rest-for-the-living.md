# 094 — No Rest for the Living (player-supplied expansion)

## Scope and result

Implemented spec 094 worker A: added `catalog/no-rest-for-the-living.toml`, an offline evidence test
and the source research for the player-supplied Doom II expansion. The recipe has zero packages and
one required, default-on `user_supplied` `main` maps component; it requires Doom II and never
downloads, bundles or redistributes `NERVE.WAD` or an IWAD. Final offline lint and embedding checks ran after the coordinator's capability signal and merging
worker D's zero-package/import work.

| | No Rest for the Living |
| --- | --- |
| Id | `no-rest-for-the-living` |
| Base game | Doom II; `variants = []` accepts every recognized Doom II IWAD (BFG, classic, French, Steam/KEX) |
| Components | one required/default-on `main`, `user_supplied = true`, one `maps` member, no package |
| Packages | none (metadata-only; no dummy package) |
| Imported BFG file | 3,819,855 bytes; SHA-256 `2d234de8…458fa9`; MD5 `967d5ae2…b3b0` |
| Steam/KEX variant | reviewed published 4,039,441 bytes / MD5 `23422eb4…a6f`; no published or inspected SHA-256 |
| Port family / complevel | limit-removing / `vanilla` (2); Needs Deck Validation |
| Tags | `cacowards`, `classics`, `episode` |
| Original public date | **2010-05-26**, day precision (XBLA release) |
| Artwork | one sourced in-game DoomWiki screenshot (`01.jpg`, 800×600, 80,715 B) with recorded provenance; fallback otherwise |

## Evidence

The supplied `/Users/vitor/Downloads/NERVE.WAD` was read only as data: `PWAD`, 108 lumps,
`MAP01`–`MAP09`, no `DEHACKED`/`COMPLVL`/`UMAPINFO`/`MAPINFO`, and the hashes above. It matches the
existing reviewed `nerve-bfg` row exactly. The KEX variant was **not** locally inspected; only the
published size/MD5 (and SHA-1/CRC-32) from the reviewed IWAD table are recorded, so the recipe does
not invent a KEX SHA-256 and does not label an unrecognized KEX import as verified.

`NERVE.WAD` is an expansion PWAD, not an IWAD. The recipe therefore uses Doom II as `base_game` and
loads the imported file with `-file`; DSDA-Doom's pinned v0.30.1 `D_AddFile` selects `pack_nerve` for
any filename ending `nerve.wad`, case-insensitively, so the imported uppercase filename is preserved
and works. The recipe's import instructions keep the `nerve.wad` filename.

`known_sha256` contains only the inspected BFG SHA-256. The existing import flow accepts an unknown
hash only with its warning behavior; a KEX import is therefore accepted as a player file but
reported as unrecognized, never as verified. The exact KEX size/MD5 and the warning limitation are
recorded in `docs/research/no-rest-for-the-living.md` and in the recipe's verification notes.

The 2010 Cacowards Top Ten p1 recognition is cited from the archived inventory; the recipe uses the
allowlisted Doomworld feature page
<https://www.doomworld.com/cacowards/2010/> and does not broaden hosts. The original public release
date is 2010-05-26 (day precision) from the XBLA Doom II release date and the archived DoomWiki
release page; later BFG/Classic Complete/Unity/KEX releases are variants, not the original date. The
full source notes are in `docs/research/no-rest-for-the-living.md`.

## Capability integration (worker D)

D's spec 094 R3/R4 capability is merged into the integration branch: zero-package lint now allows a
required user-supplied main when no downloadable member exists, and Setup exposes the required
import row without offering an off toggle. The recipe embeds and passes offline lint with the
rebuilt `ddl`; D still owns the production import/argv/headless journey checks. This worker did not
edit shared model/lint/runtime/UI or shared tests.

## Verification

Focused checks only (shared target, `CARGO_BUILD_JOBS=2`, `--test-threads=2`):

| Check | Result |
| --- | --- |
| `cargo test -p ddl-core --test catalog_094 -- --test-threads=2` | 5 passed, 0 failed |
| `cargo fmt --all --check` | clean |
| Offline catalog lint (rebuilt `ddl`) | `Catalog lint passed (offline).` |
| `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| Full workspace/Slint checks | not run here (coordinator-owned); no fetch lint because there are no downloads |

`crates/ddl-core/tests/catalog_094.rs` is the independent offline test. It pins the Doom II/variant
model, the single required user-supplied maps component, the BFG SHA-256, the preserved filename in
the obtain text, the zero-package shape, the KEX unknown-hash caveat, the tag set and the
Cacowards/Deck limitations. It does not import the commercial file or use it as a fixture; no
commercial bytes are committed. D's tests own the install/import/argv/headless journey.

## Artwork

One real in-game screenshot is committed at
`crates/ddl/ui/assets/screenshots/no-rest-for-the-living/01.jpg` (800×600 JPEG, 80,715 bytes),
converted from the archived DoomWiki map image `No_Rest_for_the_Living_Map01_Castle2.PNG` with
`scripts/lean-image.sh`. `SOURCES.toml` records the Wayback image URL, canonical map page, credit,
original size, retrieval date and the honest permission note (DoomWiki CC BY-SA 4.0 text; commercial
id/Nerve game imagery, bundled under D-024 and removed on request). No image was synthesized or
captured from a window.

## Open questions / Deck checks

- Confirm metadata-only install makes no network request and Play is blocked with a clear Import
  action until the required file exists.
- Confirm the required component is controller-importable/replaceable and has no off toggle that
  silently launches plain Doom II.
- Confirm Doom II + `-file` selects the NERVE mission and the nine-map progression; verify
  vanilla/2, audio and controller navigation.
- Confirm the Steam/KEX NERVE file is accepted over a KEX Doom II IWAD with the honest unknown-hash
  warning, and is not presented as verified.

Commit under this task; no push, tag, release, main merge, dependency, host or shared-code change.
