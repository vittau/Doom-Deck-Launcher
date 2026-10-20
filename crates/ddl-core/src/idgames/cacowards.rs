//! Cacoward winners, read from the spec 042 research table (spec 050, R7/R8).
//!
//! The relevance order and the `cacoward` ranking signal need to know which
//! archive paths won an award. That list already lives in
//! `docs/research/cacowards.md` (spec 042), so the builder reads it instead of
//! keeping a second copy that would drift. The parser is deliberately small:
//! it tracks the `### YYYY` edition headings and looks for `/idgames/<path>.zip`
//! links in the table rows beneath them.
//!
//! Special awards (Mordeth, Mockaward, …) are not in the per-edition tables and
//! are skipped. If the file is missing or unreadable the build simply runs
//! without Cacoward data; nothing fails (R7, "when present").

use std::collections::BTreeMap;

/// The largest research document DDL reads.
pub const MAX_CACOWARDS_BYTES: u64 = 512 * 1024;

/// Archive path (lowercase, with `.zip`) → award year.
pub type Cacowards = BTreeMap<String, u16>;

/// Parses the Cacowards research document.
///
/// Only paths under `levels/` are kept: those are the entries the index
/// contains, and the archive's `themes/`, `combos/` and `deathmatch/` trees are
/// out of scope (spec 050, R1).
#[must_use]
pub fn parse_cacowards(markdown: &str) -> Cacowards {
    let mut winners = Cacowards::new();
    let mut year: Option<u16> = None;
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            year = heading
                .split(|character: char| !character.is_ascii_digit())
                .find(|part| part.len() == 4)
                .and_then(|part| part.parse::<u16>().ok())
                .filter(|year| (1994..=2100).contains(year));
            continue;
        }
        if line.starts_with("## ") {
            // A new top-level section (for example "Special awards") has its
            // own Year column and no `### YYYY` headings.
            year = None;
            continue;
        }
        let Some(year) = year else {
            continue;
        };
        if !line.contains("/idgames/") {
            continue;
        }
        for path in idgames_paths(line) {
            if (path.starts_with("levels/doom/") || path.starts_with("levels/doom2/"))
                && !path.contains("/deathmatch/")
            {
                winners.insert(path, year);
            }
        }
    }
    winners
}

/// Every `/idgames/<path>.zip` reference on one markdown line.
fn idgames_paths(line: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("/idgames/") {
        rest = &rest[start + "/idgames/".len()..];
        let end = rest
            .find(|character: char| {
                matches!(
                    character,
                    ')' | '`' | ' ' | '\t' | '|' | ']' | '[' | '"' | '\''
                )
            })
            .unwrap_or(rest.len());
        let candidate = rest[..end].trim_end_matches('.');
        if let Some(path) = candidate.strip_suffix(".zip") {
            let path = path.trim_start_matches('/');
            if !path.is_empty() {
                paths.push(path.to_ascii_lowercase() + ".zip");
            }
        }
        rest = &rest[end..];
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "\
## Per-edition tables

### 2021 — 12 winners, 15 runner-ups

| Award | Title | /idgames |
| --- | --- | --- |
| Top Ten p1 | AUGER;ZENITH | [`levels/doom2/Ports/megawads/dbp37_augzen.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/dbp37_augzen.zip) |
| Runner-up | Other | not matched |

### 2024 — 12 winners, 14 runner-ups

| Top Twelve p4 | Eviternity 2 | [`levels/doom2/Ports/megawads/eviternityii.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/eviternityii.zip) |
| Multiplayer | 32in24-5 | [`levels/doom2/deathmatch/Ports/megawads/32in24-5.zip`](https://www.gamers.org/pub/idgames/levels/doom2/deathmatch/Ports/megawads/32in24-5.zip) |

## Special awards whose recipient is a mapset

| 2004 | Mordeth Award | Hellcore | [`levels/doom2/Ports/megawads/hellcore.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/hellcore.zip) |
";

    #[test]
    fn winners_are_keyed_by_archive_path_and_year() {
        let winners = parse_cacowards(DOC);
        assert_eq!(
            winners.get("levels/doom2/ports/megawads/dbp37_augzen.zip"),
            Some(&2021)
        );
        assert_eq!(
            winners.get("levels/doom2/ports/megawads/eviternityii.zip"),
            Some(&2024)
        );
    }

    #[test]
    fn deathmatch_and_special_awards_are_not_index_entries() {
        let winners = parse_cacowards(DOC);
        assert!(!winners.contains_key("levels/doom2/deathmatch/ports/megawads/32in24-5.zip"));
        assert!(!winners.contains_key("levels/doom2/ports/megawads/hellcore.zip"));
        assert_eq!(winners.len(), 2);
    }

    #[test]
    fn prose_without_tables_yields_nothing() {
        assert!(parse_cacowards("No awards here.\n").is_empty());
    }

    #[test]
    fn the_real_research_document_parses() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/research/cacowards.md"
        );
        if let Ok(text) = std::fs::read_to_string(path) {
            let winners = parse_cacowards(&text);
            // The known 2024 winner cited in the catalog.
            assert_eq!(
                winners.get("levels/doom2/ports/megawads/eviternityii.zip"),
                Some(&2024)
            );
            assert!(winners.len() > 100, "{} winners", winners.len());
        }
    }
}
