## Change

Describe the mapset/release and link its canonical source. For catalog additions,
follow [Add a mapset](../CONTRIBUTING.md#add-a-mapset).

## Verification evidence

- Tester, date and exact DSDA-Doom version:
- IWAD variant, complevel (alias, and whether the author's TXT / `COMPLVL` / UMAPINFO or the port family convention gave it) and selected components/load order:
- Evidence status (inspected / verified / partial / incompatible):
- Archive/member size and SHA-256 provenance:
- Known limitations and checks still needing a Steam Deck:
- Permission/credit text preserved, and image permission (if any):

## Checks

- [ ] `ddl catalog lint` passes (or `cargo run -p ddl -- catalog lint`).
- [ ] `ddl catalog lint --fetch` passes for changed recipes.
- [ ] Screenshots (if any) are JPEG, at most 1280×800 and 192 KiB: `scripts/lean-image.sh <original> crates/ddl/ui/assets/screenshots/<id>/01.jpg`.
- [ ] Gameplay claims are backed by actual testing; inspection is labeled honestly.
- [ ] Optional/exclusive/replacement choices and user-supplied imports are documented.
- [ ] No IWADs, paid downloads, unreviewed hosts, or executable archive content added.
