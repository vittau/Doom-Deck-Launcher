//! Resolution of DDL's per-user directories according to the XDG Base
//! Directory specification (spec 001, R2).
//!
//! The resolver is a pure function of its environment: it never reads the real
//! process environment itself, so tests need no environment manipulation and
//! the caller decides where the values come from. See [`Env`].
//!
//! Owned directories (all joined with [`APP_DIR`]):
//!
//! | Directory | Variable             | Fallback             |
//! | --------- | -------------------- | -------------------- |
//! | config    | `XDG_CONFIG_HOME`    | `$HOME/.config`      |
//! | data      | `XDG_DATA_HOME`      | `$HOME/.local/share` |
//! | cache     | `XDG_CACHE_HOME`     | `$HOME/.cache`       |
//! | state     | `XDG_STATE_HOME`     | `$HOME/.local/state` |
//!
//! Per the specification, an XDG variable is only honoured when it is set to a
//! non-empty **absolute** path; anything else (unset, empty or relative) is
//! equivalent to "unset" and the `$HOME`-based fallback applies. `HOME` itself
//! must be set to an absolute path: when it is missing, resolution fails with
//! [`Error::MissingHome`] instead of panicking or silently inventing a path.

use std::fmt;
use std::path::{Path, PathBuf};

/// Name appended to every resolved base directory, e.g. `~/.config/ddl`.
pub const APP_DIR: &str = "ddl";

/// A source of environment variables.
///
/// Implemented for every `Fn(&str) -> Option<String>`, so callers can pass a
/// captured map (tests) or the real process environment
/// (`|key| std::env::var(key).ok()`).
pub trait Env {
    /// Returns the value of `key`, or `None` when it is not set.
    fn var(&self, key: &str) -> Option<String>;
}

impl<F> Env for F
where
    F: Fn(&str) -> Option<String>,
{
    fn var(&self, key: &str) -> Option<String> {
        self(key)
    }
}

/// DDL's resolved directories. Each path already ends in [`APP_DIR`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    /// Settings, profiles and DSDA configuration (`~/.config/ddl`).
    pub config: PathBuf,
    /// Engines, installed content and catalogs (`~/.local/share/ddl`).
    pub data: PathBuf,
    /// Downloads, thumbnails and cached API responses (`~/.cache/ddl`).
    pub cache: PathBuf,
    /// Logs and other state that should survive a reboot (`~/.local/state/ddl`).
    pub state: PathBuf,
}

/// Why directory resolution failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// `HOME` is unset, empty or not an absolute path, so the XDG fallbacks
    /// (which are all defined relative to `$HOME`) cannot be computed.
    MissingHome,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHome => f.write_str(
                "HOME is not set to an absolute path, so DDL's XDG directories cannot be resolved",
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Resolves DDL's directories from `env`.
///
/// # Errors
///
/// Returns [`Error::MissingHome`] when `HOME` is not set to an absolute path.
pub fn resolve(env: &impl Env) -> Result<Paths, Error> {
    let home = absolute_var(env, "HOME").ok_or(Error::MissingHome)?;
    Ok(Paths {
        config: dir(env, "XDG_CONFIG_HOME", &home, ".config"),
        data: dir(env, "XDG_DATA_HOME", &home, ".local/share"),
        cache: dir(env, "XDG_CACHE_HOME", &home, ".cache"),
        state: dir(env, "XDG_STATE_HOME", &home, ".local/state"),
    })
}

/// Resolves `var` when it holds a usable absolute path, otherwise the
/// `$HOME`-relative `fallback`, and appends [`APP_DIR`] to either.
fn dir(env: &impl Env, var: &str, home: &Path, fallback: &str) -> PathBuf {
    match absolute_var(env, var) {
        Some(base) => base.join(APP_DIR),
        None => home.join(fallback).join(APP_DIR),
    }
}

/// Returns the value of `var` as a [`PathBuf`] when it is non-empty and
/// absolute. Empty and relative values are treated as "unset" (XDG spec).
fn absolute_var(env: &impl Env, var: &str) -> Option<PathBuf> {
    let value = env.var(var)?;
    if value.is_empty() {
        return None;
    }
    let path = Path::new(&value);
    path.is_absolute().then(|| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    /// Builds an [`Env`] closure from key/value pairs, standing in for the
    /// process environment (spec 001, A2: tests need no real environment).
    fn env(pairs: &[(&str, &str)]) -> impl Env {
        let map: BTreeMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    fn home_only() -> impl Env {
        env(&[("HOME", "/home/deck")])
    }

    #[test]
    fn absolute_xdg_variables_are_used_verbatim_and_joined_with_ddl() {
        let paths = resolve(&env(&[
            ("HOME", "/home/deck"),
            ("XDG_CONFIG_HOME", "/tmp/xdg-config"),
            ("XDG_DATA_HOME", "/tmp/xdg-data"),
            ("XDG_CACHE_HOME", "/tmp/xdg-cache"),
            ("XDG_STATE_HOME", "/tmp/xdg-state"),
        ]))
        .expect("HOME is absolute");

        assert_eq!(paths.config, PathBuf::from("/tmp/xdg-config/ddl"));
        assert_eq!(paths.data, PathBuf::from("/tmp/xdg-data/ddl"));
        assert_eq!(paths.cache, PathBuf::from("/tmp/xdg-cache/ddl"));
        assert_eq!(paths.state, PathBuf::from("/tmp/xdg-state/ddl"));
    }

    #[test]
    fn relative_xdg_variables_are_ignored() {
        let paths = resolve(&env(&[
            ("HOME", "/home/deck"),
            ("XDG_CONFIG_HOME", "relative/config"),
            ("XDG_DATA_HOME", "./relative-data"),
            ("XDG_CACHE_HOME", "relative-cache"),
            ("XDG_STATE_HOME", "relative-state"),
        ]))
        .expect("HOME is absolute");

        assert_eq!(paths.config, PathBuf::from("/home/deck/.config/ddl"));
        assert_eq!(paths.data, PathBuf::from("/home/deck/.local/share/ddl"));
        assert_eq!(paths.cache, PathBuf::from("/home/deck/.cache/ddl"));
        assert_eq!(paths.state, PathBuf::from("/home/deck/.local/state/ddl"));
    }

    #[test]
    fn unset_xdg_variables_fall_back_to_home() {
        let paths = resolve(&home_only()).expect("HOME is absolute");

        assert_eq!(paths.config, PathBuf::from("/home/deck/.config/ddl"));
        assert_eq!(paths.data, PathBuf::from("/home/deck/.local/share/ddl"));
        assert_eq!(paths.cache, PathBuf::from("/home/deck/.cache/ddl"));
        assert_eq!(paths.state, PathBuf::from("/home/deck/.local/state/ddl"));
    }

    #[test]
    fn empty_xdg_variables_are_ignored() {
        let paths = resolve(&env(&[
            ("HOME", "/home/deck"),
            ("XDG_CONFIG_HOME", ""),
            ("XDG_DATA_HOME", ""),
            ("XDG_CACHE_HOME", ""),
            ("XDG_STATE_HOME", ""),
        ]))
        .expect("HOME is absolute");

        assert_eq!(paths.config, PathBuf::from("/home/deck/.config/ddl"));
        assert_eq!(paths.data, PathBuf::from("/home/deck/.local/share/ddl"));
    }

    #[test]
    fn mixed_absolute_and_unset_variables_are_handled_independently() {
        let paths = resolve(&env(&[
            ("HOME", "/home/deck"),
            ("XDG_CONFIG_HOME", "/etc/ddl-config"),
            ("XDG_DATA_HOME", "not-absolute"),
        ]))
        .expect("HOME is absolute");

        assert_eq!(paths.config, PathBuf::from("/etc/ddl-config/ddl"));
        assert_eq!(paths.data, PathBuf::from("/home/deck/.local/share/ddl"));
        assert_eq!(paths.cache, PathBuf::from("/home/deck/.cache/ddl"));
        assert_eq!(paths.state, PathBuf::from("/home/deck/.local/state/ddl"));
    }

    #[test]
    fn missing_home_is_an_error() {
        let error = resolve(&env(&[])).expect_err("HOME is missing");
        assert_eq!(error, Error::MissingHome);
        assert!(error.to_string().contains("HOME"));
    }

    #[test]
    fn empty_home_is_an_error() {
        assert_eq!(resolve(&env(&[("HOME", "")])), Err(Error::MissingHome));
    }

    #[test]
    fn relative_home_is_an_error() {
        assert_eq!(
            resolve(&env(&[("HOME", "home/deck")])),
            Err(Error::MissingHome)
        );
    }

    #[test]
    fn home_is_required_even_when_every_xdg_variable_is_absolute() {
        // Documented behaviour (module docs): DDL always needs HOME, so a
        // quirky environment without it fails loudly instead of half-working.
        assert_eq!(
            resolve(&env(&[
                ("XDG_CONFIG_HOME", "/tmp/config"),
                ("XDG_DATA_HOME", "/tmp/data"),
                ("XDG_CACHE_HOME", "/tmp/cache"),
                ("XDG_STATE_HOME", "/tmp/state"),
            ])),
            Err(Error::MissingHome)
        );
    }
}
