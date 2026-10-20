//! Per-profile scalar overrides (spec 005, R7).
//!
//! A profile may change a small set of DSDA scalar settings for one launch
//! without writing them into the shared, DDL-owned active file. They are handed
//! to spec 003's `LaunchPlan` as transient `-assign key=value` tokens, which
//! DSDA applies for that run only (research `docs/research/dsda-engine.md` §3).
//!
//! The allowlist is the scalar key inventory from the research document §4.
//! Binding keys (`input_*`) are deliberately excluded because `-assign` targets
//! registered config values, not the separate binding table.

use std::fmt;

/// Scalar keys a profile may override, from `docs/research/dsda-engine.md` §4.
pub const ALLOWED_KEYS: &[&str] = &[
    // Player / compatibility toggles.
    "vanilla_keymap",
    "input_profile",
    "movement_vertmouse",
    "allow_freelook",
    "autorun",
    // Mouse.
    "use_mouse",
    "mouse_menu_navigation",
    "mouse_sensitivity_horiz",
    "mouse_sensitivity_vert",
    "dsda_mouse_acceleration",
    "mouse_sensitivity_mlook",
    "mouse_stutter_correction",
    "mouse_doubleclick_as_use",
    "mouse_carrytics",
    "movement_mouseinvert",
    "movement_mousestrafedivisor",
    "dsda_fine_sensitivity",
    // Gamepad.
    "use_game_controller",
    "movement_strafe50",
    "movement_strafe50onturns",
    "movement_shorttics",
    "left_analog_deadzone",
    "right_analog_deadzone",
    "left_trigger_deadzone",
    "right_trigger_deadzone",
    "left_analog_sensitivity_x",
    "left_analog_sensitivity_y",
    "right_analog_sensitivity_x",
    "right_analog_sensitivity_y",
    "analog_look_acceleration",
    "swap_analogs",
    "invert_analog_look",
    // Demo playback.
    "playback_mouse_controls",
    // Video baseline the template owns (spec 029, R4). String keys such as
    // `screen_resolution` are not assignable here because `-assign` values are
    // integers (research §9).
    "use_fullscreen",
    "exclusive_fullscreen",
    "render_aspect",
    "render_stretch_hud",
    // Frame pacing the template owns (spec 056, R1). `dsda_fps_limit` is
    // deliberately absent: DDL does not cap the frame rate.
    "render_vsync",
    "uncapped_framerate",
];

/// Why an override could not be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverrideError {
    /// The key is not in [`ALLOWED_KEYS`].
    UnknownKey(String),
}

impl fmt::Display for OverrideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKey(key) => write!(f, "`{key}` is not an assignable DSDA config key"),
        }
    }
}

impl std::error::Error for OverrideError {}

/// An ordered set of transient scalar overrides for one launch profile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overrides {
    entries: Vec<(String, i64)>,
}

impl Overrides {
    /// An empty override set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether `key` may be overridden.
    #[must_use]
    pub fn is_allowed(key: &str) -> bool {
        ALLOWED_KEYS.contains(&key)
    }

    /// Records `key = value`.
    ///
    /// # Errors
    /// Returns [`OverrideError::UnknownKey`] when `key` is not in the allowlist
    /// (this also rejects any `key` that could start an argv token).
    pub fn set(&mut self, key: &str, value: i64) -> Result<(), OverrideError> {
        if !Self::is_allowed(key) {
            return Err(OverrideError::UnknownKey(key.to_owned()));
        }
        match self.entries.iter_mut().find(|(name, _)| name == key) {
            Some(entry) => entry.1 = value,
            None => self.entries.push((key.to_owned(), value)),
        }
        Ok(())
    }

    /// The override for `key`, if set.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<i64> {
        self.entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| *value)
    }

    /// The overrides in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, i64)> {
        self.entries
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
    }

    /// Whether no overrides are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The `key=value` pairs, in insertion order, for `LaunchPlan`.
    #[must_use]
    pub fn assign_pairs(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect()
    }

    /// The transient argv tokens: one `-assign` followed by the pairs, or an
    /// empty vector so the caller can omit the flag entirely (D-011).
    #[must_use]
    pub fn assign_args(&self) -> Vec<String> {
        if self.entries.is_empty() {
            return Vec::new();
        }
        let mut args = Vec::with_capacity(self.entries.len() + 1);
        args.push("-assign".to_owned());
        args.extend(self.assign_pairs());
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_keys_are_valid_assign_targets() {
        for key in ALLOWED_KEYS {
            assert!(
                !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "`{key}` must match [a-z0-9_]+ (spec 003, R3)"
            );
        }
    }

    #[test]
    fn template_owned_keys_can_be_overridden_per_profile() {
        let template = super::super::template::Template::embedded().expect("valid template");
        for (key, _) in &template.values {
            assert!(
                Overrides::is_allowed(key),
                "`{key}` is owned by the template but cannot be overridden"
            );
        }
    }

    #[test]
    fn setting_a_known_key_records_it_once() {
        let mut overrides = Overrides::new();
        overrides.set("allow_freelook", 1).expect("known key");
        overrides.set("allow_freelook", 0).expect("known key");
        assert_eq!(overrides.get("allow_freelook"), Some(0));
        assert_eq!(overrides.assign_pairs(), vec!["allow_freelook=0"]);
    }

    #[test]
    fn setting_an_unknown_key_is_a_typed_error() {
        let mut overrides = Overrides::new();
        let error = overrides
            .set("input_fire", 1)
            .expect_err("binding is not a scalar");
        assert_eq!(error, OverrideError::UnknownKey("input_fire".to_owned()));
        assert!(overrides.is_empty());
    }

    #[test]
    fn assign_args_emit_one_flag_and_keep_insertion_order() {
        let mut overrides = Overrides::new();
        overrides.set("swap_analogs", 1).expect("known");
        overrides.set("mouse_sensitivity_horiz", 12).expect("known");
        assert_eq!(
            overrides.assign_args(),
            vec![
                "-assign".to_owned(),
                "swap_analogs=1".to_owned(),
                "mouse_sensitivity_horiz=12".to_owned(),
            ]
        );
    }

    #[test]
    fn no_overrides_emit_no_flag() {
        let overrides = Overrides::new();
        assert!(overrides.assign_args().is_empty());
        assert!(overrides.is_empty());
    }
}
