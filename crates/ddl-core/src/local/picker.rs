//! Optional local-picker preference, separate from app and view state (spec 106).
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
struct Preference {
    schema_version: u32,
    folder: PathBuf,
}
const NAME: &str = "local-picker.json";
/// Load the optional preference. Failure is diagnostic only; callers use Home.
#[must_use]
pub fn remembered(config: &Path, home: &Path) -> PathBuf {
    let read = || -> Result<PathBuf, String> {
        let path = config.join(NAME);
        let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 16 * 1024 {
            return Err("picker preference is not a bounded regular file".into());
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|e| e.to_string())?
            .take(16 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 {
            return Err("picker preference too large".into());
        }
        let p: Preference = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if p.schema_version != 1 || !usable(&p.folder) {
            return Err("picker preference unavailable or unsupported".into());
        }
        Ok(p.folder)
    };
    read().unwrap_or_else(|error| {
        tracing::debug!(%error, "using Home for local picker");
        home.to_path_buf()
    })
}
/// Whether a folder can actually be enumerated.
#[must_use]
pub fn usable(path: &Path) -> bool {
    path.is_absolute() && std::fs::read_dir(path).is_ok()
}
/// Save only after the caller has successfully browsed the folder.
/// # Errors
/// Returns serialization or atomic-write errors; these must remain optional.
pub fn remember(config: &Path, folder: &Path) -> io::Result<()> {
    if !usable(folder) {
        return Err(io::Error::other("folder unavailable"));
    }
    let bytes = serde_json::to_vec(&Preference {
        schema_version: 1,
        folder: folder.into(),
    })?;
    crate::idgames::write_atomic(&config.join(NAME), &bytes)
}
/// Readable filename default, bounded by the existing 32-character name UI rule.
#[must_use]
pub fn filename_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let name = stem
        .split(|c: char| c.is_whitespace() || matches!(c, '_' | '-' | '.'))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let name: String = name.chars().filter(|c| !c.is_control()).take(32).collect();
    (!name.trim().is_empty()).then(|| name.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_preference_restart_and_fallback_are_isolated() {
        let t = tempfile::tempdir().unwrap();
        let config = t.path().join("config");
        let home = t.path().join("home");
        let folder = home.join("maps");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(config.join("app.json"), b"app sentinel").unwrap();
        std::fs::write(config.join("view.json"), b"view sentinel").unwrap();
        assert_eq!(remembered(&config, &home), home);
        remember(&config, &folder).unwrap();
        assert_eq!(remembered(&config, &home), folder);
        assert!(remember(&config, &home.join("missing")).is_err());
        assert_eq!(remembered(&config, &home), folder);
        std::fs::remove_dir(&folder).unwrap();
        assert_eq!(remembered(&config, &home), home);
        for data in [
            b"invalid".to_vec(),
            br#"{"schema_version":99,"folder":"/"}"#.to_vec(),
            vec![b' '; 17000],
        ] {
            std::fs::write(config.join(NAME), data).unwrap();
            assert_eq!(remembered(&config, &home), home);
        }
        std::fs::remove_file(config.join(NAME)).unwrap();
        std::fs::create_dir(config.join(NAME)).unwrap();
        assert_eq!(remembered(&config, &home), home);
        assert_eq!(
            std::fs::read(config.join("app.json")).unwrap(),
            b"app sentinel"
        );
        assert_eq!(
            std::fs::read(config.join("view.json")).unwrap(),
            b"view sentinel"
        );
    }
    #[test]
    fn unicode_names_collapse_separators_and_bound_characters() {
        assert_eq!(
            filename_name(Path::new("/maps/  夜_é--maps.v2.wad")),
            Some("夜 é maps v2".into())
        );
        assert_eq!(filename_name(Path::new("/maps/___--.wad")), None);
        assert_eq!(
            filename_name(Path::new(&format!("/{}.wad", "夜".repeat(60))))
                .unwrap()
                .chars()
                .count(),
            32
        );
    }
}

#[cfg(all(test, unix))]
mod unix_tests {
    use super::*;
    #[test]
    fn unavailable_folder_and_nonregular_preference_fall_back() {
        use std::os::unix::{fs::PermissionsExt, net::UnixListener};
        let t = tempfile::tempdir().unwrap();
        let config = t.path().join("config");
        let home = t.path().join("home");
        let folder = home.join("card");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::create_dir_all(&folder).unwrap();
        remember(&config, &folder).unwrap();
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o000)).unwrap();
        assert_eq!(remembered(&config, &home), home);
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::remove_file(config.join(NAME)).unwrap();
        let _socket = UnixListener::bind(config.join(NAME)).unwrap();
        assert_eq!(remembered(&config, &home), home);
    }
}
