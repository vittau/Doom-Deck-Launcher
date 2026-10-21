//! The `steam-deck-default` baseline template (spec 005, R2; spec 034, R2).
//!
//! The template is hand-authored data, so it lives in the repository as TOML
//! (decision D-006) under `crates/ddl-core/templates/dsda/`. It is embedded in
//! the binary so provisioning can never fail because a data file went missing.
//!
//! Only the keys DDL intends to own may appear in the template: [`Values`] and
//! [`Strings`] deny unknown fields, and every binding action must be in
//! [`KNOWN_BINDING_ACTIONS`]. A typo in the file is therefore a load error
//! rather than a silent claim on a key DSDA owns.
//!
//! Bindings are written as complete `input_*` entries: the wanted keyboard,
//! mouse and gamepad codes plus the DSDA-Doom v0.30.1 `upstream` default they
//! replace. The upstream triple is not written to the active file; the
//! three-way merge in [`super::merge`] uses it to recognise a binding the
//! player never touched when a new template version starts owning that key.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use super::format::{Binding, BindingProfile, Document, Value};

/// The template shipped with DDL.
pub const EMBEDDED: &str = include_str!("../../templates/dsda/steam-deck-default.toml");

/// The input actions a template may bind.
///
/// This is the subset of the complete `input_defs` inventory in
/// `docs/research/dsda-engine.md` §4 that DDL owns. Adding an action here is a
/// deliberate decision to take that binding away from DSDA's menu.
/// [prboom2/src/m_misc.c:391-581](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L391-L581)
pub const KNOWN_BINDING_ACTIONS: &[&str] = &[
    "input_map",
    "input_map_zoomin",
    "input_map_zoomout",
    "input_autorun",
    "input_nextweapon",
    "input_prevweapon",
    "input_quicksave",
    "input_quickload",
    "input_reverse",
];

/// DSDA's valid keyboard codes: `1`–`511`, or `0` for "none"
/// [prboom2/src/dsda/input.h:25](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/input.h#L25).
const KEY_MAX_EXCLUSIVE: i64 = 512;
/// DSDA's valid mouse indices: `-1`–`7`
/// [prboom2/src/dsda/input.h:26](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/input.h#L26).
const MOUSE_MAX_EXCLUSIVE: i64 = 8;
/// DSDA's valid gamepad indices: `-1`–`22`
/// [prboom2/src/dsda/input.h:27](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/input.h#L27).
const JOY_MAX_EXCLUSIVE: i64 = 23;
/// DSDA has three input profiles
/// [prboom2/src/dsda/input.h:24](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/input.h#L24).
const PROFILE_COUNT: usize = 3;

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
    /// DSDA-Doom upstream defaults for value keys that are new to the template,
    /// sorted by key (spec 056, R3). The migration uses them to recognise an
    /// untouched new key exactly like a binding's `upstream` triple.
    pub upstream_values: Vec<(String, i64)>,
    /// DSDA-Doom upstream defaults for string keys that are new to the template
    /// (spec 064, R7), sorted by key. Same purpose as [`Self::upstream_values`]
    /// for keys such as `videomode`.
    pub upstream_strings: Vec<(String, String)>,
    /// The scalar string keys DDL owns, in template order (spec 029, R4).
    pub strings: Vec<(String, String)>,
    /// The `input_*` bindings DDL owns, in template order (spec 034, R2).
    pub bindings: Vec<TemplateBinding>,
}

/// One `input_*` binding the template owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateBinding {
    /// The DSDA action key, for example `input_map`.
    pub action: String,
    /// The complete binding DDL wants, applied to all three input profiles.
    pub wanted: Binding,
    /// The DSDA-Doom v0.30.1 default the wanted binding replaces.
    pub upstream: Binding,
}

impl TemplateBinding {
    /// The wanted gamepad button, when every profile shares the same one.
    #[must_use]
    pub fn wanted_joy(&self) -> Option<i64> {
        shared_joy(&self.wanted)
    }

    /// The upstream gamepad button, when every profile shares the same one.
    #[must_use]
    pub fn upstream_joy(&self) -> Option<i64> {
        shared_joy(&self.upstream)
    }
}

/// Returns the gamepad button shared by all profiles, if there is one.
fn shared_joy(binding: &Binding) -> Option<i64> {
    let first = binding.profiles.first()?.joy;
    binding
        .profiles
        .iter()
        .all(|profile| profile.joy == first)
        .then_some(first)
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
    /// Returns [`TemplateError`] on invalid TOML, a missing required field, an
    /// unknown field, an unknown binding action, or an out-of-range button code.
    pub fn from_toml(text: &str) -> Result<Self, TemplateError> {
        let file: TemplateFile = toml::from_str(text).map_err(TemplateError::Toml)?;
        let values = file.values.into_entries();
        for key in file.upstream_values.keys() {
            if !values.iter().any(|(owned, _)| owned == key) {
                return Err(TemplateError::Invalid(format!(
                    "upstream default `{key}` is not a value DDL owns"
                )));
            }
        }
        let strings = file.strings.into_entries();
        for key in file.upstream_strings.keys() {
            if !strings.iter().any(|(owned, _)| owned == key) {
                return Err(TemplateError::Invalid(format!(
                    "upstream string default `{key}` is not a string DDL owns"
                )));
            }
        }
        let bindings = file
            .bindings
            .into_iter()
            .map(TemplateBindingFile::into_binding)
            .collect::<Result<Vec<_>, _>>()?;
        for (index, binding) in bindings.iter().enumerate() {
            if bindings[..index]
                .iter()
                .any(|earlier| earlier.action == binding.action)
            {
                return Err(TemplateError::Invalid(format!(
                    "binding action `{}` appears more than once",
                    binding.action
                )));
            }
        }
        Ok(Self {
            template_version: file.template_version,
            dsda_version_min: file.dsda_version_min,
            dsda_version_max: file.dsda_version_max,
            values,
            upstream_values: file.upstream_values.into_iter().collect(),
            upstream_strings: file.upstream_strings.into_iter().collect(),
            strings,
            bindings,
        })
    }

    /// The template's keys as a config document.
    #[must_use]
    pub fn document(&self) -> Document {
        let mut document = Document::default();
        for (name, value) in &self.values {
            document.set(name, Value::Int(*value));
        }
        for (name, value) in &self.strings {
            document.set(name, Value::Str(value.clone()));
        }
        for binding in &self.bindings {
            document.set(&binding.action, Value::Binding(binding.wanted.clone()));
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

    /// The DSDA-Doom upstream default recorded for value key `name`, if the
    /// template records one (spec 056, R3).
    #[must_use]
    pub fn upstream_value(&self, name: &str) -> Option<i64> {
        self.upstream_values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| *value)
    }

    /// The DSDA-Doom upstream default recorded for string key `name`, if the
    /// template records one (spec 064, R7).
    #[must_use]
    pub fn upstream_string(&self, name: &str) -> Option<&str> {
        self.upstream_strings
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The template's binding for `action`, if it owns that key.
    #[must_use]
    pub fn binding(&self, action: &str) -> Option<&TemplateBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.action == action)
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
pub enum TemplateError {
    /// The TOML itself did not parse, or a required/known field was missing.
    Toml(toml::de::Error),
    /// The TOML parsed but a binding is not something DSDA could load.
    Invalid(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toml(err) => write!(f, "invalid DSDA config template: {err}"),
            Self::Invalid(reason) => write!(f, "invalid DSDA config template: {reason}"),
        }
    }
}

impl std::error::Error for TemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Toml(err) => Some(err),
            Self::Invalid(_) => None,
        }
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
    upstream_values: BTreeMap<String, i64>,
    #[serde(default)]
    upstream_strings: BTreeMap<String, String>,
    #[serde(default)]
    strings: Strings,
    #[serde(default)]
    bindings: Vec<TemplateBindingFile>,
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
    mouse_sensitivity_vert: i64,
    mouse_sensitivity_mlook: i64,
    swap_analogs: i64,
    invert_analog_look: i64,
    // --- Steam Deck video baseline (spec 029, R4) ---
    use_fullscreen: i64,
    exclusive_fullscreen: i64,
    render_aspect: i64,
    render_stretch_hud: i64,
    // --- Frame pacing (spec 056, R1) ---
    render_vsync: i64,
    uncapped_framerate: i64,
}

/// Exactly the string keys DDL owns (spec 029, R4; spec 064, R7). Optional so a
/// template without any string key still parses.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Strings {
    screen_resolution: String,
    /// DSDA's renderer selector: `"Software"` (upstream default) or
    /// `"OpenGL"` ([`i_video.c:1003-1013`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_video.c#L1003-L1013);
    /// menu labels in [`m_menu.c:3463-3468`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_menu.c#L3463-L3468)).
    #[serde(default)]
    videomode: String,
}

impl Strings {
    fn into_entries(self) -> Vec<(String, String)> {
        let mut entries = Vec::new();
        if !self.screen_resolution.is_empty() {
            entries.push(("screen_resolution".to_owned(), self.screen_resolution));
        }
        if !self.videomode.is_empty() {
            entries.push(("videomode".to_owned(), self.videomode));
        }
        entries
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
                "mouse_sensitivity_vert".to_owned(),
                self.mouse_sensitivity_vert,
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
            ("render_vsync".to_owned(), self.render_vsync),
            ("uncapped_framerate".to_owned(), self.uncapped_framerate),
        ]
    }
}

/// One `[[bindings]]` entry as written in the TOML file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateBindingFile {
    action: String,
    keys: Vec<i64>,
    mouse: i64,
    joy: i64,
    upstream: BindingFields,
}

/// The keyboard, mouse and gamepad codes of one binding.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingFields {
    keys: Vec<i64>,
    mouse: i64,
    joy: i64,
}

impl TemplateBindingFile {
    fn into_binding(self) -> Result<TemplateBinding, TemplateError> {
        if !KNOWN_BINDING_ACTIONS.contains(&self.action.as_str()) {
            return Err(TemplateError::Invalid(format!(
                "`{}` is not an input action DDL may bind",
                self.action
            )));
        }
        let wanted = binding_for(&self.action, "wanted", &self.keys, self.mouse, self.joy)?;
        let upstream = binding_for(
            &self.action,
            "upstream",
            &self.upstream.keys,
            self.upstream.mouse,
            self.upstream.joy,
        )?;
        Ok(TemplateBinding {
            action: self.action,
            wanted,
            upstream,
        })
    }
}

/// Validates one triple and expands it to DSDA's three input profiles.
fn binding_for(
    action: &str,
    field: &str,
    keys: &[i64],
    mouse: i64,
    joy: i64,
) -> Result<Binding, TemplateError> {
    if keys.is_empty() {
        return Err(TemplateError::Invalid(format!(
            "`{action}` has no {field} keys; use `[0]` for no keyboard binding"
        )));
    }
    for key in keys {
        if *key == 0 {
            if keys.len() > 1 {
                return Err(TemplateError::Invalid(format!(
                    "`{action}` {field} keys cannot mix `0` (none) with other codes"
                )));
            }
        } else if !(1..KEY_MAX_EXCLUSIVE).contains(key) {
            return Err(TemplateError::Invalid(format!(
                "`{action}` {field} key {key} is outside DSDA's 1..511 keyboard range"
            )));
        }
    }
    if !(-1..MOUSE_MAX_EXCLUSIVE).contains(&mouse) {
        return Err(TemplateError::Invalid(format!(
            "`{action}` {field} mouse {mouse} is outside DSDA's -1..7 range"
        )));
    }
    if !(-1..JOY_MAX_EXCLUSIVE).contains(&joy) {
        return Err(TemplateError::Invalid(format!(
            "`{action}` {field} gamepad button {joy} is outside DSDA's -1..22 range"
        )));
    }
    let profile = BindingProfile {
        keys: keys.to_vec(),
        mouse,
        joy,
    };
    Ok(Binding {
        profiles: (0..PROFILE_COUNT).map(|_| profile.clone()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_template_parses_with_the_expected_version() {
        let template = Template::embedded().expect("embedded template must be valid");
        assert_eq!(template.template_version, "5");
        assert_eq!(
            template.version_range().to_string(),
            "0.30.1",
            "the range is a single tested version until Deck testing widens it"
        );
        assert_eq!(
            template.bindings.len(),
            KNOWN_BINDING_ACTIONS.len(),
            "the embedded template binds every action DDL may own"
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
            "mouse_sensitivity_vert",
            "mouse_sensitivity_mlook",
            "swap_analogs",
            "invert_analog_look",
            "render_vsync",
            "uncapped_framerate",
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
        "mouse_sensitivity_vert = 1\n",
        "mouse_sensitivity_mlook = 10\n",
        "swap_analogs = 0\n",
        "invert_analog_look = 0\n",
        "use_fullscreen = 1\n",
        "exclusive_fullscreen = 0\n",
        "render_aspect = 0\n",
        "render_stretch_hud = 2\n",
        "render_vsync = 0\n",
        "uncapped_framerate = 1\n",
    );

    fn minimal_template(extra: &str) -> String {
        format!(
            "template_version = \"5\"\ndsda_version_min = \"0.30.1\"\ndsda_version_max = \"0.30.1\"\n[values]\n{MINIMAL_VALUES}{extra}",
        )
    }

    #[test]
    fn unknown_keys_are_rejected_so_typos_cannot_claim_ownership() {
        let error = Template::from_toml(&minimal_template("typo_key = 3\n"))
            .expect_err("an unknown value key must fail");
        assert!(error.to_string().contains("invalid DSDA config template"));
    }

    /// R4/A3: the string table is optional, so an older template without one
    /// still parses.
    #[test]
    fn strings_are_optional() {
        let template = Template::from_toml(&minimal_template("")).expect("parses");
        assert!(template.strings.is_empty());
        assert_eq!(template.get_str("screen_resolution"), None);
        assert_eq!(template.get_str("videomode"), None);
    }

    /// R7 (spec 064): the string upstream table is optional, must name an owned
    /// string, and records the renderer's own default.
    #[test]
    fn upstream_string_defaults_are_optional_but_validated() {
        let template = Template::from_toml(&minimal_template("")).expect("parses");
        assert!(template.upstream_strings.is_empty());
        assert_eq!(template.upstream_string("videomode"), None);

        let with_string = format!(
            "{}\n[strings]\nscreen_resolution = \"1280x800\"\nvideomode = \"OpenGL\"\n\n[upstream_strings]\nvideomode = \"Software\"\n",
            minimal_template("")
        );
        let template = Template::from_toml(&with_string).expect("parses");
        assert_eq!(template.get_str("videomode"), Some("OpenGL"));
        assert_eq!(template.upstream_string("videomode"), Some("Software"));

        let error = Template::from_toml(&format!(
            "{}\n[strings]\nscreen_resolution = \"1280x800\"\n\n[upstream_strings]\nvideomode = \"Software\"\n",
            minimal_template("")
        ))
        .expect_err("an upstream string key the template does not own must fail");
        assert!(
            error.to_string().contains("videomode"),
            "the error names the key: {error}"
        );
    }

    /// R3 (spec 056): upstream defaults are optional, and when present they must
    /// name a key the template actually owns.
    #[test]
    fn upstream_value_defaults_are_optional_but_validated() {
        let template = Template::from_toml(&minimal_template("")).expect("parses");
        assert!(template.upstream_values.is_empty());
        assert_eq!(template.upstream_value("render_vsync"), None);

        let template =
            Template::from_toml(&minimal_template("\n[upstream_values]\nrender_vsync = 0\n"))
                .expect("parses");
        assert_eq!(template.upstream_value("render_vsync"), Some(0));

        let error = Template::from_toml(&minimal_template("\n[upstream_values]\ntypo_key = 0\n"))
            .expect_err("an upstream key the template does not own must fail");
        assert!(
            error.to_string().contains("typo_key"),
            "the error names the key: {error}"
        );
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
            ("render_vsync", 0, 1),
            ("uncapped_framerate", 0, 1),
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
        assert_eq!(
            template.get("render_aspect"),
            Some(0),
            "Auto follows the display DDL detects"
        );
        assert_eq!(
            template.get("render_stretch_hud"),
            Some(2),
            "the status bar fills the panel width"
        );
        assert_eq!(
            template.get_str("screen_resolution"),
            Some("1280x800"),
            "the config fallback is still the Deck panel native resolution"
        );
        assert_eq!(
            template.get_str("videomode"),
            Some("OpenGL"),
            "hardware rendering is the default (D-037)"
        );
        assert_eq!(template.upstream_string("videomode"), Some("Software"));
    }

    /// R1 (spec 056): VSync is on, uncapped interpolation stays on, and no FPS
    /// limit key caps the frame rate. `dsda_fps_limit` is DSDA's own key with an
    /// upstream default of 0; the template must not claim it.
    #[test]
    fn v4_turns_on_vsync_without_capping_the_frame_rate() {
        let template = Template::embedded().expect("valid");
        assert_eq!(template.get("render_vsync"), Some(1), "VSync is on");
        assert_eq!(
            template.get("uncapped_framerate"),
            Some(1),
            "interpolated uncapped rendering stays on"
        );
        assert_eq!(
            template.get("dsda_fps_limit"),
            None,
            "neither `dsda_fps_limit` nor `dsda_background_fps_limit` may cap play"
        );
        assert!(template.get("dsda_background_fps_limit").is_none());
    }

    /// R2 (spec 056): the menu's "Horizontal Sensitivity" and "Vertical
    /// Sensitivity" are the mouse keys [m_menu.c:3565-3566], and both are 5.
    #[test]
    fn v4_sets_both_mouse_look_sensitivities_to_five() {
        let template = Template::embedded().expect("valid");
        assert_eq!(template.get("mouse_sensitivity_horiz"), Some(5));
        assert_eq!(template.get("mouse_sensitivity_vert"), Some(5));
        assert_eq!(
            template.get("mouse_sensitivity_mlook"),
            Some(10),
            "Free Look Sensitivity is a different setting and keeps its default"
        );
        assert_eq!(
            template.get("right_analog_sensitivity_x"),
            Some(1536),
            "the analog keys use a different scale and are untouched"
        );
        assert_eq!(template.get("right_analog_sensitivity_y"), Some(768));
    }

    /// R3 (spec 056): the value keys added in v4 record DSDA's own default so
    /// the migration can tell "untouched" from "the player changed it".
    #[test]
    fn v4_records_upstream_defaults_for_the_new_value_keys() {
        let template = Template::embedded().expect("valid");
        assert_eq!(template.upstream_value("render_vsync"), Some(0));
        assert_eq!(template.upstream_value("uncapped_framerate"), Some(1));
        assert_eq!(template.upstream_value("mouse_sensitivity_vert"), Some(1));
        assert_eq!(
            template.upstream_value("use_fullscreen"),
            None,
            "v2 keys were handled by their own migration and need no upstream record"
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
        assert_eq!(template.get("mouse_sensitivity_mlook"), Some(10));
        assert_eq!(
            template.get("mouse_sensitivity_horiz"),
            Some(5),
            "the 5/5 mouse look preset is a deliberate owner request, not calibration"
        );
    }

    #[test]
    fn config_text_is_one_entry_per_owned_key() {
        let template = Template::embedded().expect("valid");
        let text = template.config_text();
        assert_eq!(
            text.lines().count(),
            template.values.len() + template.strings.len() + template.bindings.len(),
            "no comments or stray lines are written into the active file"
        );
        assert!(text.contains("use_game_controller 1\n"));
        assert!(text.contains("screen_resolution \"1280x800\"\n"));
        assert!(text.contains("videomode \"OpenGL\"\n"));
        assert!(
            text.contains("input_map 9 -1 4 | 9 -1 4 | 9 -1 4\n"),
            "bindings are written for all three profiles:\n{text}"
        );
    }

    /// A2 (spec 034): every binding is a known action and every code is one
    /// DSDA's binding parser accepts.
    #[test]
    fn every_binding_is_a_known_action_with_valid_codes() {
        let template = Template::embedded().expect("valid");
        for binding in &template.bindings {
            assert!(
                KNOWN_BINDING_ACTIONS.contains(&binding.action.as_str()),
                "{} is not a known input action",
                binding.action
            );
            for (label, value) in [("wanted", &binding.wanted), ("upstream", &binding.upstream)] {
                assert_eq!(
                    value.profiles.len(),
                    PROFILE_COUNT,
                    "{} {label} must fill DSDA's three profiles",
                    binding.action
                );
                for profile in &value.profiles {
                    assert!(!profile.keys.is_empty());
                    for key in &profile.keys {
                        assert!(
                            *key == 0 || (1..KEY_MAX_EXCLUSIVE).contains(key),
                            "{} {label} key {key} is outside 0..511",
                            binding.action
                        );
                    }
                    assert!(
                        (-1..MOUSE_MAX_EXCLUSIVE).contains(&profile.mouse),
                        "{} {label} mouse {} is outside -1..7",
                        binding.action,
                        profile.mouse
                    );
                    assert!(
                        (-1..JOY_MAX_EXCLUSIVE).contains(&profile.joy),
                        "{} {label} joy {} is outside -1..22",
                        binding.action,
                        profile.joy
                    );
                }
            }
            assert_eq!(
                binding.wanted.profiles[0], binding.wanted.profiles[1],
                "{} applies to every profile",
                binding.action
            );
            assert_eq!(
                binding.wanted.profiles[1], binding.wanted.profiles[2],
                "{} applies to every profile",
                binding.action
            );
        }
    }

    /// R2 (spec 034): the shipped buttons are exactly the owner's table, and
    /// every binding keeps DSDA's keyboard default.
    #[test]
    fn deck_controller_layout_matches_the_owner_table() {
        let template = Template::embedded().expect("valid");
        for (action, joy, upstream_joy) in [
            ("input_map", 4, 21),
            ("input_map_zoomin", 11, -1),
            ("input_map_zoomout", 12, -1),
            ("input_autorun", 1, 7),
            ("input_nextweapon", 14, 3),
            ("input_prevweapon", 13, 2),
            ("input_quicksave", 9, -1),
            ("input_quickload", 10, -1),
            ("input_reverse", 2, 8),
        ] {
            let binding = template
                .binding(action)
                .unwrap_or_else(|| panic!("{action} is missing"));
            assert_eq!(binding.wanted_joy(), Some(joy), "{action} button");
            assert_eq!(
                binding.upstream_joy(),
                Some(upstream_joy),
                "{action} records DSDA's default"
            );
            assert_eq!(
                binding.wanted.profiles[0].keys, binding.upstream.profiles[0].keys,
                "{action} keeps DSDA's keyboard binding"
            );
            assert_eq!(
                binding.wanted.profiles[0].mouse, binding.upstream.profiles[0].mouse,
                "{action} keeps DSDA's mouse binding"
            );
        }
        assert!(
            template.binding("input_use").is_none(),
            "use/open stays on DSDA's default A binding; DDL does not own it"
        );
    }

    /// R1 (spec 034): the recorded upstream triples are DSDA v0.30.1's own
    /// defaults, so the migration can recognise an untouched binding.
    #[test]
    fn upstream_defaults_match_the_researched_source() {
        let template = Template::embedded().expect("valid");
        for (action, keys, mouse, joy) in [
            ("input_map", vec![9], -1, 21),
            ("input_map_zoomin", vec![61], -1, -1),
            ("input_map_zoomout", vec![45], -1, -1),
            ("input_autorun", vec![186], -1, 7),
            ("input_nextweapon", vec![0], -1, 3),
            ("input_prevweapon", vec![0], -1, 2),
            ("input_quicksave", vec![192], -1, -1),
            ("input_quickload", vec![195], -1, -1),
            ("input_reverse", vec![47], -1, 8),
        ] {
            let binding = template.binding(action).expect(action);
            let profile = &binding.upstream.profiles[0];
            assert_eq!(profile.keys, keys, "{action} upstream keys");
            assert_eq!(profile.mouse, mouse, "{action} upstream mouse");
            assert_eq!(profile.joy, joy, "{action} upstream joy");
        }
    }

    #[test]
    fn an_unknown_binding_action_is_rejected() {
        let error = Template::from_toml(&minimal_template(
            "[[bindings]]\naction = \"input_not_real\"\nkeys = [0]\nmouse = -1\njoy = 1\nupstream = { keys = [0], mouse = -1, joy = -1 }\n",
        ))
        .expect_err("unknown action");
        assert!(
            error.to_string().contains("input_not_real"),
            "the error names the action: {error}"
        );
    }

    #[test]
    fn out_of_range_button_codes_are_rejected() {
        let entry = |upstream| {
            minimal_template(&format!(
                "[[bindings]]\naction = \"input_map\"\nkeys = [9]\nmouse = -1\njoy = 4\nupstream = {{ {upstream} }}\n",
            ))
        };
        for (field, upstream) in [
            ("gamepad", "keys = [9], mouse = -1, joy = 23"),
            ("mouse", "keys = [9], mouse = 8, joy = 21"),
            ("keyboard", "keys = [512], mouse = -1, joy = 21"),
        ] {
            let error = Template::from_toml(&entry(upstream)).expect_err(field);
            assert!(
                error.to_string().contains("outside"),
                "{field} error: {error}"
            );
        }
    }

    #[test]
    fn duplicate_binding_actions_are_rejected() {
        let entry = "[[bindings]]\naction = \"input_map\"\nkeys = [9]\nmouse = -1\njoy = 4\nupstream = { keys = [9], mouse = -1, joy = 21 }\n";
        let error = Template::from_toml(&minimal_template(&format!("{entry}{entry}")))
            .expect_err("duplicate action");
        assert!(error.to_string().contains("more than once"), "{error}");
    }
}
