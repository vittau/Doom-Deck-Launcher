# 076 — Explain normal and COMPAT SIGIL music choices

**Status:** Ready. **Origin:** owner finds REG and E3 labels confusing.

## Requirements

- R1: Use plain player-facing names that distinguish the normal SIGIL episode from the COMPAT
  edition. Make the normal MIDI edition, normal paid Buckethead replacement and COMPAT maps/
  paid soundtrack relationship clear without requiring knowledge of REG or E3 abbreviations.
- R2: Keep exact WAD filenames in import instructions/descriptions where they identify the file
  the player must select. Explain that DSDA-Doom supports the normal episode; COMPAT is the
  older-port compatibility alternative. Use already verified research in 068/reference-sigil;
  do not change component IDs, hashes, bytes, dependency rules, defaults or load order.
- R3: Inspect the actual options/import sheet headlessly for legibility and clipping. Test copy
  and retained component behavior with focused existing tests; keep filenames and paid-source
  provenance accurate. Fold final capture refresh into 074 after all visual fixes.

## Ownership and verification

Same UI follow-up worker as 074 may edit only player-facing SIGIL recipe labels/descriptions/
obtain copy and relevant UI strings/tests. No identity/compatibility/schema/audio behavior
changes. Other catalog worker has completed 073 and now performs read-only performance/catalog
research. Report docs/reports/076-sigil-edition-copy.md. Follow 074 verification and constraints.
