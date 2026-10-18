//! Building a DSDA-Doom launch plan for a custom setup, and the file-identity
//! checks that gate every local-content launch (spec 026, R2/R3/R4).
//!
//! The plan is assembled through the existing [`LaunchPlan`] builder (spec 003),
//! so a custom setup gets the same absolute paths, separate argv tokens,
//! `-noautoload`, and DDL-owned config/save/data/shot directories as a catalog
//! mapset. The only differences the model expresses are the two ordered groups
//! and the optional `-complevel`.

use std::path::{Path, PathBuf};

use crate::catalog::Game as RecipeGame;
use crate::dsda_config::{ALLOWED_KEYS, DsdaConfigManager, Overrides};
use crate::engines::dsda::{AssignAllowlist, AssignOverride, EngineRunner, LaunchPlan};
use crate::iwads::identify::{Identifier, Outcome, sha256_file};
use crate::iwads::registry::{FileTime, Registry, RegistryEntry, Source};
use crate::iwads::{CanonicalLinks, Game as IwadsGame, IwadTable};
use crate::profiles::{Profile, ProfileDirs};

use super::kind::{IdentityError, UserFile};
use super::setup::{CustomIwad, CustomSetup, SetupError};

/// The `-assign` keys DDL owns (spec 005, R7), used by a custom launch too.
#[must_use]
pub fn assign_allowlist() -> AssignAllowlist {
    AssignAllowlist::with_keys(ALLOWED_KEYS.iter().copied())
}

/// The engine a custom setup launches, mirroring `resolver::Engine` so the
/// caller can adapt spec 004's install state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// Absolute path to the engine executable.
    pub executable: PathBuf,
    /// The engine version DDL recorded.
    pub version: String,
}

/// Why a custom launch cannot be prepared.
#[derive(Debug)]
pub enum LaunchError {
    /// A user file is missing or changed (spec 026, R4), named in full.
    Identity(IdentityError),
    /// The identified IWAD, or its canonical link, is unusable.
    Iwad(String),
    /// The setup or profile is not internally consistent.
    Setup(SetupError),
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identity(error) => write!(f, "{error}"),
            Self::Iwad(detail) => write!(f, "{detail}"),
            Self::Setup(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for LaunchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Identity(error) => Some(error),
            Self::Setup(error) => Some(error),
            Self::Iwad(_) => None,
        }
    }
}

impl From<IdentityError> for LaunchError {
    fn from(error: IdentityError) -> Self {
        Self::Identity(error)
    }
}

impl From<SetupError> for LaunchError {
    fn from(error: SetupError) -> Self {
        Self::Setup(error)
    }
}

/// Verifies every recorded user file of `setup` in model order (spec 026, R4).
#[must_use]
pub fn verify_setup(setup: &CustomSetup) -> Vec<IdentityError> {
    setup
        .all_user_files()
        .into_iter()
        .filter_map(|file| file.verify().err())
        .collect()
}

/// The first problem among a file list, if any (spec 026, R4).
#[must_use]
pub fn first_issue(files: &[UserFile]) -> Option<IdentityError> {
    files.iter().find_map(|file| file.verify().err())
}

/// Re-records the SHA-256 and size of every **changed** entry, keeping the path
/// and kind. Missing entries are left for the caller to report. Returns the
/// paths that were re-recorded (spec 026, R4).
///
/// # Errors
///
/// Returns [`SetupError::File`] when a changed file cannot be re-read.
pub fn accept_changed(files: &mut [UserFile]) -> Result<Vec<PathBuf>, SetupError> {
    let mut accepted = Vec::new();
    for file in files.iter_mut() {
        match file.verify() {
            Ok(()) | Err(IdentityError::Missing { .. }) => {}
            Err(IdentityError::Changed { .. }) => {
                let fresh = file.rehash()?;
                accepted.push(fresh.path.clone());
                *file = fresh;
            }
            Err(IdentityError::Unreadable { path, error }) => {
                return Err(SetupError::File(super::kind::FileError::Unreadable {
                    path,
                    error,
                }));
            }
        }
    }
    Ok(accepted)
}

/// Builds the launch plan for `setup` and `profile` (spec 026, R3).
///
/// The profile supplies skill, pistol start and `-assign` overrides; the setup
/// supplies the IWAD, the two ordered groups and the compatibility choice. The
/// plan's directories are created by [`LaunchPlan::prepare_dirs`] before the
/// caller validates and runs it.
///
/// # Errors
///
/// Returns a [`LaunchError`] when a user file failed its identity check or the
/// IWAD cannot be resolved.
pub fn build_plan(
    setup: &CustomSetup,
    profile: &Profile,
    engine: &Engine,
    registry: &Registry,
    data_dir: &Path,
    config: &DsdaConfigManager,
) -> Result<LaunchPlan, LaunchError> {
    setup.validate().map_err(LaunchError::Setup)?;
    for file in setup.all_user_files() {
        file.verify()?;
    }

    let iwad = resolve_iwad(&setup.iwad, registry, data_dir)?;
    let dirs = ProfileDirs::new(data_dir, &profile.id);
    let skill = crate::engines::dsda::Skill::new(profile.skill)
        .map_err(|error| LaunchError::Setup(SetupError::Invalid(error.to_string())))?;

    Ok(LaunchPlan {
        executable: engine.executable.clone(),
        iwad,
        files: setup.files.iter().map(|f| f.path.clone()).collect(),
        patches: setup.patches.iter().map(|p| p.path.clone()).collect(),
        complevel: setup.compatibility.level(),
        skill,
        warp: None,
        pistol_start: profile.pistol_start,
        record: None,
        config: config.active_path(),
        save_dir: dirs.save,
        data_dir: dirs.data,
        shot_dir: dirs.shots,
        working_dir: dirs.run,
        assigns: profile_assigns(profile)?,
    })
}

/// The transient `-assign` overrides a profile owns, validated against DDL's
/// allowlist (spec 005, R7).
fn profile_assigns(profile: &Profile) -> Result<Vec<AssignOverride>, LaunchError> {
    let mut overrides = Overrides::new();
    for (key, value) in &profile.overrides {
        overrides
            .set(key, *value)
            .map_err(|error| LaunchError::Setup(SetupError::Invalid(error.to_string())))?;
    }
    Ok(overrides
        .iter()
        .map(|(key, value)| AssignOverride::new(key, value.to_string()))
        .collect())
}

/// The IWAD path DSDA receives (decision D-017).
///
/// An identified variant goes through its canonical link. A user file whose
/// header is `IWAD` is identified once more at launch: when it is a known
/// variant it too goes through the canonical link; otherwise it is passed as it
/// is and DSDA derives the mission from its file name.
fn resolve_iwad(
    iwad: &CustomIwad,
    registry: &Registry,
    data_dir: &Path,
) -> Result<PathBuf, LaunchError> {
    let table = IwadTable::embedded();
    let links = CanonicalLinks::new(data_dir);
    match iwad {
        CustomIwad::Identified { variant } => {
            let entry = registry
                .entries()
                .iter()
                .find(|entry| entry.variant.as_deref() == Some(variant.as_str()))
                .ok_or_else(|| {
                    LaunchError::Iwad(format!(
                        "IWAD `{variant}` is not in DDL's registry; run `ddl iwad scan` first"
                    ))
                })?;
            links
                .canonical_path(entry, &table)
                .map_err(|error| LaunchError::Iwad(error.to_string()))
        }
        CustomIwad::File { file } => {
            let identification = Identifier::embedded()
                .identify(&file.path)
                .map_err(|error| {
                    LaunchError::Iwad(format!(
                        "`{}` is not a readable IWAD: {error}",
                        file.path.display()
                    ))
                })?;
            match identification.outcome {
                Outcome::Known { id, game } => {
                    let entry = entry_for(file, &id, game)?;
                    links
                        .canonical_path(&entry, &table)
                        .map_err(|error| LaunchError::Iwad(error.to_string()))
                }
                _ => Ok(file.path.clone()),
            }
        }
    }
}

/// A synthetic registry entry for a user-file IWAD that identification matched,
/// so it can go through the canonical link like any scanned IWAD.
fn entry_for(
    file: &UserFile,
    variant: &str,
    game: IwadsGame,
) -> Result<RegistryEntry, LaunchError> {
    let metadata = std::fs::metadata(&file.path).map_err(|error| {
        LaunchError::Iwad(format!("cannot read `{}`: {error}", file.path.display()))
    })?;
    let mtime = FileTime::from_metadata(&metadata).ok_or_else(|| {
        LaunchError::Iwad(format!(
            "`{}` has no usable modification time",
            file.path.display()
        ))
    })?;
    let sha256 = sha256_file(&file.path)
        .map_err(|error| LaunchError::Iwad(format!("cannot hash the IWAD: {error}")))?;
    Ok(RegistryEntry {
        path: file.path.clone(),
        variant: Some(variant.to_owned()),
        candidates: Vec::new(),
        game: Some(game),
        sha256,
        size: metadata.len(),
        mtime,
        source: Source::Manual,
    })
}

/// Maps an identified IWAD family onto the recipe catalog's base game.
#[must_use]
pub fn recipe_game(game: IwadsGame) -> RecipeGame {
    match game {
        IwadsGame::Doom => RecipeGame::Doom,
        IwadsGame::Doom2 | IwadsGame::Nerve => RecipeGame::Doom2,
        IwadsGame::Tnt => RecipeGame::Tnt,
        IwadsGame::Plutonia => RecipeGame::Plutonia,
    }
}

/// Runs a prepared plan through `runner` (spec 003), for the CLI's
/// `ddl launch <custom id>` without `--dry-run`.
///
/// # Errors
///
/// Returns the runner's error.
pub fn launch(
    runner: &dyn EngineRunner,
    plan: &LaunchPlan,
) -> Result<crate::engines::dsda::LaunchOutcome, crate::engines::dsda::LaunchError> {
    runner.launch(plan, &assign_allowlist())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::engines::dsda::Complevel;
    use crate::local::kind::Kind;
    use crate::local::setup::Compatibility;

    use tempfile::TempDir;

    /// A minimal valid WAD built by hand; no real game data.
    pub(super) fn wad(magic: &[u8; 4], lumps: &[&str]) -> Vec<u8> {
        let mut directory = Vec::new();
        for name in lumps {
            directory.extend_from_slice(&0_i32.to_le_bytes());
            directory.extend_from_slice(&0_i32.to_le_bytes());
            let mut padded = [0_u8; 8];
            let len = name.len().min(8);
            padded[..len].copy_from_slice(&name.as_bytes()[..len]);
            directory.extend_from_slice(&padded);
        }
        let mut out = Vec::new();
        out.extend_from_slice(magic);
        out.extend_from_slice(&(lumps.len() as i32).to_le_bytes());
        out.extend_from_slice(&12_i32.to_le_bytes());
        out.extend_from_slice(&directory);
        out
    }

    fn write(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn user_file(dir: &TempDir, name: &str, bytes: &[u8], kind: Kind) -> UserFile {
        let path = write(dir, name, bytes);
        let file = UserFile::read(&path).unwrap();
        assert_eq!(file.kind, kind);
        file
    }

    fn setup(
        files: Vec<UserFile>,
        patches: Vec<UserFile>,
        compatibility: Compatibility,
    ) -> CustomSetup {
        let mut setup = CustomSetup::new(
            "my-setup",
            "My Setup",
            CustomIwad::Identified {
                variant: "doom2-1.9".to_owned(),
            },
            compatibility,
            1_700_000_000,
        );
        setup.files = files;
        setup.patches = patches;
        setup
    }

    fn profile() -> Profile {
        let mut profile = Profile::new("my-setup-default", "Default", "my-setup");
        profile.skill = 4;
        profile.pistol_start = true;
        profile
    }

    fn engine() -> Engine {
        Engine {
            executable: PathBuf::from("/opt/ddl/dsda-doom.appimage"),
            version: "0.30.1".to_owned(),
        }
    }

    fn config(temp: &TempDir) -> DsdaConfigManager {
        DsdaConfigManager::new(temp.path().join("config")).unwrap()
    }

    fn tokens(plan: &LaunchPlan) -> Vec<String> {
        plan.to_command()
            .iter()
            .map(|token| token.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn auto_omits_complevel_and_pinned_emits_it_with_two_ordered_groups() {
        let temp = TempDir::new().unwrap();
        let data = temp.path().join("data");
        let config = config(&temp);
        let files = vec![
            user_file(&temp, "first.wad", &wad(b"PWAD", &["MAP01"]), Kind::Wad),
            user_file(&temp, "second.zip", b"PK\x03\x04rest", Kind::Zip),
        ];
        let patches = vec![user_file(
            &temp,
            "fix.deh",
            b"Patch File for DeHackEd v3.0\nThing 1\nHit points = 30\n",
            Kind::Deh,
        )];

        // Auto: no `-complevel`, then `-file` with both files in order, `-deh`
        // after it.
        let auto = build_plan(
            &setup(files.clone(), patches.clone(), Compatibility::Auto),
            &profile(),
            &engine(),
            &Registry::new(),
            &data,
            &config,
        );
        // The IWAD is not in the registry, so the plan is blocked before argv.
        assert!(matches!(auto, Err(LaunchError::Iwad(_))));

        // With a registry entry the plan builds. Use a synthetic variant table
        // entry by reusing the real `doom2-1.9` variant, whose file we supply.
        let iwad_path = temp.path().join("doom2.wad");
        std::fs::write(&iwad_path, wad(b"IWAD", &["PLAYPAL"])).unwrap();
        let mut registry = Registry::new();
        let metadata = std::fs::metadata(&iwad_path).unwrap();
        registry.insert(RegistryEntry {
            path: iwad_path.clone(),
            variant: Some("doom2-1.9".to_owned()),
            candidates: Vec::new(),
            game: Some(IwadsGame::Doom2),
            sha256: sha256_file(&iwad_path).unwrap(),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).unwrap(),
            source: Source::Manual,
        });

        let plan = build_plan(
            &setup(files.clone(), patches.clone(), Compatibility::Auto),
            &profile(),
            &engine(),
            &registry,
            &data,
            &config,
        )
        .unwrap();
        assert_eq!(plan.complevel, None, "Auto omits -complevel");
        let argv = tokens(&plan);
        assert!(!argv.iter().any(|token| token == "-complevel"), "{argv:?}");
        let file_at = argv.iter().position(|t| t == "-file").unwrap();
        let deh_at = argv.iter().position(|t| t == "-deh").unwrap();
        assert!(file_at < deh_at, "the files list precedes the patch list");
        for (offset, expected) in files.iter().enumerate() {
            assert_eq!(argv[file_at + 1 + offset], expected.path.to_string_lossy());
        }
        assert_eq!(argv[deh_at + 1], patches[0].path.to_string_lossy());
        assert!(argv.contains(&"-noautoload".to_owned()));
        assert_eq!(plan.skill.value(), 4);
        assert!(plan.pistol_start);

        // Pinned: `-complevel 21`.
        let plan = build_plan(
            &setup(files, patches, Compatibility::Level(Complevel::MBF21)),
            &profile(),
            &engine(),
            &registry,
            &data,
            &config,
        )
        .unwrap();
        let argv = tokens(&plan);
        let at = argv.iter().position(|t| t == "-complevel").unwrap();
        assert_eq!(argv[at + 1], "21");
    }

    #[test]
    fn a_user_file_iwad_is_passed_through_without_the_registry() {
        let temp = TempDir::new().unwrap();
        let data = temp.path().join("data");
        let config = config(&temp);
        let iwad = user_file(&temp, "my.wad", &wad(b"IWAD", &["PLAYPAL"]), Kind::Wad);
        let mut setup = setup(Vec::new(), Vec::new(), Compatibility::Auto);
        setup.iwad = CustomIwad::File { file: iwad.clone() };
        let plan = build_plan(
            &setup,
            &profile(),
            &engine(),
            &Registry::new(),
            &data,
            &config,
        )
        .unwrap();
        // Not a known variant, so the user's own path is used.
        assert_eq!(plan.iwad, iwad.path);

        // The user-file IWAD is checked with the other user files (R4).
        std::fs::remove_file(&iwad.path).unwrap();
        assert!(matches!(
            verify_setup(&setup).as_slice(),
            [IdentityError::Missing { .. }]
        ));
    }

    #[test]
    fn missing_and_changed_files_block_and_accepting_rerecords_only_changes() {
        let temp = TempDir::new().unwrap();
        let path = write(&temp, "map.wad", &wad(b"PWAD", &["MAP01"]));
        let mut files = vec![UserFile::read(&path).unwrap()];
        assert!(first_issue(&files).is_none());

        std::fs::write(&path, wad(b"PWAD", &["MAP02"])).unwrap();
        assert!(matches!(
            first_issue(&files),
            Some(IdentityError::Changed { .. })
        ));
        let accepted = accept_changed(&mut files).unwrap();
        assert_eq!(accepted, vec![path.clone()]);
        assert!(first_issue(&files).is_none());

        std::fs::remove_file(&path).unwrap();
        assert!(matches!(
            first_issue(&files),
            Some(IdentityError::Missing { .. })
        ));
        assert!(accept_changed(&mut files).unwrap().is_empty());
    }

    #[test]
    fn a_missing_identified_iwad_is_reported_before_the_plan_is_built() {
        let temp = TempDir::new().unwrap();
        let error = build_plan(
            &setup(Vec::new(), Vec::new(), Compatibility::Auto),
            &profile(),
            &engine(),
            &Registry::new(),
            &temp.path().join("data"),
            &config(&temp),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("not in DDL's registry"),
            "{error}"
        );
    }
}
