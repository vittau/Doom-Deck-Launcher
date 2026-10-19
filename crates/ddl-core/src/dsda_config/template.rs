//! The `steam-deck-default` baseline template (spec 005, R2).
//!
//! The template is hand-authored data, so it lives in the repository as TOML
//! (decision D-006) under `crates/ddl-core/templates/dsda/`. It is embedded in
//! the binary so provisioning can never fail because a data file went missing.
//!
//! Only the keys DDL intends to own may appear in the template: [`Values`]
//! denies unknown fields, so a typo in the file is a load error rather than a
//! silent claim on a key DSDA owns. Bindings are intentionally absent in v1
//! (spec 005, R2).

use std::fmt;

use serde::Deserialize;

use super::format::{Document, Value};

/// The template shipped with DDL.
pub const EMBEDDED: &str = include_str!("../../templates/dsda/steam-deck-default.toml");

/// A parsed DSDA-Doom configuration baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// Baseline revision; names `base/<template_version>.cfg` (R3, R5).
    pub template_version: String,
    /// Lowest DSDA-Doom version this baseline was checked against.
    pub dsda_version_min: String,
    /// Highest DSDA-Doom version this baseline was checked against.
    pub dsda_version_max: String,
    /// The scalar integer keys DDL owns, in template order.
    pub values: Vec<(String, i64)>,
    /// The scalar string keys DDL owns, in template order (spec 029, R4).
    pub strings: Vec<(String, String)>,
}

impl Template {
    /// Parses the embedded template that ships with DDL.
    ///
    /// # Errors
    /// Returns [`TemplateError`] if the embedded file is malformed.
    pub fn embedded() -> Result<Self, TemplateError> {
        Self::from_toml(EMBEDDED)
    }

    /// Parses a template from TOML.
    ///
    /// # Errors
    /// Returns [`TemplateError`] on invalid TOML, a missing required field, or
    /// an unknown field.
    pub fn from_toml(text: &str) -> Result<Self, TemplateError> {
        let file: TemplateFile = toml::from_str(text).map_err(TemplateError)?;
        Ok(Self {
            template_version: file.template_version,
            dsda_version_min: file.dsda_version_min,
            dsda_version_max: file.dsda_version_max,
            values: file.values.into_entries(),
            strings: file.strings.into_entries(),
        })
    }

    /// The template's values as a config document.
    #[must_use]
    pub fn document(&self) -> Document {
        let mut document = Document::default();
        for (name, value) in &self.values {
            document.set(name, Value::Int(*value));
        }
        for (name, value) in &self.strings {
            document.set(name, Value::Str(value.clone()));
        }
        document
    }

    /// The template rendered to DSDA's `name value` text.
    #[must_use]
    pub fn config_text(&self) -> String {
        self.document().to_text()
    }

    /// The template's integer value for `name`, if it owns that key.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<i64> {
        self.values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| *value)
    }

    /// The template's string value for `name`, if it owns that key.
    #[must_use]
    pub fn get_str(&self, name: &str) -> Option<&str> {
        self.strings
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The DSDA-Doom versions this baseline was checked against.
    #[must_use]
    pub fn version_range(&self) -> VersionRange {
        VersionRange {
            min: self.dsda_version_min.clone(),
            max: self.dsda_version_max.clone(),
        }
    }
}

/// The inclusive DSDA-Doom version range a template was checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRange {
    /// Lowest tested version.
    pub min: String,
    /// Highest tested version.
    pub max: String,
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.min == self.max {
            f.write_str(&self.min)
        } else {
            write!(f, "{}..={}", self.min, self.max)
        }
    }
}

/// Why a template could not be parsed.
#[derive(Debug)]
pub struct TemplateError(toml::de::Error);

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid DSDA config template: {}", self.0)
    }
}

impl std::error::Error for TemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Top-level shape of the template file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateFile {
    template_version: String,
    dsda_version_min: String,
    dsda_version_max: String,
    values: Values,
    #[serde(default)]
    strings: Strings,
}

/// Exactly the integer keys DDL owns. A new field here is a deliberate decision
/// to take ownership of another key from DSDA.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Values {
    use_game_controller: i64,
    use_mouse: i64,
    left_analog_deadzone: i64,
    right_analog_deadzone: i64,
    left_analog_sensitivity_x: i64,
    left_analog_sensitivity_y: i64,
    right_analog_sensitivity_x: i64,
    right_analog_sensitivity_y: i64,
    analog_look_acceleration: i64,
    dsda_mouse_acceleration: i64,
    mouse_sensitivity_horiz: i64,
    mouse_sensitivity_mlook: i64,
    swap_analogs: i64,
    invert_analog_look: i64,
    // --- Steam Deck video baseline (spec 029, R4) ---
    use_fullscreen: i64,
    exclusive_fullscreen: i64,
    render_aspect: i64,
    render_stretch_hud: i64,
}

/// Exactly the string keys DDL owns (spec 029, R4). Optional so a template
/// without any string key still parses.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Strings {
    screen_resolution: String,
}

impl Strings {
    fn into_entries(self) -> Vec<(String, String)> {
        if self.screen_resolution.is_empty() {
            Vec::new()
        } else {
            vec![("screen_resolution".to_owned(), self.screen_resolution)]
        }
    }
}

impl Values {
    fn into_entries(self) -> Vec<(String, i64)> {
        vec![
            ("use_game_controller".to_owned(), self.use_game_controller),
            ("use_mouse".to_owned(), self.use_mouse),
            ("left_analog_deadzone".to_owned(), self.left_analog_deadzone),
            (
                "right_analog_deadzone".to_owned(),
                self.right_analog_deadzone,
            ),
            (
                "left_analog_sensitivity_x".to_owned(),
                self.left_analog_sensitivity_x,
            ),
            (
                "left_analog_sensitivity_y".to_owned(),
                self.left_analog_sensitivity_y,
            ),
            (
                "right_analog_sensitivity_x".to_owned(),
                self.right_analog_sensitivity_x,
            ),
            (
                "right_analog_sensitivity_y".to_owned(),
                self.right_analog_sensitivity_y,
            ),
            (
                "analog_look_acceleration".to_owned(),
                self.analog_look_acceleration,
            ),
            (
                "dsda_mouse_acceleration".to_owned(),
                self.dsda_mouse_acceleration,
            ),
            (
                "mouse_sensitivity_horiz".to_owned(),
                self.mouse_sensitivity_horiz,
            ),
            (
                "mouse_sensitivity_mlook".to_owned(),
                self.mouse_sensitivity_mlook,
            ),
            ("swap_analogs".to_owned(), self.swap_analogs),
            ("invert_analog_look".to_owned(), self.invert_analog_look),
            ("use_fullscreen".to_owned(), self.use_fullscreen),
            ("exclusive_fullscreen".to_owned(), self.exclusive_fullscreen),
            ("render_aspect".to_owned(), self.render_aspect),
            ("render_stretch_hud".to_owned(), self.render_stretch_hud),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_template_parses_with_the_expected_version() {
        let template = Template::embedded().expect("embedded template must be valid");
        assert_eq!(template.template_version, "2");
        assert_eq!(
            template.version_range().to_string(),
            "0.30.1",
            "the range is a single tested version until Deck testing widens it"
        );
    }

    #[test]
    fn embedded_template_owns_the_required_keys() {
        let template = Template::embedded().expect("valid");
        assert_eq!(
            template.get("use_game_controller"),
            Some(1),
            "DSDA's default is 0; DDL turns the pad on"
        );
        assert_eq!(template.get("use_mouse"), Some(1));
        // Every analog/mouse value from OVERVIEW 8.1 must be present.
        for key in [
            "left_analog_deadzone",
            "right_analog_deadzone",
            "left_analog_sensitivity_x",
            "left_analog_sensitivity_y",
            "right_analog_sensitivity_x",
            "right_analog_sensitivity_y",
            "analog_look_acceleration",
            "dsda_mouse_acceleration",
            "mouse_sensitivity_horiz",
            "mouse_sensitivity_mlook",
            "swap_analogs",
            "invert_analog_look",
        ] {
            assert!(template.get(key).is_some(), "{key} is missing");
        }
    }

    /// A template TOML with every required integer key but no optional strings.
    const MINIMAL_VALUES: &str = concat!(
        "use_game_controller = 1\n",
        "use_mouse = 1\n",
        "left_analog_deadzone = 6556\n",
        "right_analog_deadzone = 6556\n",
        "left_analog_sensitivity_x = 100\n",
        "left_analog_sensitivity_y = 100\n",
        "right_analog_sensitivity_x = 1536\n",
        "right_analog_sensitivity_y = 768\n",
        "analog_look_acceleration = 0\n",
        "dsda_mouse_acceleration = 0\n",
        "mouse_sensitivity_horiz = 10\n",
        "mouse_sensitivity_mlook = 10\n",
        "swap_analogs = 0\n",
        "invert_analog_look = 0\n",
        "use_fullscreen = 1\n",
        "exclusive_fullscreen = 0\n",
        "render_aspect = 2\n",
        "render_stretch_hud = 2\n",
    );

    #[test]
    fn unknown_keys_are_rejected_so_typos_cannot_claim_ownership() {
        let error = Template::from_toml(&format!(
            "template_version = \"1\"\ndsda_version_min = \"0.30.1\"\ndsda_version_max = \"0.30.1\"\n[values]\n{MINIMAL_VALUES}typo_key = 3\n",
        ))
        .expect_err("an unknown value key must fail");
        assert!(error.to_string().contains("invalid DSDA config template"));
    }

    /// R4/A3: the string table is optional, so an older template without one
    /// still parses.
    #[test]
    fn strings_are_optional() {
        let template = Template::from_toml(&format!(
            "template_version = \"1\"\ndsda_version_min = \"0.30.1\"\ndsda_version_max = \"0.30.1\"\n[values]\n{MINIMAL_VALUES}",
        ))
        .expect("a template without [strings] parses");
        assert!(template.strings.is_empty());
        assert_eq!(template.get_str("screen_resolution"), None);
    }

    /// A3: every new Deck video key parses and sits in DSDA's documented range.
    #[test]
    fn deck_video_values_are_in_the_documented_ranges() {
        let template = Template::embedded().expect("valid");
        for (key, min, max) in [
            ("use_fullscreen", 0, 1),
            ("exclusive_fullscreen", 0, 1),
            ("render_aspect", 0, 4),
            ("render_stretch_hud", 0, 2),
        ] {
            let value = template
                .get(key)
                .unwrap_or_else(|| panic!("{key} is missing"));
            assert!(
                (min..=max).contains(&value),
                "{key} = {value} is outside DSDA's range {min}..={max}"
            );
        }
        assert_eq!(
            template.get("use_fullscreen"),
            Some(1),
            "desktop fullscreen"
        );
        assert_eq!(
            template.get("exclusive_fullscreen"),
            Some(0),
            "exclusive fullscreen would fight gamescope"
        );
        assert_eq!(template.get("render_aspect"), Some(2), "16:10 widescreen");
        assert_eq!(
            template.get("render_stretch_hud"),
            Some(2),
            "the status bar fills the panel width"
        );
        assert_eq!(
            template.get_str("screen_resolution"),
            Some("1280x800"),
            "the Deck panel native resolution"
        );
    }

    #[test]
    fn documented_values_are_not_deck_tuning() {
        // Guard against someone "calibrating" the shipped baseline without the
        // on-device evidence: these are the researched upstream defaults.
        let template = Template::embedded().expect("valid");
        assert_eq!(template.get("left_analog_deadzone"), Some(6556));
        assert_eq!(template.get("right_analog_sensitivity_x"), Some(1536));
        assert_eq!(template.get("right_analog_sensitivity_y"), Some(768));
        assert_eq!(template.get("mouse_sensitivity_horiz"), Some(10));
    }

    #[test]
    fn config_text_is_one_entry_per_owned_key() {
        let template = Template::embedded().expect("valid");
        let text = template.config_text();
        assert_eq!(
            text.lines().count(),
            template.values.len() + template.strings.len(),
            "no comments or stray lines are written into the active file"
        );
        assert!(text.contains("use_game_controller 1\n"));
        assert!(text.contains("screen_resolution \"1280x800\"\n"));
    }
}
