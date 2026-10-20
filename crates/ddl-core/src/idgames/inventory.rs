//! The archive file inventory (spec 050, R1).
//!
//! `/idgames` mirrors publish two archive-generated listings:
//!
//! - `ls-laR.gz` — a gzip of `ls -laR`, grouped by directory, with the lossy
//!   `ls` date (recent files print a time, no year);
//! - `fullsort.gz` — one `YYYY/MM/DD <size> <path>` record per line, newest
//!   first, with unambiguous dates but no directory grouping.
//!
//! Both are accepted ([`parse_inventory`]); `fullsort.gz` is the fallback when
//! the `ls` format changes. Only the two DOOM trees DDL can install survive
//! ([`keep_path`]): `levels/doom` and `levels/doom2`, never `/deathmatch/`,
//! `incoming/`, `newstuff/` or the non-DOOM games (heretic, hexen, strife,
//! hacx, doom64). Everything else is counted, not silently ignored.
//!
//! The listing is remote input, so a path is only kept when it is a plain
//! relative path of ordinary segments ([`safe_segment`]); traversal, absolute
//! paths, backslashes and control characters are rejected.

use std::io::Read as _;

use super::date::Date;

/// Largest compressed inventory DDL reads (the real `ls-laR.gz` is ~419 KB).
pub const MAX_INVENTORY_GZIP_BYTES: u64 = 8 * 1024 * 1024;

/// Largest decompressed inventory DDL reads (the real one is ~2.8 MB).
pub const MAX_INVENTORY_TEXT_BYTES: u64 = 32 * 1024 * 1024;

/// One `.zip` in the inventory, with the directory that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryEntry {
    /// Archive directory, e.g. `levels/doom2/Ports/megawads`.
    pub dir: String,
    /// File name without `.zip`, e.g. `dbp37_augzen`.
    pub stem: String,
    /// Size in bytes as the listing states it.
    pub size: u64,
    /// Archive date, day precision.
    pub date: Date,
}

impl InventoryEntry {
    /// The stable archive path with its `.zip` suffix (`levels/…/stem.zip`).
    #[must_use]
    pub fn zip_path(&self) -> String {
        format!("{}/{}.zip", self.dir, self.stem)
    }

    /// The same-stem sidecar `.txt` path (`levels/…/stem.txt`).
    #[must_use]
    pub fn txt_path(&self) -> String {
        format!("{}/{}.txt", self.dir, self.stem)
    }
}

/// Why an inventory line was dropped. Counts end up in the build report so a
/// format change is visible in CI instead of silently shrinking the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exclusion {
    /// Not a `.zip` (the `.txt` sidecars, `README`, checksums, …).
    NotAZip,
    /// A duplicate scratch tree (`incoming/`, `newstuff/`); `levels/` is durable.
    TransientTree,
    /// A `/deathmatch/` tree, which DDL does not install.
    Deathmatch,
    /// A game tree DDL cannot install (heretic, hexen, strife, hacx, doom64, …).
    OtherGame,
    /// A path that is not a safe relative path.
    UnsafePath,
    /// A line the format did not explain.
    Unparsed,
}

/// What [`parse_inventory`] saw.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InventoryReport {
    /// `.zip` entries kept, in listing order.
    pub kept: usize,
    /// Lines looked at.
    pub lines: usize,
    /// `(reason, count)` for every dropped zip or unexplained line.
    pub excluded: Vec<(Exclusion, usize)>,
}

impl InventoryReport {
    fn count(&mut self, reason: Exclusion) {
        if let Some((_, count)) = self.excluded.iter_mut().find(|(seen, _)| *seen == reason) {
            *count += 1;
        } else {
            self.excluded.push((reason, 1));
        }
    }

    /// The number dropped for `reason`.
    #[must_use]
    pub fn excluded_count(&self, reason: Exclusion) -> usize {
        self.excluded
            .iter()
            .find(|(seen, _)| *seen == reason)
            .map_or(0, |(_, count)| *count)
    }
}

/// A parsed inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Entries that passed the filter, sorted by path.
    pub entries: Vec<InventoryEntry>,
    /// What was seen and dropped.
    pub report: InventoryReport,
}

/// Decompresses an inventory file and parses it (spec 050, R1).
///
/// A gzip stream is required: the mirrors publish `.gz`, and refusing plain
/// text here keeps "did the download actually happen?" honest. Tests call
/// [`parse_inventory`] directly with text.
///
/// # Errors
///
/// Returns a human-readable reason when the file is not gzip, is larger than
/// [`MAX_INVENTORY_GZIP_BYTES`] or decompresses past
/// [`MAX_INVENTORY_TEXT_BYTES`].
pub fn parse_inventory_gzip(bytes: &[u8], today: Date) -> Result<Inventory, String> {
    let text = decode_inventory_gzip(bytes)?;
    Ok(parse_inventory(&text, today))
}

/// Decompresses an inventory file to text (spec 050, R1), without parsing it.
///
/// # Errors
///
/// See [`parse_inventory_gzip`].
pub fn decode_inventory_gzip(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() as u64 > MAX_INVENTORY_GZIP_BYTES {
        return Err(format!(
            "inventory is larger than {MAX_INVENTORY_GZIP_BYTES} compressed bytes"
        ));
    }
    let mut decoder = flate2::read::GzDecoder::new(bytes);
    let mut raw = Vec::new();
    decoder
        .by_ref()
        .take(MAX_INVENTORY_TEXT_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|error| format!("inventory is not a readable gzip stream: {error}"))?;
    if raw.len() as u64 > MAX_INVENTORY_TEXT_BYTES {
        return Err(format!(
            "inventory decompresses past {MAX_INVENTORY_TEXT_BYTES} bytes"
        ));
    }
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

/// Parses either listing format (spec 050, R1).
///
/// `today` is the day the listing was generated; it is only used to infer the
/// year of `ls` lines that print a time instead of a year.
#[must_use]
pub fn parse_inventory(text: &str, today: Date) -> Inventory {
    let mut report = InventoryReport::default();
    let mut entries = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    let fullsort = !lines.iter().any(|line| is_directory_header(line)) && lines.iter().any(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        is_fullsort_record(&fields)
    });

    if fullsort {
        for line in &lines {
            report.lines += 1;
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.is_empty() {
                continue;
            }
            if let Some(entry) = parse_fullsort_record(&fields, &mut report) {
                entries.push(entry);
            }
        }
    } else {
        let mut dir = String::new();
        for line in &lines {
            report.lines += 1;
            let trimmed = line.trim_end();
            if trimmed.is_empty() || trimmed.starts_with("total ") {
                continue;
            }
            if is_directory_header(trimmed) {
                dir = trimmed
                    .trim_end_matches(':')
                    .trim_start_matches("./")
                    .trim_start_matches('/')
                    .to_owned();
                continue;
            }
            if !trimmed.starts_with('-') {
                // `ls -laR` never prints a non-entry line other than a header,
                // `total` or blank; anything else is unexplained.
                report.count(Exclusion::Unparsed);
                continue;
            }
            if let Some(entry) = parse_ls_line(trimmed, &dir, today, &mut report) {
                entries.push(entry);
            }
        }
    }

    entries.sort_by(|a, b| (a.dir.as_str(), a.stem.as_str()).cmp(&(b.dir.as_str(), b.stem.as_str())));
    entries.dedup_by(|a, b| a.dir == b.dir && a.stem == b.stem);
    report.kept = entries.len();
    Inventory { entries, report }
}

fn is_directory_header(line: &str) -> bool {
    let line = line.trim();
    if !line.ends_with(':') {
        return false;
    }
    let header = line.trim_end_matches(':');
    header.starts_with("./") || (header.starts_with('/') && !header.contains(' '))
}

fn is_fullsort_record(fields: &[&str]) -> bool {
    fields.len() >= 3
        && fields[0].len() == 10
        && fields[0].as_bytes().get(4) == Some(&b'/')
        && fields[0].as_bytes().get(7) == Some(&b'/')
        && fields[1].bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_fullsort_record(fields: &[&str], report: &mut InventoryReport) -> Option<InventoryEntry> {
    if !is_fullsort_record(fields) {
        report.count(Exclusion::Unparsed);
        return None;
    }
    let date = Date::parse_slash_iso(fields[0])?;
    let size: u64 = fields[1].parse().ok()?;
    let path = fields[2];
    let (dir, name) = path.rsplit_once('/')?;
    keep_entry(dir, name, size, date, report)
}

fn parse_ls_line(
    line: &str,
    dir: &str,
    today: Date,
    report: &mut InventoryReport,
) -> Option<InventoryEntry> {
    // `-rw-rw-r-- 1 idgmaint idgames 12771737 Jul 26  2021 dbp37_augzen.txt`
    // Filenames may contain spaces, so everything after the eighth field is the
    // name (the archive policy says ASCII letters/digits/dash/underscore, but
    // old uploads predate it).
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 9 {
        report.count(Exclusion::Unparsed);
        return None;
    }
    let size: u64 = match fields[4].parse() {
        Ok(size) => size,
        Err(_) => {
            report.count(Exclusion::Unparsed);
            return None;
        }
    };
    let Some(date) = super::date::parse_ls_date(fields[5], fields[6], fields[7], today) else {
        report.count(Exclusion::Unparsed);
        return None;
    };
    let name = fields[8..].join(" ");
    keep_entry(dir, &name, size, date, report)
}

fn keep_entry(
    dir: &str,
    name: &str,
    size: u64,
    date: Date,
    report: &mut InventoryReport,
) -> Option<InventoryEntry> {
    if size == 0 || !safe_dir(dir) || !safe_segment(name) {
        report.count(Exclusion::UnsafePath);
        return None;
    }
    let Some(stem) = name.strip_suffix(".zip").or_else(|| name.strip_suffix(".ZIP")) else {
        report.count(Exclusion::NotAZip);
        return None;
    };
    if stem.is_empty() {
        report.count(Exclusion::NotAZip);
        return None;
    }
    match is_indexable_dir(dir) {
        true => {}
        false => {
            report.count(exclusion_for(dir));
            return None;
        }
    }
    Some(InventoryEntry {
        dir: dir.to_owned(),
        stem: stem.to_owned(),
        size,
        date,
    })
}

/// Whether a directory is one DDL indexes (spec 050, R1).
pub(crate) fn is_indexable_dir(dir: &str) -> bool {
    if dir == "incoming"
        || dir.starts_with("incoming/")
        || dir == "newstuff"
        || dir.starts_with("newstuff/")
    {
        return false;
    }
    if has_segment(dir, "deathmatch") {
        return false;
    }
    dir == "levels/doom"
        || dir.starts_with("levels/doom/")
        || dir == "levels/doom2"
        || dir.starts_with("levels/doom2/")
}

fn exclusion_for(dir: &str) -> Exclusion {
    if dir == "incoming" || dir.starts_with("incoming/") || dir == "newstuff" || dir.starts_with("newstuff/") {
        Exclusion::TransientTree
    } else if has_segment(dir, "deathmatch") {
        Exclusion::Deathmatch
    } else {
        Exclusion::OtherGame
    }
}

fn has_segment(dir: &str, wanted: &str) -> bool {
    dir.split('/').any(|segment| segment.eq_ignore_ascii_case(wanted))
}

/// A directory made only of ordinary segments.
fn safe_dir(dir: &str) -> bool {
    !dir.is_empty()
        && !dir.starts_with('/')
        && !dir.contains('\\')
        && dir.split('/').all(safe_segment)
}

/// A file or directory name DDL is willing to concatenate into a URL: no
/// separators, no traversal, no control characters, bounded length. Ordinary
/// spaces are allowed because old uploads predate the archive's naming policy;
/// [`super::fetch::Mirror::url`] percent-encodes them.
fn safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= 128
        && segment != "."
        && segment != ".."
        && segment == segment.trim()
        && segment.chars().all(|character| {
            (character.is_ascii_graphic() || character == ' ')
                && character != '/'
                && character != '\\'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::new(2026, 10, 20).unwrap()
    }

    const LS: &str = "\
./levels/doom2/Ports/megawads:
total 12
-rw-rw-r--  1 idgmaint idgames      6212 Jul 26  2021 dbp37_augzen.txt
-rw-rw-r--  1 idgmaint idgames  12771737 Jul 26  2021 dbp37_augzen.zip
-rw-rw-r--  1 idgmaint idgames   1234567 Oct 18 11:48 tistartan.zip
./levels/doom/Ports/a-c:
-rw-rw-r--  1 idgmaint idgames   900000 Jan  5  1999 something.zip
./levels/doom2/deathmatch/Ports/0-9:
-rw-rw-r--  1 idgmaint idgames   100000 Jan  5  1999 8bitdm.zip
./incoming/holding:
-rw-rw-r--  1 idgmaint idgames   100000 Oct 18 11:48 draft.zip
./newstuff:
-rw-rw-r--  1 idgmaint idgames   100000 Oct 18 11:48 copy.zip
./levels/heretic:
-rw-rw-r--  1 idgmaint idgames   100000 Oct 18 11:48 heresy.zip
./themes/xmas:
-rw-rw-r--  1 idgmaint idgames   100000 Oct 18 11:48 dbp19xmas2.zip
./levels/doom2/Ports/megawads/../../../etc:
-rw-rw-r--  1 idgmaint idgames   100000 Oct 18 11:48 escape.zip
";

    #[test]
    fn keeps_only_the_two_doom_level_trees() {
        let inventory = parse_inventory(LS, today());
        let paths: Vec<String> = inventory
            .entries
            .iter()
            .map(InventoryEntry::zip_path)
            .collect();
        assert_eq!(
            paths,
            vec![
                "levels/doom/Ports/a-c/something.zip".to_owned(),
                "levels/doom2/Ports/megawads/dbp37_augzen.zip".to_owned(),
                "levels/doom2/Ports/megawads/tistartan.zip".to_owned(),
            ]
        );
        assert_eq!(inventory.report.excluded_count(Exclusion::Deathmatch), 1);
        assert_eq!(inventory.report.excluded_count(Exclusion::TransientTree), 2);
        assert_eq!(inventory.report.excluded_count(Exclusion::OtherGame), 2);
        assert_eq!(inventory.report.excluded_count(Exclusion::UnsafePath), 1);
        assert_eq!(inventory.report.excluded_count(Exclusion::NotAZip), 1);
        assert_eq!(inventory.report.kept, 3);
    }

    #[test]
    fn recent_dates_get_their_year_inferred() {
        let inventory = parse_inventory(LS, today());
        let tistartan = inventory
            .entries
            .iter()
            .find(|entry| entry.stem == "tistartan")
            .unwrap();
        assert_eq!(tistartan.date.to_iso(), "2026-10-18");
        let augzen = inventory
            .entries
            .iter()
            .find(|entry| entry.stem == "dbp37_augzen")
            .unwrap();
        assert_eq!(augzen.date.to_iso(), "2021-07-26");
        assert_eq!(augzen.size, 12_771_737);
    }

    #[test]
    fn filenames_with_spaces_are_kept_whole() {
        let ls = "./levels/doom2/a-c:\n-rw-rw-r-- 1 idgmaint idgames 100 Oct 18 11:48 odd name.zip\n";
        let inventory = parse_inventory(ls, today());
        assert_eq!(inventory.entries.len(), 1);
        assert_eq!(inventory.entries[0].stem, "odd name");
    }

    #[test]
    fn fullsort_is_accepted_as_an_alternative_inventory() {
        let text = "\
2026/10/18   10370840  levels/doom2/Ports/megawads/tistartan.zip
2021/07/26   12771737  levels/doom2/Ports/megawads/dbp37_augzen.zip
2021/07/26   12771737  incoming/dbp37_augzen.zip
1993/02/03     260606  historic/doom0_2.zip
2021/07/26   12771737  levels/doom2/deathmatch/x.zip
2021/07/26   12771737  levels/hexen/x.zip
";
        let inventory = parse_inventory(text, today());
        let paths: Vec<String> = inventory
            .entries
            .iter()
            .map(InventoryEntry::zip_path)
            .collect();
        assert_eq!(
            paths,
            vec![
                "levels/doom2/Ports/megawads/dbp37_augzen.zip".to_owned(),
                "levels/doom2/Ports/megawads/tistartan.zip".to_owned(),
            ]
        );
        assert_eq!(
            inventory.entries[0].date,
            Date::new(2021, 7, 26).unwrap()
        );
        assert_eq!(inventory.report.excluded_count(Exclusion::TransientTree), 1);
        assert_eq!(inventory.report.excluded_count(Exclusion::OtherGame), 2);
        assert_eq!(inventory.report.excluded_count(Exclusion::Deathmatch), 1);
    }

    #[test]
    fn empty_input_is_not_an_error() {
        let inventory = parse_inventory("", today());
        assert!(inventory.entries.is_empty());
        assert_eq!(inventory.report.kept, 0);
    }

    #[test]
    fn unsafe_paths_never_reach_a_url() {
        for dir in ["/etc", "levels/..", "levels/doom2/../../x", "levels/do\nom2"] {
            assert!(!safe_dir(dir), "{dir:?} must be unsafe");
        }
        assert!(safe_dir("levels/doom2/Ports/megawads"));
        for name in ["", ".", "..", "a/b.zip", "a\\b.zip", "a\nb.zip"] {
            assert!(!safe_segment(name), "{name:?} must be unsafe");
        }
        assert!(safe_segment("dbp37_augzen.zip"));
    }
}
