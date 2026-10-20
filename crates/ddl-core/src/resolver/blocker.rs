//! What the resolver needs and produces: the installed engine handle, the typed
//! blockers that keep a launch from starting, and the "Will launch" summary
//! (spec 009, R5/R6).

use std::fmt;
use std::path::PathBuf;

use crate::catalog::Game;
use crate::engines::dsda::Complevel;

/// The installed engine a resolution launches.
///
/// Spec 004 (engine install) owns discovery and installation; this is the small
/// slice the resolver needs, so spec 010 can adapt spec 004's install state to
/// it without the resolver depending on an in-progress API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// Absolute path to the engine executable (or extracted `AppRun`).
    pub executable: PathBuf,
    /// The engine version DDL recorded, e.g. `0.30.1`.
    pub version: String,
}

/// Why a launch cannot be prepared. Every variant is something DDL can explain
/// to the player and, where possible, point at a fix (spec 009, R5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Blocker {
    /// No accepted IWAD variant is installed for the required game.
    MissingIwad {
        /// The base game the recipe requires.
        game: Game,
        /// The accepted variant ids (empty means any variant of `game`).
        accepted: Vec<String>,
    },
    /// The mapset, or a member the selection needs, is not installed.
    NotInstalled,
    /// An enabled `user_supplied` component has not been imported yet (D-014).
    ComponentNotProvided {
        /// Component id.
        component: String,
        /// Component label, for the message.
        label: String,
    },
    /// No usable DSDA-Doom engine is installed.
    EngineMissing,
    /// The profile asks for something the recipe does not allow.
    IncompatibleChoice {
        /// What is incompatible.
        detail: String,
    },
    /// Installed content no longer matches what the install recorded.
    IntegrityFailed {
        /// What failed.
        detail: String,
    },
    /// A user file attached to the launch is gone (spec 026, R4).
    UserFileMissing {
        /// The missing path.
        path: PathBuf,
    },
    /// A user file changed since DDL recorded it and the player has not
    /// confirmed the change (spec 026, R4).
    UserFileChanged {
        /// The changed path.
        path: PathBuf,
    },
}

impl fmt::Display for Blocker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingIwad { game, accepted } => {
                write!(
                    f,
                    "{} is missing; add a folder containing your own copy",
                    game_label(*game)
                )?;
                if !accepted.is_empty() {
                    write!(f, " (accepted: {})", accepted.join(", "))?;
                }
                Ok(())
            }
            Self::NotInstalled => f.write_str("this mapset is not installed"),
            Self::ComponentNotProvided { label, .. } => {
                write!(f, "{label} has not been provided yet")
            }
            Self::EngineMissing => f.write_str("DSDA-Doom is not installed"),
            Self::IncompatibleChoice { detail } => write!(f, "{detail}"),
            Self::IntegrityFailed { detail } => {
                write!(f, "installed content failed an integrity check: {detail}")
            }
            Self::UserFileMissing { path } => write!(
                f,
                "your file `{}` is missing; re-add it or remove it from the list",
                path.display()
            ),
            Self::UserFileChanged { path } => write!(
                f,
                "your file `{}` changed since DDL recorded it; confirm the change to use it",
                path.display()
            ),
        }
    }
}

impl std::error::Error for Blocker {}

/// The lowercase label of a required base game.
#[must_use]
pub fn game_label(game: Game) -> &'static str {
    match game {
        Game::Doom => "doom",
        Game::Doom2 => "doom2",
        Game::Tnt => "tnt",
        Game::Plutonia => "plutonia",
    }
}

/// One entry of the numbered `-file` load order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryContent {
    /// 1-based position in the load order.
    pub position: usize,
    /// The owning component's label.
    pub label: String,
    /// Absolute path of the file.
    pub path: PathBuf,
}

/// Everything the Play setup panel shows (spec 009, R6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// The recipe's base game.
    pub base_game: Game,
    /// Whether a matching IWAD is present. Always `true` for a `Resolution`,
    /// which only exists once the IWAD was found.
    pub iwad_present: bool,
    /// The chosen IWAD variant id, once known.
    pub iwad_variant: Option<String>,
    /// The IWAD path DSDA receives (DDL's canonical symlink).
    pub iwad_path: PathBuf,
    /// The numbered `-file` load order with component labels.
    pub content: Vec<SummaryContent>,
    /// The compatibility level: DSDA's word when the recipe uses one, with the
    /// number it resolves to (D-030).
    pub complevel: Complevel,
    /// The starting skill, 1–5.
    pub skill: u8,
    /// Whether pistol start is on.
    pub pistol_start: bool,
    /// The engine version.
    pub engine_version: String,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Will launch")?;
        writeln!(
            f,
            "  base game: {} ({})",
            game_label(self.base_game),
            if self.iwad_present {
                "present"
            } else {
                "missing"
            }
        )?;
        match &self.iwad_variant {
            Some(variant) => writeln!(f, "  IWAD: {} ({variant})", self.iwad_path.display())?,
            None => writeln!(f, "  IWAD: {}", self.iwad_path.display())?,
        }
        if self.content.is_empty() {
            writeln!(f, "  content (load order): (none)")?;
        } else {
            writeln!(f, "  content (load order):")?;
            for entry in &self.content {
                writeln!(f, "    {}. {}", entry.position, entry.label)?;
                writeln!(f, "       {}", entry.path.display())?;
            }
        }
        writeln!(f, "  complevel: {}", self.complevel)?;
        writeln!(f, "  skill: {}", self.skill)?;
        writeln!(
            f,
            "  pistol start: {}",
            if self.pistol_start { "yes" } else { "no" }
        )?;
        write!(f, "  engine: {}", self.engine_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::dsda::ComplevelWord;

    #[test]
    fn a_missing_iwad_names_the_game_and_accepted_variants() {
        let blocker = Blocker::MissingIwad {
            game: Game::Doom,
            accepted: vec!["doom-1.9ud".to_owned()],
        };
        let text = blocker.to_string();
        assert!(text.contains("doom"), "{text}");
        assert!(text.contains("doom-1.9ud"), "{text}");
    }

    #[test]
    fn summary_renders_the_numbered_load_order_and_the_complevel_word() {
        let summary = Summary {
            base_game: Game::Doom2,
            iwad_present: true,
            iwad_variant: Some("doom2-1.9".to_owned()),
            iwad_path: PathBuf::from("/data/iwads/doom2-1.9/doom2.wad"),
            content: vec![
                SummaryContent {
                    position: 1,
                    label: "Main".to_owned(),
                    path: PathBuf::from("/data/library/x/1.0/free/main.wad"),
                },
                SummaryContent {
                    position: 2,
                    label: "Add-on".to_owned(),
                    path: PathBuf::from("/data/library/x/1.0/free/addon.wad"),
                },
            ],
            complevel: Complevel::from_word(ComplevelWord::Vanilla, 2).unwrap(),
            skill: 3,
            pistol_start: false,
            engine_version: "0.30.1".to_owned(),
        };
        let text = summary.to_string();
        assert!(text.contains("base game: doom2 (present)"), "{text}");
        assert!(text.contains("1. Main"), "{text}");
        assert!(text.contains("2. Add-on"), "{text}");
        assert!(text.contains("complevel: vanilla (2)"), "{text}");
        assert!(!text.contains("proposed"), "{text}");
        assert!(text.contains("engine: 0.30.1"), "{text}");
    }
}
