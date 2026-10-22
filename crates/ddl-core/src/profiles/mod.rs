//! Per-mapset play profiles (spec 009, R4).
//!
//! A [`Profile`] records the player's choices for one recipe: enabled optional
//! components (honouring exclusive groups and replacements), skill, pistol
//! start, an optional IWAD-variant override, the last played time and the
//! transient scalar overrides spec 005 renders as `-assign`. Each profile owns
//! its `save`, `data` and `shots` directories under `<data>/profiles/<id>/`
//! (decision D-011).
//!
//! ```no_run
//! use ddl_core::profiles::{Profile, ProfileStore};
//!
//! let store = ProfileStore::new("/home/deck/.config/ddl", "/home/deck/.local/share/ddl");
//! let mut profile = Profile::new("p1", "Play SIGIL", "sigil");
//! profile.set_component("compat", false);
//! profile.pistol_start = true;
//! store.save(&profile).expect("save");
//! ```

mod names;
mod profile;
mod store;

pub use names::validate_name;
pub use profile::{DEFAULT_SKILL, Profile, ProfileError, SCHEMA_VERSION};
pub use store::{CONFIG_DIR, DATA_DIR, ProfileDirs, ProfileStore};

/// A small recipe with a main component, an exclusive replacement and a
/// user-supplied option, used by unit tests that only need component selection
/// (it is intentionally not lint-clean: it names no packages).
#[cfg(test)]
pub(crate) fn test_recipe() -> crate::catalog::Recipe {
    use crate::catalog::{
        BaseGame, Compatibility, Complevel, Component, ComponentMember, EngineEvidence, Game,
        Package, PortFamily, Recipe, Role, RuntimeDefaults, Verification, VerificationStatus,
    };

    let empty_members = || {
        vec![ComponentMember {
            package: None,
            member: None,
            role: Role::Maps,
        }]
    };
    Recipe {
        schema_version: 1,
        id: "sigil-test".to_owned(),
        title: "Sigil test".to_owned(),
        authors: vec!["Test".to_owned()],
        release: "1.0".to_owned(),
        original_release: None,
        page: "https://example.invalid/page".to_owned(),
        description: "Test recipe.".to_owned(),
        permission: "Test.".to_owned(),
        screenshots: Vec::new(),
        images: Vec::new(),
        base_game: BaseGame {
            game: Game::Doom,
            variants: vec!["doom-1.9ud".to_owned()],
        },
        packages: Vec::<Package>::new(),
        components: vec![
            Component {
                id: "main".to_owned(),
                label: "Main".to_owned(),
                description: "Main content.".to_owned(),
                optional: false,
                default: true,
                position: 1,
                exclusive_group: None,
                replaces: None,
                requires: None,
                user_supplied: false,
                obtain: None,
                known_sha256: Vec::new(),
                members: empty_members(),
            },
            Component {
                id: "compat".to_owned(),
                label: "Compat".to_owned(),
                description: "Replacement.".to_owned(),
                optional: true,
                default: false,
                position: 1,
                exclusive_group: Some("edition".to_owned()),
                replaces: Some("main".to_owned()),
                requires: None,
                user_supplied: false,
                obtain: None,
                known_sha256: Vec::new(),
                members: empty_members(),
            },
            Component {
                id: "soundtrack".to_owned(),
                label: "Soundtrack".to_owned(),
                description: "User supplied.".to_owned(),
                optional: true,
                default: false,
                position: 1,
                exclusive_group: Some("edition".to_owned()),
                replaces: Some("main".to_owned()),
                requires: None,
                user_supplied: true,
                obtain: Some("Import your file.".to_owned()),
                known_sha256: Vec::new(),
                members: empty_members(),
            },
        ],
        compatibility: Compatibility {
            family: PortFamily::Vanilla,
            complevel: Complevel::Number(3),
            engine: EngineEvidence {
                range: "=0.30.1".to_owned(),
                status: VerificationStatus::Inspected,
            },
            runtime: RuntimeDefaults::default(),
        },
        verification: Verification {
            status: VerificationStatus::Inspected,
            tester: "test".to_owned(),
            date: "2026-10-16".to_owned(),
            engine_version: "0.30.1".to_owned(),
            inspected: vec!["test".to_owned()],
            notes: "test".to_owned(),
            limitations: Vec::new(),
        },
        tags: Vec::new(),
    }
}
