//! DSDA-Doom's two MIDI soundfont keys (spec 067, R5).
//!
//! An explicit Audio choice owns exactly [`MIDIPLAYER_KEY`] and
//! [`SOUNDFONT_KEY`]. Until the player makes one, DDL must not write either key
//! and must not call a manual backend/font "Default". This module is the pure
//! policy: rendering the two values and classifying what an existing config
//! already holds. The guarded write itself lives in the DSDA config manager.

use std::path::Path;

use super::{Choice, Error, config_path, invalid};
use crate::dsda_config::format::{Document, Value};

/// DSDA-Doom v0.30.1 string key for the MIDI backend
/// (`configuration.c:752-763`, default `"fluidsynth"`).
pub const MIDIPLAYER_KEY: &str = "snd_midiplayer";
/// DSDA-Doom v0.30.1 string key for an external SF2
/// (`flplayer.c:268-318`; an empty value uses the SNDFONT lump).
pub const SOUNDFONT_KEY: &str = "snd_soundfont";
/// The backend value used for both supported Audio choices.
pub const FLUIDSYNTH: &str = "fluidsynth";

/// The two values DDL writes for one explicit choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    /// Value for [`MIDIPLAYER_KEY`].
    pub midiplayer: String,
    /// Value for [`SOUNDFONT_KEY`]; empty means the engine's own soundfont.
    pub soundfont: String,
}

impl Keys {
    /// The engine's native configuration: FluidSynth with no external font.
    #[must_use]
    pub fn native() -> Self {
        Self {
            midiplayer: FLUIDSYNTH.to_owned(),
            soundfont: String::new(),
        }
    }
}

/// What DDL can honestly say about the engine's current audio configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Current {
    /// FluidSynth with no external font: DDL may label this **Default**.
    Default,
    /// A manually chosen backend or font. DDL preserves it and labels it as a
    /// custom engine setting, never as Default.
    Custom,
}

/// Classifies the audio keys in an existing `active.cfg`.
///
/// Missing keys mean DSDA's own defaults (`fluidsynth`, empty), which is the
/// same configuration as [`Current::Default`]. A non-string or unknown value is
/// treated as custom: DDL cannot prove it equals the native configuration.
#[must_use]
pub fn current(document: &Document) -> Current {
    let native_backend = match document.get(MIDIPLAYER_KEY) {
        None => true,
        Some(value) => value.as_str() == Some(FLUIDSYNTH),
    };
    let no_external_font = match document.get(SOUNDFONT_KEY) {
        None => true,
        Some(value) => value.as_str().is_some_and(str::is_empty),
    };
    if native_backend && no_external_font {
        Current::Default
    } else {
        Current::Custom
    }
}

/// The two values to write for an explicit choice.
///
/// `Arachno` requires the verified absolute font path, so a caller cannot write
/// a check-less path through this API. DSDA v0.30.1 has no string escape
/// grammar, so a path it cannot represent is rejected instead of damaged.
///
/// # Errors
///
/// Returns an error when `Arachno` has no font or the path is relative, not
/// UTF-8, or contains a quote/control character.
pub fn keys_for(choice: Choice, font: Option<&Path>) -> Result<Keys, Error> {
    match choice {
        Choice::Default => Ok(Keys::native()),
        Choice::Arachno => {
            let path = font.ok_or_else(|| invalid("Arachno is not installed"))?;
            Ok(Keys {
                midiplayer: FLUIDSYNTH.to_owned(),
                soundfont: config_path(path)?.to_owned(),
            })
        }
    }
}

/// Sets both audio values in `document`, preserving comments, unknown lines and
/// every unrelated setting in place.
///
/// DSDA reads the last occurrence of a key, so duplicate audio lines would beat
/// the explicit choice. Duplicates are collapsed into one canonical line.
pub fn apply(document: &mut Document, keys: &Keys) {
    put(document, MIDIPLAYER_KEY, &keys.midiplayer);
    put(document, SOUNDFONT_KEY, &keys.soundfont);
}

fn put(document: &mut Document, name: &str, value: &str) {
    let occurrences = document
        .entries()
        .filter(|entry| entry.name == name)
        .count();
    if occurrences > 1 {
        document.remove(name);
    }
    document.set(name, Value::Str(value.to_owned()));
}

/// Whether `document` already holds exactly the requested values, with no
/// duplicate line that could override them.
///
/// Used to avoid rewriting (and backing up) an unchanged choice before every
/// launch. A non-string value never matches, so applying repairs it.
#[must_use]
pub fn matches(document: &Document, keys: &Keys) -> bool {
    let single = |name: &str| {
        document
            .entries()
            .filter(|entry| entry.name == name)
            .count()
            == 1
    };
    single(MIDIPLAYER_KEY)
        && single(SOUNDFONT_KEY)
        && document.get(MIDIPLAYER_KEY).and_then(Value::as_str) == Some(keys.midiplayer.as_str())
        && document.get(SOUNDFONT_KEY).and_then(Value::as_str) == Some(keys.soundfont.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_means_fluidsynth_with_an_empty_font() {
        let keys = keys_for(Choice::Default, None).unwrap();
        assert_eq!(keys.midiplayer, "fluidsynth");
        assert_eq!(keys.soundfont, "");
    }

    #[test]
    fn arachno_requires_a_verified_absolute_font_and_keeps_spaces() {
        assert!(keys_for(Choice::Arachno, None).is_err());
        assert!(keys_for(Choice::Arachno, Some(Path::new("fonts/a.sf2"))).is_err());
        let keys = keys_for(
            Choice::Arachno,
            Some(Path::new("/data/ddl fonts/Arachno SoundFont.sf2")),
        )
        .unwrap();
        assert_eq!(keys.midiplayer, "fluidsynth");
        assert_eq!(keys.soundfont, "/data/ddl fonts/Arachno SoundFont.sf2");
        assert!(
            keys_for(Choice::Arachno, Some(Path::new("/data/a\"b.sf2"))).is_err(),
            "DSDA has no escape grammar"
        );
    }

    #[test]
    fn classifies_native_missing_and_custom_values() {
        assert_eq!(current(&Document::parse("")), Current::Default);
        assert_eq!(
            current(&Document::parse(
                "snd_midiplayer \"fluidsynth\"\nsnd_soundfont \"\"\n"
            )),
            Current::Default
        );
        assert_eq!(
            current(&Document::parse("snd_midiplayer \"fluidsynth\"\n")),
            Current::Default
        );
        assert_eq!(
            current(&Document::parse(
                "snd_midiplayer \"fluidsynth\"\nsnd_soundfont \"/x.sf2\"\n"
            )),
            Current::Custom
        );
        assert_eq!(
            current(&Document::parse(
                "snd_midiplayer \"sdl\"\nsnd_soundfont \"\"\n"
            )),
            Current::Custom
        );
        assert_eq!(
            current(&Document::parse("snd_midiplayer 5\n")),
            Current::Custom,
            "a non-string value is not provably native"
        );
    }

    #[test]
    fn applying_replaces_in_place_and_preserves_unrelated_settings() {
        let mut document = Document::parse(
            "# player comment\nuse_mouse 1\nsnd_midiplayer \"sdl\"\nunknown_future 7\n",
        );
        let keys = keys_for(Choice::Default, None).unwrap();
        apply(&mut document, &keys);
        assert_eq!(
            document.to_text(),
            "# player comment\nuse_mouse 1\nsnd_midiplayer \"fluidsynth\"\nunknown_future 7\nsnd_soundfont \"\"\n"
        );
        // Applying twice does not duplicate keys.
        let before = document.to_text();
        apply(&mut document, &keys);
        assert_eq!(document.to_text(), before);
    }

    #[test]
    fn matching_detects_only_exact_string_values() {
        let keys = keys_for(Choice::Default, None).unwrap();
        assert!(matches(
            &Document::parse("snd_midiplayer \"fluidsynth\"\nsnd_soundfont \"\"\n"),
            &keys
        ));
        assert!(!matches(&Document::parse(""), &keys));
        assert!(!matches(
            &Document::parse(
                "snd_midiplayer \"fluidsynth\"\nsnd_soundfont \"\"\nsnd_soundfont \"x\"\n"
            ),
            &keys
        ));
        assert!(!matches(
            &Document::parse("snd_midiplayer 5\nsnd_soundfont \"\"\n"),
            &keys
        ));
    }

    #[test]
    fn arachno_round_trips_through_the_engine_config_grammar() {
        let mut document = Document::parse("use_mouse 1\n");
        let keys = keys_for(
            Choice::Arachno,
            Some(Path::new("/data/my fonts/arachno.sf2")),
        )
        .unwrap();
        apply(&mut document, &keys);
        let text = document.to_text();
        assert!(
            text.contains("snd_soundfont \"/data/my fonts/arachno.sf2\"\n"),
            "{text}"
        );
        assert_eq!(current(&Document::parse(&text)), Current::Custom);
        assert_eq!(
            Document::parse(&text)
                .get(SOUNDFONT_KEY)
                .and_then(Value::as_str),
            Some("/data/my fonts/arachno.sf2")
        );
    }
}
