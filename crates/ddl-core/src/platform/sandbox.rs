//! Detection of the Flatpak sandbox (spec 059, R1; decision D-034).
//!
//! A process runs under Flatpak when `/.flatpak-info` exists, the documented
//! marker for apps (docs/research/flatpak.md §3.2). The probe goes through
//! [`Probe`] and the environment through [`Env`], so tests use a fake marker
//! and fake variables and never depend on where they run.
//!
//! [`Sandbox`] is the one value the rest of DDL asks "am I sandboxed?" of. It
//! only changes behaviour when the answer is yes: [`Sandbox::none`] keeps every
//! non-Flatpak code path exactly as it was.

use std::path::Path;

use super::paths::Env;

/// The file Flatpak mounts into every sandbox.
pub const FLATPAK_INFO: &str = "/.flatpak-info";

/// Environment variable naming the Flatpak application id.
pub const FLATPAK_ID_VAR: &str = "FLATPAK_ID";

/// A source of "does this path exist?" answers.
///
/// Implemented for every `Fn(&Path) -> bool`, so tests pass a closure.
pub trait Probe {
    /// Whether `path` exists.
    fn exists(&self, path: &Path) -> bool;
}

impl<F> Probe for F
where
    F: Fn(&Path) -> bool,
{
    fn exists(&self, path: &Path) -> bool {
        self(path)
    }
}

/// The real filesystem.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostProbe;

impl Probe for HostProbe {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

/// Whether DDL runs inside a sandbox, and which one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sandbox {
    flatpak: Option<Flatpak>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Flatpak {
    /// `FLATPAK_ID`, when Flatpak set it to something usable.
    app_id: Option<String>,
}

impl Sandbox {
    /// Not sandboxed: today's behaviour everywhere.
    #[must_use]
    pub fn none() -> Self {
        Self { flatpak: None }
    }

    /// A Flatpak sandbox with the given application id.
    #[must_use]
    pub fn flatpak(app_id: Option<String>) -> Self {
        Self {
            flatpak: Some(Flatpak { app_id }),
        }
    }

    /// Detects the sandbox from `probe` and `env`.
    ///
    /// `FLATPAK_ID` is only reported when it is a non-empty single line.
    #[must_use]
    pub fn detect(probe: &impl Probe, env: &impl Env) -> Self {
        if !probe.exists(Path::new(FLATPAK_INFO)) {
            return Self::none();
        }
        let app_id = env
            .var(FLATPAK_ID_VAR)
            .map(|id| id.trim().to_owned())
            .filter(|id| !id.is_empty() && !id.contains(['\n', '\r']));
        Self::flatpak(app_id)
    }

    /// Detects the sandbox of the running process.
    #[must_use]
    pub fn detect_host() -> Self {
        Self::detect(&HostProbe, &|key: &str| std::env::var(key).ok())
    }

    /// Whether DDL runs under Flatpak.
    #[must_use]
    pub fn is_flatpak(&self) -> bool {
        self.flatpak.is_some()
    }

    /// The Flatpak application id, when sandboxed and known.
    #[must_use]
    pub fn app_id(&self) -> Option<&str> {
        self.flatpak.as_ref()?.app_id.as_deref()
    }

    /// Whether the engine AppImage must be extracted instead of mounted
    /// (R2): FUSE cannot work inside Flatpak (research §3.1).
    #[must_use]
    pub fn must_extract_engine(&self) -> bool {
        self.is_flatpak()
    }

    /// Whether DDL may offer to update itself (R7). Under Flatpak the app
    /// is updated by Discover / Flathub, and `/app` is read-only anyway.
    #[must_use]
    pub fn allows_self_update(&self) -> bool {
        !self.is_flatpak()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;

    fn env(pairs: &[(&str, &str)]) -> impl Env {
        let map: BTreeMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    fn marker(present: bool) -> impl Probe {
        move |path: &Path| present && path == Path::new("/.flatpak-info")
    }

    #[test]
    fn the_marker_file_means_flatpak() {
        let sandbox = Sandbox::detect(&marker(true), &env(&[]));
        assert!(sandbox.is_flatpak());
        assert!(sandbox.must_extract_engine());
        assert!(!sandbox.allows_self_update());
        assert_eq!(sandbox.app_id(), None);
    }

    #[test]
    fn no_marker_means_no_sandbox_even_with_flatpak_id_set() {
        let sandbox = Sandbox::detect(
            &marker(false),
            &env(&[("FLATPAK_ID", "com.doomdecklauncher.DoomDeckLauncher")]),
        );
        assert_eq!(sandbox, Sandbox::none());
        assert!(!sandbox.must_extract_engine());
        assert!(sandbox.allows_self_update());
        assert_eq!(sandbox.app_id(), None);
    }

    #[test]
    fn the_app_id_comes_from_flatpak_id() {
        let sandbox = Sandbox::detect(
            &marker(true),
            &env(&[("FLATPAK_ID", "com.doomdecklauncher.DoomDeckLauncher")]),
        );
        assert_eq!(
            sandbox.app_id(),
            Some("com.doomdecklauncher.DoomDeckLauncher")
        );
    }

    #[test]
    fn an_empty_or_multiline_flatpak_id_is_ignored() {
        for id in ["", "  ", "a\nb"] {
            let sandbox = Sandbox::detect(&marker(true), &env(&[("FLATPAK_ID", id)]));
            assert!(sandbox.is_flatpak());
            assert_eq!(sandbox.app_id(), None, "{id:?}");
        }
    }

    #[test]
    fn only_the_documented_marker_path_is_probed() {
        let seen = std::cell::RefCell::new(Vec::<PathBuf>::new());
        let probe = |path: &Path| {
            seen.borrow_mut().push(path.to_path_buf());
            false
        };
        let _ = Sandbox::detect(&probe, &env(&[]));
        assert_eq!(*seen.borrow(), [PathBuf::from("/.flatpak-info")]);
    }
}
