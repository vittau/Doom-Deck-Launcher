//! DDL-owned DSDA-Doom configuration management (spec 005).
//!
//! The module owns four things:
//!
//! - [`format`] — a parser/serializer for DSDA's line-oriented config format
//!   (R1).
//! - [`Template`] — the versioned `steam-deck-default` baseline, shipped as TOML
//!   (R2).
//! - [`DsdaConfigManager`] — first-use provisioning, three-way migration,
//!   backups and reset against DDL's private config file (R3, R5, R6, R8).
//! - [`RunGuard`] — the shared guard that keeps DDL from touching the config
//!   while DSDA is running (R4).
//!
//! [`Overrides`] renders per-profile scalar overrides as transient `-assign`
//! tokens for the launch plan (R7); they are never written into the active file.

mod backups;
pub mod format;
mod guard;
mod manager;
mod merge;
mod overrides;
mod state;
mod template;

use std::io;

pub use guard::{GuardError, LOCK_FILE, PidLiveness, RunGuard, SystemLiveness, recorded_pid};
pub use manager::{Clock, DSDA_DIR, DsdaConfigManager, Migration, Provision, Reset, SystemClock};
pub use merge::{MergeResult, merge};
pub use overrides::{ALLOWED_KEYS, OverrideError, Overrides};
pub use state::{Conflict, State, shorten_home};
pub use template::{Template, TemplateError, VersionRange};

/// Why a configuration operation failed.
#[derive(Debug)]
pub enum Error {
    /// A filesystem operation failed.
    Io(io::Error),
    /// An engine run is active, so the configuration must not be touched.
    RunActive {
        /// The PID recorded in the run lock, when it could be read.
        pid: Option<u32>,
    },
    /// [`DsdaConfigManager::reset`] was called without `confirm = true`.
    ConfirmationRequired,
    /// The active file or state is missing; call
    /// [`DsdaConfigManager::provision`] (or `ensure_ready`) first.
    NotProvisioned,
    /// The baseline of the previously applied template version is missing, so a
    /// correct three-way merge is impossible. The caller should offer reset.
    MissingBaseline {
        /// The template version whose baseline is missing.
        version: String,
    },
    /// The embedded or supplied template could not be parsed.
    InvalidTemplate(TemplateError),
    /// The persisted state file could not be parsed.
    InvalidState(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::RunActive { pid: Some(pid) } => {
                write!(f, "a DSDA-Doom run (PID {pid}) is active")
            }
            Self::RunActive { pid: None } => f.write_str("a DSDA-Doom run is active"),
            Self::ConfirmationRequired => f.write_str("this operation needs explicit confirmation"),
            Self::NotProvisioned => f.write_str("the DSDA configuration has not been provisioned"),
            Self::MissingBaseline { version } => write!(
                f,
                "the baseline for template version `{version}` is missing, so a safe merge is impossible"
            ),
            Self::InvalidTemplate(err) => write!(f, "{err}"),
            Self::InvalidState(err) => write!(f, "invalid DSDA config state: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::InvalidTemplate(err) => Some(err),
            Self::InvalidState(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<TemplateError> for Error {
    fn from(err: TemplateError) -> Self {
        Self::InvalidTemplate(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidState(err)
    }
}

impl From<GuardError> for Error {
    fn from(err: GuardError) -> Self {
        match err {
            GuardError::RunActive { pid } => Self::RunActive { pid },
            GuardError::Io(err) => Self::Io(err),
        }
    }
}
