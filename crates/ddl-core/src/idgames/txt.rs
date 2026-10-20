//! The author's same-stem `.txt` sidecar (spec 050, R2).
//!
//! Every `/idgames` upload must ship a `.TXT` outside (and inside) the zip with
//! the same name ([`README.INCOMING`](https://www.gamers.org/pub/idgames/README.INCOMING)).
//! DDL reads at most [`MAX_TXT_BYTES`] of it and keeps only facts: `Title`,
//! `Author`, the declared game, the engine requirement, `May Not Run With`,
//! `Primary purpose` and the map count. The description, credits and the rest
//! of the text are **not** stored anywhere in the index (R3).
//!
//! Two things make this parser deliberately forgiving, because a single bad
//! file must never fail a build (R2):
//!
//! - **Encoding.** Most files are ASCII or UTF-8; older ones use Windows-1252
//!   (or CP437, whose bytes overlap). Invalid UTF-8 is decoded as
//!   Windows-1252, with the five bytes that code page leaves undefined taken
//!   from CP437. Nothing is ever rejected for encoding.
//! - **Shape.** Fields are `Label: value`, sometimes starred or quoted, and a
//!   value may wrap onto an indented line. Free-form files with no labels at
//!   all simply yield no fields; the builder keeps the entry with a filename
//!   fallback and counts it.
//!
//! Port classification follows the evidence table in
//! `docs/research/idgames-browse.md` §6.4 and never scrapes free-text
//! `complevel`: a demo-recording hint is not a format requirement.

use super::index::{IndexGame, Port};

/// The most of a sidecar DDL reads (spec 050, R2): 64 KiB.
pub const MAX_TXT_BYTES: u64 = 64 * 1024;

/// The longest title, author or engine string kept from a sidecar.
const MAX_FIELD_CHARS: usize = 240;

/// The fields the index builder cares about (spec 050, R2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TxtFields {
    /// `Title:`.
    pub title: Option<String>,
    /// `Author:`.
    pub author: Option<String>,
    /// `Game:`.
    pub game: Option<String>,
    /// `Episode and Level #:`.
    pub episode_and_level: Option<String>,
    /// `Advanced engine needed:`.
    pub advanced_engine: Option<String>,
    /// `May Not Run With…:`.
    pub may_not_run_with: Option<String>,
    /// `Primary purpose:`.
    pub primary_purpose: Option<String>,
    /// `Map #:`.
    pub map_number: Option<String>,
}

/// What one sidecar contributes to an index entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TxtMeta {
    /// The author's title, when labelled.
    pub title: Option<String>,
    /// The author(s), when labelled.
    pub author: Option<String>,
    /// The base game the text declares, when it does.
    pub game: Option<IndexGame>,
    /// The port family the text requires, or [`Port::Unknown`].
    pub port: Port,
    /// The map count when the text states a simple range (`MAP01-MAP32`).
    pub maps: Option<u16>,
    /// Every recognised field, for tests and future UI detail.
    pub fields: TxtFields,
}

impl TxtMeta {
    /// Whether the text yielded anything useful at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.author.is_none()
            && self.game.is_none()
            && self.maps.is_none()
            && self.port == Port::Unknown
            && self.fields == TxtFields::default()
    }
}

/// Parses a sidecar body. Never fails and never returns an error for bad input.
#[must_use]
pub fn parse_txt(bytes: &[u8]) -> TxtMeta {
    let text = decode_text(bytes);
    let fields = collect_fields(&text);
    let game = classify_game(fields.game.as_deref(), fields.episode_and_level.as_deref());
    let port = classify_port(
        fields.advanced_engine.as_deref(),
        fields.may_not_run_with.as_deref(),
    );
    let maps = fields
        .map_number
        .as_deref()
        .and_then(parse_map_count)
        .or_else(|| {
            fields
                .episode_and_level
                .as_deref()
                .and_then(parse_map_count)
        });
    TxtMeta {
        title: fields.title.clone(),
        author: fields.author.clone(),
        game,
        port,
        maps,
        fields,
    }
}

/// Decodes sidecar bytes: UTF-8 when valid, else Windows-1252 (CP437 for the
/// five bytes Windows-1252 leaves undefined).
#[must_use]
pub fn decode_text(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.trim_start_matches('\u{feff}').to_owned();
    }
    bytes.iter().map(|byte| windows_1252(*byte)).collect()
}

/// One byte of Windows-1252. Bytes `0x80–0x9F` are the typographic block; the
/// five undefined slots fall back to their CP437 glyphs.
fn windows_1252(byte: u8) -> char {
    const HIGH: [char; 32] = [
        '\u{20ac}', '\u{00fc}', '\u{201a}', '\u{0192}', '\u{201e}', '\u{2026}', '\u{2020}',
        '\u{2021}', '\u{02c6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{00ec}',
        '\u{017d}', '\u{00c5}', '\u{00c9}', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}',
        '\u{2022}', '\u{2013}', '\u{2014}', '\u{02dc}', '\u{2122}', '\u{0161}', '\u{203a}',
        '\u{0153}', '\u{00a5}', '\u{017e}', '\u{0178}',
    ];
    match byte {
        0x80..=0x9f => HIGH[usize::from(byte - 0x80)],
        other => char::from(other),
    }
}

/// Which labelled field a line starts, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    Author,
    Game,
    Episode,
    Engine,
    MayNotRun,
    Purpose,
    MapNumber,
}

fn collect_fields(text: &str) -> TxtFields {
    let mut fields = TxtFields::default();
    let mut current: Option<Field> = None;
    // Sidecars are small, but bound the work anyway: a hostile file must not
    // make the builder quadratic.
    for line in text.lines().take(4_000) {
        let line = line.trim_end();
        if line.trim().is_empty() {
            current = None;
            continue;
        }
        if let Some((label, value)) = split_label(line) {
            match field_for(&label) {
                Some(field) => {
                    set_field(&mut fields, field, value);
                    current = Some(field);
                }
                None => current = None,
            }
        } else if let Some(field) = current {
            // A wrapped value: only indented lines continue the previous field.
            if line.starts_with([' ', '\t']) {
                append_value(&mut fields, field, line.trim());
            } else {
                current = None;
            }
        }
    }
    fields
}

/// Splits `*** Title: AUGER;ZENITH` into its label and value.
fn split_label(line: &str) -> Option<(String, &str)> {
    let line = line.trim_start();
    let line = line.trim_start_matches(['*', '#', '-', '_', '=', '"', '\'', ' ']);
    let (label, value) = line.split_once(':')?;
    let label = label.trim();
    if label.is_empty() || label.len() > 60 || label.contains('"') {
        return None;
    }
    Some((normalise_label(label), value.trim()))
}

fn normalise_label(label: &str) -> String {
    // Lowercase, collapse runs of whitespace and drop a trailing dot so
    // `Advanced engine needed...:` and `Advanced  engine needed:` agree.
    let mut normalised = String::with_capacity(label.len());
    let mut last_space = false;
    for character in label.chars() {
        if character.is_whitespace() {
            if !last_space {
                normalised.push(' ');
            }
            last_space = true;
        } else {
            normalised.push(character.to_ascii_lowercase());
            last_space = false;
        }
    }
    normalised.trim().trim_end_matches(['.', ' ']).to_owned()
}

fn field_for(label: &str) -> Option<Field> {
    match label {
        "title" => Some(Field::Title),
        "author" | "authors" | "author(s)" => Some(Field::Author),
        "game" | "iwad" | "base game" => Some(Field::Game),
        "episode and level #" | "episode and level" | "episode & level #" | "level #" => {
            Some(Field::Episode)
        }
        "primary purpose" | "primary purpose:" => Some(Field::Purpose),
        _ if label.starts_with("advanced engine needed")
            || label.starts_with("advanced engine") =>
        {
            Some(Field::Engine)
        }
        _ if label.starts_with("engine needed") || label.starts_with("advanced port needed") => {
            Some(Field::Engine)
        }
        _ if label.starts_with("may not run with") => Some(Field::MayNotRun),
        _ if label.starts_with("map #")
            || label.starts_with("map(s)")
            || label.starts_with("maps") =>
        {
            Some(Field::MapNumber)
        }
        _ => None,
    }
}

fn set_field(fields: &mut TxtFields, field: Field, value: &str) {
    let value = clean_value(value);
    if value.is_empty() {
        return;
    }
    let slot = match field {
        Field::Title => &mut fields.title,
        Field::Author => &mut fields.author,
        Field::Game => &mut fields.game,
        Field::Episode => &mut fields.episode_and_level,
        Field::Engine => &mut fields.advanced_engine,
        Field::MayNotRun => &mut fields.may_not_run_with,
        Field::Purpose => &mut fields.primary_purpose,
        Field::MapNumber => &mut fields.map_number,
    };
    if slot.is_none() {
        *slot = Some(value);
    }
}

fn append_value(fields: &mut TxtFields, field: Field, value: &str) {
    let value = clean_value(value);
    if value.is_empty() {
        return;
    }
    let slot = match field {
        Field::Title => &mut fields.title,
        Field::Author => &mut fields.author,
        Field::Game => &mut fields.game,
        Field::Episode => &mut fields.episode_and_level,
        Field::Engine => &mut fields.advanced_engine,
        Field::MayNotRun => &mut fields.may_not_run_with,
        Field::Purpose => &mut fields.primary_purpose,
        Field::MapNumber => &mut fields.map_number,
    };
    let Some(existing) = slot.as_mut() else {
        return;
    };
    if existing.chars().count() >= MAX_FIELD_CHARS {
        return;
    }
    existing.push(' ');
    existing.push_str(&value);
    truncate(existing);
}

fn clean_value(value: &str) -> String {
    let value = value.trim().trim_matches(['"', '\'', '*', ' ']);
    let mut cleaned = sanitize_text(value);
    truncate(&mut cleaned);
    cleaned
}

/// Makes untrusted sidecar text safe for the index: control characters are
/// dropped, tabs/newlines and other whitespace collapse to one space, the
/// result is trimmed and bounded to [`MAX_FIELD_CHARS`].
///
/// Validation stays strict (a control character in the published document is
/// still refused); this is the layer that stops an author's stray `0x1A`,
/// form feed or ANSI escape from making the whole build invalid (spec 050,
/// follow-up F1).
#[must_use]
pub fn sanitize_text(value: &str) -> String {
    let mut cleaned = String::with_capacity(value.len());
    let mut last_space = false;
    for character in value.chars() {
        if matches!(character, '\t' | '\n' | '\r')
            || (!character.is_control() && character.is_whitespace())
        {
            if !last_space {
                cleaned.push(' ');
                last_space = true;
            }
        } else if character.is_control() {
            // Drop C0/C1 controls (`\x0c`, `\x1a`, `\x1b`, U+0085, …).
        } else {
            cleaned.push(character);
            last_space = false;
        }
    }
    cleaned.trim().to_owned()
}

fn truncate(text: &mut String) {
    if text.chars().count() > MAX_FIELD_CHARS {
        let ended: String = text.chars().take(MAX_FIELD_CHARS).collect();
        *text = ended.trim_end().to_owned();
    }
}

/// The base game a sidecar declares, if it declares one (R2).
#[must_use]
pub fn classify_game(game: Option<&str>, episode: Option<&str>) -> Option<IndexGame> {
    for line in [game, episode].into_iter().flatten() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("plutonia") {
            return Some(IndexGame::Plutonia);
        }
        if lower.contains("tnt") || lower.contains("evilution") {
            return Some(IndexGame::Tnt);
        }
        if lower.contains("doom 2")
            || lower.contains("doom ii")
            || lower.contains("doom2")
            || lower.contains("map01")
            || lower.contains("map 01")
        {
            return Some(IndexGame::Doom2);
        }
        if lower.contains("ultimate")
            || lower.contains("doom")
            || lower.contains("e1m")
            || lower.contains("e2m")
            || lower.contains("e3m")
            || lower.contains("e4m")
        {
            return Some(IndexGame::Doom);
        }
    }
    None
}

/// The port family the text requires (research §6.4).
///
/// The result is a **hint**: `Other` means the author names a port DDL does not
/// ship, so the browse UI disables Download & Play. `Unknown` keeps the entry
/// playable-on-try, which is the documented default.
#[must_use]
pub fn classify_port(engine: Option<&str>, may_not_run_with: Option<&str>) -> Port {
    if let Some(blocked) = may_not_run_with {
        let lower = blocked.to_ascii_lowercase();
        if lower.contains("dsda") || lower.contains("prboom") {
            return Port::Other;
        }
    }
    let Some(engine) = engine else {
        return Port::Unknown;
    };
    let lower = engine.trim().to_ascii_lowercase();
    if lower.is_empty()
        || matches!(
            lower.as_str(),
            "none" | "none!" | "na" | "n/a" | "no" | "-" | "not required"
        )
    {
        return Port::Unknown;
    }
    if lower.contains("mbf21") || lower.contains("mbf 21") {
        return Port::Mbf21;
    }
    if lower.contains("mbf") || lower.contains("mbf-compatible") || lower.contains("mbf compatible")
    {
        return Port::Mbf;
    }
    if lower.contains("boom") {
        return Port::Boom;
    }
    if lower.contains("limit") {
        return Port::LimitRemoving;
    }
    if lower.contains("vanilla")
        || lower.contains("chocolate")
        || lower.contains("crispy")
        || lower.contains("doom retro")
        || lower.contains("doomretro")
        || lower.contains("doom.exe")
        || lower.contains("doom2.exe")
    {
        return Port::Vanilla;
    }
    if lower.contains("zdoom")
        || lower.contains("gzdoom")
        || lower.contains("lzdoom")
        || lower.contains("uzdoom")
        || lower.contains("udmf")
        || lower.contains("zscript")
        || lower.contains("decorate")
        || lower.contains("zandronum")
        || lower.contains("skulltag")
    {
        return Port::Zdoom;
    }
    if lower.contains("edge")
        || lower.contains("doomsday")
        || lower.contains("eternity")
        || lower.contains("k8vavoom")
        || lower.contains("legacy")
        || lower.contains("vavoom")
        || lower.contains("delphi")
    {
        return Port::Other;
    }
    if lower.contains("dsda") || lower.contains("prboom") || lower.contains("woof") {
        // The author names DDL's own engine or a close relative but no format;
        // that is not evidence of a requirement DDL cannot meet.
        return Port::Unknown;
    }
    Port::Other
}

/// The number of maps in a `Map #:` value, when it states one.
///
/// Handles `MAP01-MAP32`, `MAP01-32`, `01-11`, `MAP01, MAP05, MAP07`, a single
/// `MAP01` and a same-episode range such as `E1M1-E1M9`. A range spanning
/// episodes (`E1M1-E4M9`) is **not** counted: the archive text does not say how
/// many maps each episode has, and DDL does not guess. Free prose is ignored.
#[must_use]
pub fn parse_map_count(value: &str) -> Option<u16> {
    let lower = value.to_ascii_lowercase();
    if !lower.contains("map") && lower.contains('e') && lower.contains('m') {
        return parse_episode_range(&lower);
    }
    let mut numbers: Vec<(u32, usize, usize)> = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if index - start <= 4
                && let Ok(number) = value[start..index].parse::<u32>()
            {
                numbers.push((number, start, index));
            }
        } else {
            index += 1;
        }
    }
    if numbers.is_empty() {
        return None;
    }
    let mut total: u32 = 0;
    let mut cursor = 0;
    while cursor < numbers.len() {
        if cursor + 1 < numbers.len() {
            let (first, _, end) = numbers[cursor];
            let (second, start, _) = numbers[cursor + 1];
            let between = &value[end..start];
            if between.contains('-') && second > first && second - first <= 999 {
                total += second - first + 1;
                cursor += 2;
                continue;
            }
        }
        total += 1;
        cursor += 1;
    }
    if (1..=999).contains(&total) {
        u16::try_from(total).ok()
    } else {
        None
    }
}

/// `E1M1-E1M9` → 9. A range that crosses episodes is left unknown.
fn parse_episode_range(value: &str) -> Option<u16> {
    let bytes = value.as_bytes();
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'e' {
            index += 1;
            let (episode, next) = read_number(bytes, index)?;
            index = next;
            if bytes.get(index) != Some(&b'm') {
                index += 1;
                continue;
            }
            index += 1;
            let (map, next) = read_number(bytes, index)?;
            index = next;
            pairs.push((episode, map));
        } else {
            index += 1;
        }
    }
    match pairs.as_slice() {
        [(first_episode, first_map), (second_episode, second_map)]
            if first_episode == second_episode && second_map >= first_map =>
        {
            let count = second_map - first_map + 1;
            if (1..=999).contains(&count) {
                u16::try_from(count).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Reads a run of digits at `start`, returning the value and the next index.
fn read_number(bytes: &[u8], start: usize) -> Option<(u32, usize)> {
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == start || end - start > 4 {
        return None;
    }
    let text = std::str::from_utf8(&bytes[start..end]).ok()?;
    Some((text.parse().ok()?, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODERN: &str = "\
===========================================================================
Title                   : AUGER;ZENITH
Author                  : Doomer Boards Community
===========================================================================

Advanced engine needed  : MBF21+UMAPINFO
Primary purpose         : Single player
===========================================================================
Map #: 01-11
Game: Doom 2
May Not Run With...: Vanilla, Chocolate
";

    #[test]
    fn extracts_the_labelled_fields() {
        let meta = parse_txt(MODERN.as_bytes());
        assert_eq!(meta.title.as_deref(), Some("AUGER;ZENITH"));
        assert_eq!(meta.author.as_deref(), Some("Doomer Boards Community"));
        assert_eq!(meta.game, Some(IndexGame::Doom2));
        assert_eq!(meta.port, Port::Mbf21);
        assert_eq!(meta.maps, Some(11));
        assert_eq!(
            meta.fields.primary_purpose.as_deref(),
            Some("Single player")
        );
        assert_eq!(
            meta.fields.may_not_run_with.as_deref(),
            Some("Vanilla, Chocolate")
        );
    }

    #[test]
    fn starred_and_spaced_labels_are_recognised() {
        let text =
            "*** TITLE:  My Map\n# Author : Someone\nAdvanced  engine needed : GZDoom 4.7.1\n";
        let meta = parse_txt(text.as_bytes());
        assert_eq!(meta.title.as_deref(), Some("My Map"));
        assert_eq!(meta.author.as_deref(), Some("Someone"));
        assert_eq!(meta.port, Port::Zdoom);
    }

    #[test]
    fn wrapped_values_are_joined() {
        let text = "Advanced engine needed: GZDoom\n  3.2.0 and up\nTitle: X\n";
        let meta = parse_txt(text.as_bytes());
        assert_eq!(
            meta.fields.advanced_engine.as_deref(),
            Some("GZDoom 3.2.0 and up")
        );
        assert_eq!(meta.title.as_deref(), Some("X"));
    }

    #[test]
    fn control_characters_never_reach_a_field() {
        // A real sidecar had `0x1A` in its title and failed the whole build;
        // `0x0C`, `0x1B` and C1 NEL (U+0085) must be just as harmless.
        let text = "Title: Cinema\x1a 2\x0c \x1b Reloaded\nAuthor: A\u{85}B\x1bC\n";
        let meta = parse_txt(text.as_bytes());
        assert_eq!(meta.title.as_deref(), Some("Cinema 2 Reloaded"));
        assert_eq!(meta.author.as_deref(), Some("ABC"));
        assert!(
            meta.title
                .as_deref()
                .is_some_and(|title| !title.contains(char::is_control))
        );
    }

    #[test]
    fn whitespace_is_collapsed_and_fields_are_bounded() {
        let text = format!("Title:   A\t\tB  \nAuthor: {}\n", "x".repeat(600));
        let meta = parse_txt(text.as_bytes());
        assert_eq!(meta.title.as_deref(), Some("A B"));
        let author = meta.author.unwrap();
        assert_eq!(author.chars().count(), MAX_FIELD_CHARS);
        assert!(!author.ends_with(' '));
        assert_eq!(
            sanitize_text("  \u{a0} spaced \t text \r\n "),
            "spaced text"
        );
    }

    #[test]
    fn non_utf8_text_decodes_as_windows_1252() {
        // `M\xfcnchen` in Windows-1252/CP437: 0xFC is `ü`, not a UTF-8 continuation byte.
        let bytes = b"Title: M\xfcnchen\nAuthor: J\xf6rg\n";
        let meta = parse_txt(bytes);
        assert_eq!(meta.title.as_deref(), Some("München"));
        assert_eq!(meta.author.as_deref(), Some("Jörg"));
    }

    #[test]
    fn free_form_text_yields_nothing_instead_of_failing() {
        let meta = parse_txt(b"This is just prose about a map I made in 1996.\n");
        assert!(meta.is_empty());
        assert_eq!(meta.port, Port::Unknown);
        assert_eq!(meta.maps, None);
    }

    #[test]
    fn empty_and_binary_input_never_panics() {
        assert!(parse_txt(b"").is_empty());
        let meta = parse_txt(&[0u8, 1, 2, 0xff, 0xfe]);
        assert_eq!(meta.title, None);
    }

    #[test]
    fn chaotic_engine_lines_never_block_dsda_needlessly() {
        assert_eq!(classify_port(Some("None"), None), Port::Unknown);
        assert_eq!(classify_port(Some("Na"), None), Port::Unknown);
        assert_eq!(classify_port(None, None), Port::Unknown);
        assert_eq!(classify_port(Some("boom-compatible"), None), Port::Boom);
        assert_eq!(
            classify_port(Some("limit-removing (cl2)"), None),
            Port::LimitRemoving
        );
        assert_eq!(classify_port(Some("DSDA-Doom 0.30.0"), None), Port::Unknown);
        assert_eq!(classify_port(Some("PrBoom+"), None), Port::Boom);
        // The author says DDL's engine will not run it: block.
        assert_eq!(classify_port(Some("MBF21"), Some("DSDA-Doom")), Port::Other);
        // A vanilla-only complaint is not DSDA-hostile.
        assert_eq!(
            classify_port(Some("MBF21"), Some("Vanilla, Chocolate")),
            Port::Mbf21
        );
    }

    #[test]
    fn map_counts_come_from_ranges_and_lists() {
        assert_eq!(parse_map_count("MAP01-MAP32"), Some(32));
        assert_eq!(parse_map_count("MAP01-32"), Some(32));
        assert_eq!(parse_map_count("01-11"), Some(11));
        assert_eq!(parse_map_count("1 - 9"), Some(9));
        assert_eq!(parse_map_count("MAP07"), Some(1));
        assert_eq!(parse_map_count("MAP01, MAP05, MAP07"), Some(3));
        assert_eq!(parse_map_count("E1M1-E1M9"), Some(9));
        assert_eq!(parse_map_count("E1M1-E4M9"), None);
        assert_eq!(parse_map_count("as many as I felt like"), None);
        assert_eq!(parse_map_count("32-1"), Some(2));
    }

    #[test]
    fn game_lines_rank_the_specific_iwad_first() {
        assert_eq!(classify_game(Some("Doom 2"), None), Some(IndexGame::Doom2));
        assert_eq!(
            classify_game(Some("TNT: Evilution"), None),
            Some(IndexGame::Tnt)
        );
        assert_eq!(
            classify_game(Some("Doom 2, Plutonia"), None),
            Some(IndexGame::Plutonia)
        );
        assert_eq!(
            classify_game(None, Some("E1M1-E1M9")),
            Some(IndexGame::Doom)
        );
        assert_eq!(
            classify_game(None, Some("MAP01-MAP10")),
            Some(IndexGame::Doom2)
        );
        assert_eq!(classify_game(Some("Heretic"), None), None);
    }
}
