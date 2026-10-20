//! Paths the host has but the Flatpak sandbox cannot see (spec 059, R4).
//!
//! Inside the sandbox DDL reaches the player's files only through the
//! read-only host access the Flatpak was granted. A Steam library on an SD card
//! (`/run/media/...`) or a Heroic install on another disk is *named* by records
//! DDL can read (`libraryfolders.vdf`, Heroic's `installed.json`, minigalaxy's
//! config) yet the directory itself may be missing from the sandbox's mount
//! namespace or refuse reading. Showing an empty list then looks like "no games";
//! this module finds those paths so the UI can say what is wrong instead.
//!
//! It makes no assumption about which permissions the Flatpak was built with:
//! a path counts as hidden only when something *points at it* (a launcher
//! record, or a folder the player configured) and it cannot be opened. Outside
//! the sandbox it reports nothing, so non-Flatpak behaviour is unchanged.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use super::{steam, stores};
use crate::platform::sandbox::Sandbox;

/// Most recorded launcher paths inspected, so a hostile record cannot make
/// startup slow.
const MAX_PATHS: usize = 256;

/// Largest launcher record read, in bytes.
const MAX_RECORD_BYTES: u64 = 4 * 1024 * 1024;

/// Whether a directory can be opened from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// The directory opens (or the path is not a directory: not our concern).
    Visible,
    /// The path does not exist in this mount namespace.
    Missing,
    /// The path exists but reading it is refused.
    Denied,
}

impl Access {
    /// Whether the path is out of reach.
    #[must_use]
    pub fn is_hidden(self) -> bool {
        self != Self::Visible
    }
}

/// Probes `path` by opening it as a directory.
#[must_use]
pub fn access(path: &Path) -> Access {
    match std::fs::read_dir(path) {
        Ok(_) => Access::Visible,
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => Access::Missing,
            io::ErrorKind::PermissionDenied => Access::Denied,
            _ => Access::Visible,
        },
    }
}

/// Directories that readable launcher records point to, whether or not those
/// directories can be opened.
#[must_use]
pub fn recorded_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for root in steam::steam_roots(home) {
        let vdf = root.join("steamapps").join("libraryfolders.vdf");
        if let Some(text) = read_small(&vdf) {
            roots.extend(steam::library_paths_from_vdf(&text));
        }
    }
    for record in stores::heroic_installed_files(home) {
        if let Some(text) = read_small(&record) {
            roots.extend(stores::install_paths_from_heroic(&text));
        }
    }
    for config in stores::minigalaxy_config_files(home) {
        if let Some(text) = read_small(&config)
            && let Some(dir) = stores::install_dir_from_minigalaxy(&text)
        {
            roots.push(dir);
        }
    }
    roots.sort();
    roots.dedup();
    roots.truncate(MAX_PATHS);
    roots
}

/// The paths named by launcher records or by `configured` folders that the
/// sandbox cannot open. Empty outside the sandbox.
#[must_use]
pub fn hidden_paths(sandbox: &Sandbox, home: &Path, configured: &[PathBuf]) -> Vec<PathBuf> {
    if !sandbox.is_flatpak() {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    recorded_roots(home)
        .into_iter()
        .chain(configured.iter().cloned())
        .filter(|path| path.is_absolute() && seen.insert(path.clone()))
        .filter(|path| access(path).is_hidden())
        .collect()
}

fn read_small(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > MAX_RECORD_BYTES {
        return None;
    }
    std::io::read_to_string(file).ok()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn sandboxed() -> Sandbox {
        Sandbox::flatpak(Some("io.github.vittau.Doom-Deck-Launcher".to_owned()))
    }

    /// A fake host: a Steam root whose `libraryfolders.vdf` names one library
    /// that exists in the sandbox and one (an SD card) that does not.
    fn host(temp: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let home = temp.join("home");
        let visible = home.join("SteamLibrary");
        let sd_card = temp.join("run/media/deck/sd-card/SteamLibrary");
        fs::create_dir_all(visible.join("steamapps")).unwrap();
        let steam = home.join(".local/share/Steam/steamapps");
        fs::create_dir_all(&steam).unwrap();
        fs::write(
            steam.join("libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n",
                visible.display(),
                sd_card.display()
            ),
        )
        .unwrap();
        (home, visible, sd_card)
    }

    #[test]
    fn a_library_the_sandbox_cannot_open_is_reported() {
        let temp = tempfile::tempdir().unwrap();
        let (home, visible, sd_card) = host(temp.path());

        let hidden = hidden_paths(&sandboxed(), &home, &[]);

        assert_eq!(hidden, std::slice::from_ref(&sd_card));
        assert_eq!(access(&sd_card), Access::Missing);
        assert_eq!(access(&visible), Access::Visible);
        // The visible library is still discovered as before.
        assert!(
            steam::libraries(&home)
                .iter()
                .any(|library| library.path == visible)
        );
    }

    #[test]
    fn outside_the_sandbox_nothing_is_reported() {
        let temp = tempfile::tempdir().unwrap();
        let (home, _, _) = host(temp.path());
        assert!(hidden_paths(&Sandbox::none(), &home, &[]).is_empty());
    }

    #[test]
    fn a_configured_folder_that_is_gone_is_reported_once() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let gone = temp.path().join("run/media/deck/usb/Doom");
        let here = home.join("Doom");
        fs::create_dir_all(&here).unwrap();

        let hidden = hidden_paths(
            &sandboxed(),
            &home,
            &[gone.clone(), here, gone.clone(), PathBuf::from("relative")],
        );

        assert_eq!(hidden, [gone]);
    }

    #[test]
    fn heroic_and_minigalaxy_records_are_followed() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        let heroic = home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store");
        fs::create_dir_all(&heroic).unwrap();
        let lost = temp.path().join("mnt/games/DOOM II");
        fs::write(
            heroic.join("installed.json"),
            format!(
                "{{\"installed\":[{{\"install_path\":\"{}\"}}]}}",
                lost.display()
            ),
        )
        .unwrap();
        let mini = home.join(".config/minigalaxy");
        fs::create_dir_all(&mini).unwrap();
        let lost_mini = temp.path().join("mnt/gog");
        fs::write(
            mini.join("config.json"),
            format!("{{\"install_dir\":\"{}\"}}", lost_mini.display()),
        )
        .unwrap();

        let hidden = hidden_paths(&sandboxed(), &home, &[]);

        assert_eq!(hidden, [lost, lost_mini]);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_that_refuses_reading_is_reported_as_denied() {
        use std::os::unix::fs::PermissionsExt as _;
        let temp = tempfile::tempdir().unwrap();
        let locked = temp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        // Root (some CI containers) reads anything; then there is nothing to test.
        let readable = fs::read_dir(&locked).is_ok();
        let seen = access(&locked);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        if !readable {
            assert_eq!(seen, Access::Denied);
            assert!(seen.is_hidden());
        }
    }
}
