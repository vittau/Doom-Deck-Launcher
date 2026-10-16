//! Parser and serializer for DSDA-Doom's line-oriented configuration format
//! (spec 005, R1).
//!
//! The grammar is transcribed from DSDA-Doom v0.30.1
//! (`prboom2/src/m_misc.c:678–809`) and documented in
//! `docs/research/dsda-engine.md` §3–§4:
//!
//! - One `name value` pair per line; names are case-sensitive.
//! - Integers are read with C's `%i`: decimal, `0`-prefixed octal and `0x` hex.
//! - Strings are wrapped in double quotes; the reader removes the first and last
//!   character and does not process escapes.
//! - `#` starts a comment line. A line whose first non-space character is not
//!   alphanumeric is skipped by DSDA.
//! - Binding lines (`input_*`, except the scalar `input_profile`) use the
//!   three-profile grammar
//!   `action key[,key…] mouse joy | key[,key…] mouse joy | key[,key…] mouse joy`.
//!
//! Anything the parser does not recognize is kept verbatim as [`Line::Opaque`],
//! so an unknown or future key survives a parse/serialize round trip. Values are
//! re-emitted canonically (for example `0x10` becomes `16`), which preserves the
//! engine's semantics under `%i` while making values comparable for the
//! three-way merge in [`super::merge`].

use std::fmt::Write as _;

/// One line of a DSDA-Doom configuration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// An empty (or whitespace-only) line.
    Blank,
    /// A `#` comment line, kept exactly as written (including the `#`).
    Comment { raw: String },
    /// A parsed `name value` line.
    Entry(Entry),
    /// A line the parser did not recognize, kept verbatim so nothing is lost.
    Opaque { raw: String },
}

/// A `name value` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The key, case-sensitive.
    pub name: String,
    /// The parsed value.
    pub value: Value,
}

/// A parsed configuration value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// An integer, already decoded from decimal/octal/hex.
    Int(i64),
    /// A quoted string with the surrounding quotes removed.
    Str(String),
    /// A three-profile `input_*` binding.
    Binding(Binding),
    /// A value that is neither a quoted string nor a complete integer; kept
    /// verbatim.
    Raw(String),
}

impl Value {
    /// Renders the value as it must appear in the file (quotes re-added for
    /// strings, profiles joined for bindings).
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Self::Int(number) => number.to_string(),
            Self::Str(text) => format!("\"{text}\""),
            Self::Binding(binding) => binding.render(),
            Self::Raw(text) => text.clone(),
        }
    }

    /// Returns the integer when this is an [`Value::Int`].
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(number) => Some(*number),
            _ => None,
        }
    }

    /// Returns the string when this is a [`Value::Str`].
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(text) => Some(text),
            _ => None,
        }
    }
}

/// The three input profiles of an `input_*` binding line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Binding {
    /// One profile per segment; a DSDA-generated file always has three, but a
    /// file with fewer segments is preserved as written.
    pub profiles: Vec<BindingProfile>,
}

impl Binding {
    fn render(&self) -> String {
        self.profiles
            .iter()
            .map(BindingProfile::render)
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// A single input profile: keyboard keys, one mouse button, one pad button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingProfile {
    /// Keyboard key codes; `0` means "none".
    pub keys: Vec<i64>,
    /// Mouse button index; `-1` means "none".
    pub mouse: i64,
    /// Gamepad button index; `-1` means "none".
    pub joy: i64,
}

impl BindingProfile {
    fn render(&self) -> String {
        let keys = self
            .keys
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!("{keys} {} {}", self.mouse, self.joy)
    }
}

/// A parsed configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    lines: Vec<Line>,
}

impl Document {
    /// Parses `text`. Parsing never fails: an unrecognized line is preserved as
    /// [`Line::Opaque`].
    #[must_use]
    pub fn parse(text: &str) -> Self {
        if text.is_empty() {
            return Self::default();
        }
        let body = text.strip_suffix('\n').unwrap_or(text);
        let lines = body
            .split('\n')
            .map(|raw| classify(raw.strip_suffix('\r').unwrap_or(raw)))
            .collect();
        Self { lines }
    }

    /// Serializes the document. Every line ends in `\n`, including the last one.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for line in &self.lines {
            match line {
                Line::Blank => out.push('\n'),
                Line::Comment { raw } | Line::Opaque { raw } => {
                    out.push_str(raw);
                    out.push('\n');
                }
                Line::Entry(entry) => {
                    let _ = writeln!(out, "{} {}", entry.name, entry.value.render());
                }
            }
        }
        out
    }

    /// Returns the value stored under `name`, if any.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.entries()
            .find(|entry| entry.name == name)
            .map(|e| &e.value)
    }

    /// Iterates over the parsed entries in file order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.lines.iter().filter_map(|line| match line {
            Line::Entry(entry) => Some(entry),
            _ => None,
        })
    }

    /// Iterates over the key names in file order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries().map(|entry| entry.name.as_str())
    }

    /// Sets `name` to `value`, replacing the first existing entry in place or
    /// appending a new entry at the end. Comments and unknown lines are left
    /// untouched.
    pub fn set(&mut self, name: &str, value: Value) {
        for line in &mut self.lines {
            if let Line::Entry(entry) = line
                && entry.name == name
            {
                entry.value = value;
                return;
            }
        }
        self.lines.push(Line::Entry(Entry {
            name: name.to_owned(),
            value,
        }));
    }

    /// Removes every entry with `name`.
    pub fn remove(&mut self, name: &str) {
        self.lines.retain(|line| match line {
            Line::Entry(entry) => entry.name != name,
            _ => true,
        });
    }

    /// Whether the document has no lines at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Classifies a single physical line (without its newline).
fn classify(raw: &str) -> Line {
    let trimmed = raw.trim_start();
    let Some(first) = trimmed.chars().next() else {
        return Line::Blank;
    };
    if first == '#' {
        return Line::Comment {
            raw: raw.to_owned(),
        };
    }
    if !first.is_ascii_alphanumeric() {
        // DSDA skips such lines; keep them verbatim so DDL does not drop data.
        return Line::Opaque {
            raw: raw.to_owned(),
        };
    }
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let Some(name) = parts.next() else {
        return Line::Opaque {
            raw: raw.to_owned(),
        };
    };
    let rest = parts.next().map(str::trim).unwrap_or_default();
    if rest.is_empty() {
        return Line::Opaque {
            raw: raw.to_owned(),
        };
    }
    let value = parse_value(name, rest).unwrap_or_else(|| Value::Raw(rest.to_owned()));
    Line::Entry(Entry {
        name: name.to_owned(),
        value,
    })
}

/// Parses the value part of a `name value` line.
fn parse_value(name: &str, rest: &str) -> Option<Value> {
    // `input_profile` is a scalar, not a binding, and shares the prefix.
    if name != "input_profile" && name.starts_with("input_") {
        return parse_binding(rest).map(Value::Binding);
    }
    if rest.starts_with('"') {
        // DSDA removes the first and last character of the value without an
        // escaping parser (`m_misc.c`); mirror that exactly.
        let bytes = rest.as_bytes();
        if bytes.len() < 2 {
            return None;
        }
        return Some(Value::Str(rest[1..rest.len() - 1].to_owned()));
    }
    parse_dsda_int(rest).map(Value::Int)
}

/// Parses an integer with C `%i` semantics: an optional sign, then `0x` hex,
/// `0`-prefixed octal, or decimal. The whole token must be consumed, so a
/// generated value like `12abc` is treated as opaque rather than silently
/// truncated.
fn parse_dsda_int(text: &str) -> Option<i64> {
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if unsigned.is_empty() {
        return None;
    }
    let magnitude = if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        if hex.is_empty() {
            return None;
        }
        i64::from_str_radix(hex, 16).ok()?
    } else if unsigned.len() > 1 && unsigned.starts_with('0') {
        i64::from_str_radix(&unsigned[1..], 8).ok()?
    } else {
        unsigned.parse::<i64>().ok()?
    };
    Some(if negative { -magnitude } else { magnitude })
}

/// Parses the three-profile binding grammar.
fn parse_binding(rest: &str) -> Option<Binding> {
    let segments: Vec<&str> = rest.split('|').collect();
    if segments.is_empty() || segments.len() > 3 {
        return None;
    }
    let mut profiles = Vec::with_capacity(segments.len());
    for segment in segments {
        let fields: Vec<&str> = segment.split_whitespace().collect();
        if fields.len() != 3 {
            return None;
        }
        let keys = fields[0]
            .split(',')
            .map(parse_dsda_int)
            .collect::<Option<Vec<_>>>()?;
        let mouse = parse_dsda_int(fields[1])?;
        let joy = parse_dsda_int(fields[2])?;
        profiles.push(BindingProfile { keys, mouse, joy });
    }
    Some(Binding { profiles })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal DSDA-shaped file built from the researched defaults: comments,
    /// a quoted string, decimal/octal/hex integers, a binding line and an
    /// unknown key. Not copied from any real user file (spec 005, A1).
    const FIXTURE: &str = include_str!("../../tests/fixtures/dsda-default.cfg");

    #[test]
    fn parses_comments_strings_integers_and_bindings() {
        let doc = Document::parse(FIXTURE);

        assert_eq!(
            doc.get("use_game_controller"),
            Some(&Value::Int(1)),
            "decimal integer"
        );
        assert_eq!(doc.get("mouse_sensitivity_horiz"), Some(&Value::Int(10)));
        assert_eq!(doc.get("hex_style_value"), Some(&Value::Int(0x10)));
        assert_eq!(doc.get("octal_style_value"), Some(&Value::Int(0o10)));
        assert_eq!(doc.get("negative_value"), Some(&Value::Int(-1)));
        assert_eq!(
            doc.get("screenshot_dir"),
            Some(&Value::Str(String::new())),
            "an empty quoted string"
        );
        assert_eq!(
            doc.get("some_future_ddl_key"),
            Some(&Value::Raw("kept verbatim".to_owned())),
            "an unknown key must survive as an opaque value"
        );

        let binding = doc
            .get("input_fire")
            .expect("the fixture has an input_fire binding");
        let Value::Binding(binding) = binding else {
            panic!("input_fire must parse as a binding, got {binding:?}");
        };
        assert_eq!(binding.profiles.len(), 3);
        assert_eq!(binding.profiles[0].keys, vec![157, 306]);
        assert_eq!(binding.profiles[0].mouse, 0);
        assert_eq!(binding.profiles[0].joy, 22);
        assert_eq!(binding.profiles[2].keys, vec![0]);
    }

    #[test]
    fn input_profile_is_a_scalar_not_a_binding() {
        let doc = Document::parse("input_profile 2\n");
        assert_eq!(doc.get("input_profile"), Some(&Value::Int(2)));
    }

    #[test]
    fn comments_and_malformed_lines_survive_verbatim() {
        let doc = Document::parse("# a comment\n--- not a key\ninput_broken\n");
        assert_eq!(
            doc.to_text(),
            "# a comment\n--- not a key\ninput_broken\n",
            "opaque lines are preserved byte for byte"
        );
    }

    #[test]
    fn round_trip_preserves_semantic_content() {
        let first = Document::parse(FIXTURE);
        let second = Document::parse(&first.to_text());
        assert_eq!(first, second, "parse -> serialize -> parse is stable");
        assert_eq!(second.to_text(), first.to_text(), "serialization is stable");
    }

    #[test]
    fn serializer_normalizes_integer_spelling_without_changing_meaning() {
        let doc = Document::parse("a 0x10\nb 010\nc -1\n");
        assert_eq!(doc.to_text(), "a 16\nb 8\nc -1\n");
        // The normalized file still parses to the same values.
        assert_eq!(Document::parse(&doc.to_text()), doc);
    }

    #[test]
    fn integer_parser_matches_c_percent_i() {
        assert_eq!(parse_dsda_int("0"), Some(0));
        assert_eq!(parse_dsda_int("+42"), Some(42));
        assert_eq!(parse_dsda_int("-42"), Some(-42));
        assert_eq!(parse_dsda_int("0x1f"), Some(31));
        assert_eq!(parse_dsda_int("0X1F"), Some(31));
        assert_eq!(parse_dsda_int("-0x10"), Some(-16));
        assert_eq!(parse_dsda_int("0777"), Some(511));
        // Not a complete integer: kept opaque instead of being truncated.
        assert_eq!(parse_dsda_int("12abc"), None);
        assert_eq!(parse_dsda_int("08"), None);
        assert_eq!(parse_dsda_int(""), None);
        assert_eq!(parse_dsda_int("0x"), None);
    }

    #[test]
    fn set_replaces_in_place_and_appends_new_keys() {
        let mut doc = Document::parse("# header\nb 2\na 1\n");
        doc.set("a", Value::Int(9));
        doc.set("c", Value::Int(3));
        assert_eq!(doc.to_text(), "# header\nb 2\na 9\nc 3\n");
    }

    #[test]
    fn remove_drops_matching_entries_and_keeps_comments() {
        let mut doc = Document::parse("# header\na 1\nb 2\n");
        doc.remove("a");
        assert_eq!(doc.to_text(), "# header\nb 2\n");
    }

    #[test]
    fn crlf_input_is_normalized_to_lf() {
        let doc = Document::parse("a 1\r\nb 2\r\n");
        assert_eq!(doc.to_text(), "a 1\nb 2\n");
    }
}
