//! Persisted manager state and the diagnostics snapshot (spec 005, R8).
//!
//! Machine-written state is JSON with an explicit schema version (decision
//! D-006). The stored state is small on purpose: the baseline itself lives in
//! `base/<template_version>.cfg`, and the active configuration belongs to DSDA.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::template::VersionRange;

/// Schema version written into `state.json`.
pub const STATE_SCHEMA_VERSION: u32 = 1;

/// File name of the persisted state inside DDL's DSDA config directory.
pub const STATE_FILE: &str = "state.json";

/// A template key the three-way merge could not resolve without discarding a
/// user change. The active value is kept; the conflict is reported (R5, R8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    /// The DSDA config key.
    pub key: String,
    /// The value in the baseline DDL last applied.
    pub base: Option<String>,
    /// The value currently in the active file (what is kept).
    pub active: Option<String>,
    /// The value the new template wants.
    pub template: Option<String>,
}

/// State as written to disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedState {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// The template version currently applied to the active file.
    pub template_version: String,
    /// Unix seconds of the last migration or reset, if any.
    pub last_migration: Option<u64>,
    /// Conflicts left unresolved by the last migration.
    pub conflicts: Vec<Conflict>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            template_version: String::new(),
            last_migration: None,
            conflicts: Vec::new(),
        }
    }
}

/// The read-only diagnostics snapshot returned by the manager (R8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// DSDA-Doom versions the current template was checked against.
    pub dsda_version_range: VersionRange,
    /// The template version currently applied.
    pub template_version: String,
    /// Absolute path to the active configuration file.
    pub active_file: PathBuf,
    /// Unix seconds of the last migration or reset, if any.
    pub last_migration: Option<u64>,
    /// Conflicts left unresolved by the last migration.
    pub conflicts: Vec<Conflict>,
}

impl State {
    /// The active path shortened against `home`, so exported logs carry
    /// `~/.config/ddl/...` instead of a private absolute path (R8).
    #[must_use]
    pub fn active_file_display(&self, home: &Path) -> String {
        shorten_home(&self.active_file, home)
    }
}

/// Replaces a leading `home` component with `~`.
#[must_use]
pub fn shorten_home(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorten_home_replaces_the_home_prefix() {
        assert_eq!(
            shorten_home(
                Path::new("/home/deck/.config/ddl/dsda/active.cfg"),
                Path::new("/home/deck")
            ),
            "~/.config/ddl/dsda/active.cfg"
        );
        assert_eq!(
            shorten_home(Path::new("/home/deck"), Path::new("/home/deck")),
            "~"
        );
        assert_eq!(
            shorten_home(Path::new("/etc/ddl/active.cfg"), Path::new("/home/deck")),
            "/etc/ddl/active.cfg"
        );
    }

    #[test]
    fn persisted_state_round_trips_as_versioned_json() {
        let state = PersistedState {
            schema_version: STATE_SCHEMA_VERSION,
            template_version: "2".to_owned(),
            last_migration: Some(1_700_000_000),
            conflicts: vec![Conflict {
                key: "mouse_sensitivity_horiz".to_owned(),
                base: Some("10".to_owned()),
                active: Some("12".to_owned()),
                template: Some("14".to_owned()),
            }],
        };
        let json = serde_json::to_string(&state).expect("serialize");
        assert!(json.contains("\"schema_version\":1"));
        assert_eq!(
            serde_json::from_str::<PersistedState>(&json).expect("deserialize"),
            state
        );
    }
}
