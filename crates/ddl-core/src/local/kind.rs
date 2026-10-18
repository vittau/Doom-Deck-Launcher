//! Content-based detection of the arbitrary user files DSDA-Doom accepts
//! (spec 026, R1).
//!
//! DDL never trusts an extension to decide what a file *is*: it reads bounded
//! headers and classifies the content. The extension still decides **where**
//! the file goes, because that is exactly what the engine does: `dsda_Loadfiles`
//! routes a `-file` value by a case-insensitive suffix and calls
//! `I_Error("File type … is not supported")` for anything else. So a value is
//! accepted only when its content kind and its suffix agree.
//!
//! Accepted, confirmed against upstream tag `v0.30.1`:
//!
//! * `.wad`, `.lmp` → the WAD/lump loader, so the content must be a WAD;
//! * `.zip` → the ZIP loader, so the content must be a ZIP;
//! * `.deh`, `.bex` → the patch queue, so the content must be bounded text.
//!
//! `.pk3` is **rejected**: it has no branch in `dsda_Loadfiles`, so the engine
//! would abort. See `docs/research/dsda-engine.md` §1.
//!
//! Source: `prboom2/src/d_main.c:1770-1820` (`dsda_Loadfiles`),
//! `prboom2/src/dsda/utility.c:103-106` (`dsda_HasFileExt`, `strcasecmp`),
//! `prboom2/src/SDL/i_system.c:641-670` (`I_RequireAnyFile`, `I_RequireDeh`),
//! pinned to tag `v0.30.1`.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::iwads::identify::sha256_file;

/// Largest user file DDL accepts, in bytes.
///
/// A WAD header addresses lumps and their offsets with 32-bit fields, so no
/// single WAD can exceed 4 GiB; DDL documents the same ceiling as its cap
/// (**Proposed** DDL policy — the engine itself has no smaller documented
/// limit).
pub const MAX_USER_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// How much of a candidate patch DDL reads to decide it is text.
pub const TEXT_PREFIX_BYTES: usize = 64 * 1024;

const HEADER_LEN: u64 = 12;
const LUMP_LEN: u64 = 16;
const MAX_LUMPS: u32 = 65_536;

/// The content kind of an accepted user file (spec 026, R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A WAD (or the WAD-shaped `.lmp` the engine routes to the WAD loader).
    Wad,
    /// A ZIP archive.
    Zip,
    /// A DEH/BEX patch: bounded text the engine parses as a patch.
    Deh,
}

impl Kind {
    /// A short lowercase label for storage and output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wad => "wad",
            Self::Zip => "zip",
            Self::Deh => "deh",
        }
    }

    /// Whether the engine routes this kind to the `-file` list.
    #[must_use]
    pub const fn is_load_file(self) -> bool {
        matches!(self, Self::Wad | Self::Zip)
    }

    /// Whether the engine routes this kind to the `-deh` patch list.
    #[must_use]
    pub const fn is_patch(self) -> bool {
        matches!(self, Self::Deh)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The option `dsda_Loadfiles` routes a suffix to (research §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    /// `.wad` / `.lmp`, loaded through the WAD loader.
    Wad,
    /// `.zip`, loaded through the ZIP loader.
    Zip,
    /// `.deh` / `.bex`, queued as a patch.
    Patch,
}

/// The suffix route, case-insensitively (`dsda_HasFileExt`), or `None` when the
/// engine has no branch for it and would abort.
fn route_for(path: &Path) -> Option<Route> {
    let extension = path.extension()?.to_str()?;
    if extension.eq_ignore_ascii_case("wad") || extension.eq_ignore_ascii_case("lmp") {
        Some(Route::Wad)
    } else if extension.eq_ignore_ascii_case("zip") {
        Some(Route::Zip)
    } else if extension.eq_ignore_ascii_case("deh") || extension.eq_ignore_ascii_case("bex") {
        Some(Route::Patch)
    } else {
        None
    }
}

/// One user file DDL recorded: referenced in place by absolute path, with the
/// size, SHA-256 and content kind needed to recognise it at launch (spec 026).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserFile {
    /// Absolute path of the player's file. DDL never copies or moves it.
    pub path: PathBuf,
    /// Size in bytes when DDL read it.
    pub size: u64,
    /// Lowercase-hex SHA-256 when DDL read it.
    pub sha256: String,
    /// Content kind, detected from the file (spec 026, R1).
    pub kind: Kind,
}

impl UserFile {
    /// Reads and identifies the file at `path`.
    ///
    /// Rejects non-absolute paths, missing files, directories, devices and
    /// sockets, symlinks that resolve to a non-regular file, files above
    /// [`MAX_USER_FILE_BYTES`], content that is not a WAD, ZIP or bounded text,
    /// and a suffix the engine's `-file` routing would reject.
    ///
    /// # Errors
    ///
    /// Returns a [`FileError`] describing the first problem found.
    pub fn read(path: &Path) -> Result<Self, FileError> {
        if !path.is_absolute() {
            return Err(FileError::NotAbsolute(path.to_path_buf()));
        }
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(FileError::Missing(path.to_path_buf()));
            }
            Err(error) => {
                return Err(FileError::Unreadable {
                    path: path.to_path_buf(),
                    error,
                });
            }
        };
        if !metadata.is_file() {
            // A directory, device, socket, or a symlink resolving to one.
            return Err(FileError::NotRegular(path.to_path_buf()));
        }
        let size = metadata.len();
        if size > MAX_USER_FILE_BYTES {
            return Err(FileError::Oversize {
                path: path.to_path_buf(),
                size,
                limit: MAX_USER_FILE_BYTES,
            });
        }

        let kind = detect_kind(path, size)?;
        if let Some(route) = route_for(path) {
            let expected = match route {
                Route::Wad => Kind::Wad,
                Route::Zip => Kind::Zip,
                Route::Patch => Kind::Deh,
            };
            if expected != kind {
                return Err(FileError::KindMismatch {
                    path: path.to_path_buf(),
                    suffix: path
                        .extension()
                        .map(|e| e.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    kind,
                });
            }
        } else {
            let suffix = path
                .extension()
                .map(|e| e.to_string_lossy().into_owned())
                .unwrap_or_default();
            return Err(FileError::UnsupportedSuffix {
                path: path.to_path_buf(),
                suffix,
            });
        }

        let sha256 = sha256_file(path).map_err(|error| FileError::Unreadable {
            path: path.to_path_buf(),
            error,
        })?;
        Ok(Self {
            path: path.to_path_buf(),
            size,
            sha256,
            kind,
        })
    }

    /// Whether the file still exists and matches the recorded size and SHA-256
    /// (spec 026, R4). Never substitutes a different file.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError::Missing`] when the path is gone or no longer a
    /// regular file, [`IdentityError::Changed`] when the size or hash differs,
    /// and [`IdentityError::Unreadable`] when it cannot be read.
    pub fn verify(&self) -> Result<(), IdentityError> {
        let metadata = match fs::metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(IdentityError::Missing {
                    path: self.path.clone(),
                });
            }
            Err(error) => {
                return Err(IdentityError::Unreadable {
                    path: self.path.clone(),
                    error,
                });
            }
        };
        if !metadata.is_file() || metadata.len() != self.size {
            return Err(IdentityError::Changed {
                path: self.path.clone(),
            });
        }
        let sha256 = sha256_file(&self.path).map_err(|error| IdentityError::Unreadable {
            path: self.path.clone(),
            error,
        })?;
        if !sha256.eq_ignore_ascii_case(&self.sha256) {
            return Err(IdentityError::Changed {
                path: self.path.clone(),
            });
        }
        Ok(())
    }

    /// Re-reads the file and returns its new identity, for the player's
    /// explicit "Use the changed file" confirmation (spec 026, R4).
    ///
    /// # Errors
    ///
    /// As [`UserFile::read`].
    pub fn rehash(&self) -> Result<Self, FileError> {
        Self::read(&self.path)
    }
}

/// Whether the file at `path` begins with the `IWAD` magic, for the custom
/// setup's user-file IWAD slot (spec 026, R2).
///
/// # Errors
///
/// Returns a [`FileError`] when the file cannot be read.
pub fn is_iwad_header(path: &Path) -> Result<bool, FileError> {
    let mut magic = [0_u8; 4];
    let mut file = File::open(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            FileError::Missing(path.to_path_buf())
        } else {
            FileError::Unreadable {
                path: path.to_path_buf(),
                error,
            }
        }
    })?;
    match file.read_exact(&mut magic) {
        Ok(()) => Ok(&magic == b"IWAD"),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(FileError::Unreadable {
            path: path.to_path_buf(),
            error,
        }),
    }
}

/// Classifies a file by content, ignoring its name.
///
/// # Errors
///
/// Returns a [`FileError`] when the file cannot be read or its content is not
/// an accepted kind.
pub fn detect_kind(path: &Path, size: u64) -> Result<Kind, FileError> {
    let mut prefix = vec![0_u8; TEXT_PREFIX_BYTES.min(usize::try_from(size).unwrap_or(usize::MAX))];
    let mut file = File::open(path).map_err(|error| FileError::Unreadable {
        path: path.to_path_buf(),
        error,
    })?;
    let read = file
        .read(&mut prefix)
        .map_err(|error| FileError::Unreadable {
            path: path.to_path_buf(),
            error,
        })?;
    prefix.truncate(read);

    if prefix.starts_with(b"IWAD") || prefix.starts_with(b"PWAD") {
        return check_wad(path, size);
    }
    if prefix.starts_with(b"PK\x03\x04") || prefix.starts_with(b"PK\x05\x06") {
        return Ok(Kind::Zip);
    }
    if is_bounded_text(&prefix) {
        return Ok(Kind::Deh);
    }
    Err(FileError::UnsupportedContent {
        path: path.to_path_buf(),
    })
}

/// Validates a WAD header and its lump directory.
fn check_wad(path: &Path, size: u64) -> Result<Kind, FileError> {
    if size < HEADER_LEN {
        return Err(FileError::InvalidContent {
            path: path.to_path_buf(),
            detail: "WAD header is truncated".to_owned(),
        });
    }
    let mut raw = [0_u8; HEADER_LEN as usize];
    let mut file = File::open(path).map_err(|error| FileError::Unreadable {
        path: path.to_path_buf(),
        error,
    })?;
    file.read_exact(&mut raw)
        .map_err(|error| FileError::Unreadable {
            path: path.to_path_buf(),
            error,
        })?;
    let numlumps = i32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
    let infotableofs = i32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]);
    if numlumps < 0 || infotableofs < 0 {
        return Err(FileError::InvalidContent {
            path: path.to_path_buf(),
            detail: "WAD header declares a negative count or offset".to_owned(),
        });
    }
    let numlumps = numlumps as u32;
    if numlumps > MAX_LUMPS {
        return Err(FileError::InvalidContent {
            path: path.to_path_buf(),
            detail: format!("WAD declares {numlumps} lumps, over DDL's {MAX_LUMPS}-lump limit"),
        });
    }
    let directory_end = u64::from(infotableofs as u32) + u64::from(numlumps) * LUMP_LEN;
    if u64::from(infotableofs as u32) < HEADER_LEN || directory_end > size {
        return Err(FileError::InvalidContent {
            path: path.to_path_buf(),
            detail: "WAD lump directory does not fit in the file".to_owned(),
        });
    }
    Ok(Kind::Wad)
}

/// Whether a prefix looks like bounded human-readable text: no NUL byte and at
/// most a negligible fraction of non-whitespace control bytes.
fn is_bounded_text(prefix: &[u8]) -> bool {
    if prefix.is_empty() {
        return false;
    }
    if prefix.contains(&0) {
        return false;
    }
    let control = prefix
        .iter()
        .filter(|byte| **byte < 0x20 && !matches!(**byte, b'\t' | b'\n' | b'\r' | 0x0c))
        .count();
    control * 100 <= prefix.len()
}

/// Why a user file was not accepted (spec 026, R1).
#[derive(Debug)]
pub enum FileError {
    /// The path is not absolute; user files are referenced in place.
    NotAbsolute(PathBuf),
    /// The file does not exist.
    Missing(PathBuf),
    /// The path is a directory, device, socket, or a symlink resolving to one.
    NotRegular(PathBuf),
    /// The file exceeds [`MAX_USER_FILE_BYTES`].
    Oversize {
        /// The offending path.
        path: PathBuf,
        /// The file's size.
        size: u64,
        /// The limit that was exceeded.
        limit: u64,
    },
    /// The file could not be read.
    Unreadable {
        /// The offending path.
        path: PathBuf,
        /// The underlying error.
        error: io::Error,
    },
    /// The content is not a WAD, ZIP or DEH/BEX patch.
    UnsupportedContent {
        /// The offending path.
        path: PathBuf,
    },
    /// A WAD-shaped file failed its header or directory sanity check.
    InvalidContent {
        /// The offending path.
        path: PathBuf,
        /// What is wrong.
        detail: String,
    },
    /// The content kind does not match the suffix the engine would route.
    KindMismatch {
        /// The offending path.
        path: PathBuf,
        /// The suffix the engine would route.
        suffix: String,
        /// The content kind found.
        kind: Kind,
    },
    /// The suffix has no branch in `dsda_Loadfiles`, so the engine would abort.
    UnsupportedSuffix {
        /// The offending path.
        path: PathBuf,
        /// The suffix in question.
        suffix: String,
    },
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAbsolute(path) => write!(
                f,
                "`{}` is not an absolute path; DDL references your file in place",
                path.display()
            ),
            Self::Missing(path) => write!(f, "`{}` does not exist", path.display()),
            Self::NotRegular(path) => write!(
                f,
                "`{}` is a directory, device, socket or a link to one; only regular files are \
                 accepted",
                path.display()
            ),
            Self::Oversize { path, size, limit } => write!(
                f,
                "`{}` is {size} bytes, over DDL's {limit}-byte limit",
                path.display()
            ),
            Self::Unreadable { path, error } => {
                write!(f, "could not read `{}`: {error}", path.display())
            }
            Self::UnsupportedContent { path } => write!(
                f,
                "`{}` is not a WAD, ZIP or DEH/BEX file; DSDA-Doom's `-file`/`-deh` would not \
                 accept it",
                path.display()
            ),
            Self::InvalidContent { path, detail } => {
                write!(f, "`{}` failed its content check: {detail}", path.display())
            }
            Self::KindMismatch { path, suffix, kind } => write!(
                f,
                "`{}` has a `.{suffix}` suffix, so DSDA-Doom would read it as a different kind, \
                 but its content is {kind}",
                path.display()
            ),
            Self::UnsupportedSuffix { path, suffix } => {
                if suffix.is_empty() {
                    write!(
                        f,
                        "`{}` has no extension; DSDA-Doom would search for .wad, .lmp, .zip, .deh \
                         or .bex, so name the file with its real extension",
                        path.display()
                    )
                } else {
                    write!(
                        f,
                        "`{}` ends in `.{suffix}`, which DSDA-Doom's `-file` rejects; accepted \
                         extensions are .wad, .lmp, .zip, .deh and .bex",
                        path.display()
                    )
                }
            }
        }
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unreadable { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// Why a recorded user file no longer matches (spec 026, R4).
#[derive(Debug)]
pub enum IdentityError {
    /// The file is gone or is no longer a regular file.
    Missing {
        /// The missing path.
        path: PathBuf,
    },
    /// The size or SHA-256 differs from what DDL recorded.
    Changed {
        /// The changed path.
        path: PathBuf,
    },
    /// The file exists but could not be read.
    Unreadable {
        /// The offending path.
        path: PathBuf,
        /// The underlying error.
        error: io::Error,
    },
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { path } => write!(f, "`{}` is missing", path.display()),
            Self::Changed { path } => write!(
                f,
                "`{}` changed since DDL recorded it; confirm the change to use the new file",
                path.display()
            ),
            Self::Unreadable { path, error } => {
                write!(f, "could not read `{}`: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for IdentityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unreadable { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    /// A minimal valid WAD: a 12-byte header plus an empty lump directory.
    fn wad_bytes(magic: &[u8; 4], lumps: &[&str]) -> Vec<u8> {
        let mut directory = Vec::new();
        for name in lumps {
            directory.extend_from_slice(&0_i32.to_le_bytes());
            directory.extend_from_slice(&0_i32.to_le_bytes());
            let mut padded = [0_u8; 8];
            let bytes = name.as_bytes();
            let len = bytes.len().min(8);
            padded[..len].copy_from_slice(&bytes[..len]);
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
        fs::write(&path, bytes).expect("write fixture");
        path
    }

    #[test]
    fn wad_content_is_a_wad_whatever_the_extension() {
        let dir = TempDir::new().unwrap();
        for name in ["map.wad", "map.lmp"] {
            let path = write(&dir, name, &wad_bytes(b"PWAD", &["MAP01"]));
            let file = UserFile::read(&path).expect("accepted");
            assert_eq!(file.kind, Kind::Wad);
            assert_eq!(file.size, (12 + 16) as u64);
            assert_eq!(file.sha256.len(), 64);
        }
        // A `.wad` whose content is really a ZIP is rejected, because the engine
        // would hand it to the WAD loader.
        let mismatched = write(&dir, "fake.wad", b"PK\x03\x04 rest is ignored");
        assert!(matches!(
            UserFile::read(&mismatched),
            Err(FileError::KindMismatch {
                kind: Kind::Zip,
                ..
            })
        ));
    }

    #[test]
    fn zip_content_is_a_zip_and_pk3_is_rejected() {
        let dir = TempDir::new().unwrap();
        let zip = write(&dir, "maps.zip", b"PK\x03\x04padding");
        assert_eq!(UserFile::read(&zip).unwrap().kind, Kind::Zip);
        // `.pk3` is a ZIP by content, but the engine has no branch for the
        // suffix and would abort with I_Error.
        let pk3 = write(&dir, "maps.pk3", b"PK\x03\x04padding");
        assert!(matches!(
            UserFile::read(&pk3),
            Err(FileError::UnsupportedSuffix { .. })
        ));
    }

    #[test]
    fn bounded_text_is_a_patch() {
        let dir = TempDir::new().unwrap();
        let deh = write(
            &dir,
            "fix.deh",
            b"Patch File for DeHackEd v3.0\n\nThing 1 (Trooper)\nHit points = 30\n",
        );
        let file = UserFile::read(&deh).unwrap();
        assert_eq!(file.kind, Kind::Deh);
        assert!(file.kind.is_patch());

        let bex = write(&dir, "fix.bex", b"Patch File for DeHackEd v3.0\n[BEX]");
        assert_eq!(UserFile::read(&bex).unwrap().kind, Kind::Deh);
    }

    #[test]
    fn binary_and_unknown_content_is_rejected() {
        let dir = TempDir::new().unwrap();
        let binary = write(&dir, "data.deh", b"\x00\x01\x02\x03");
        assert!(matches!(
            UserFile::read(&binary),
            Err(FileError::UnsupportedContent { .. })
        ));
        let odd = write(&dir, "file.xyz", b"just some text");
        assert!(matches!(
            UserFile::read(&odd),
            Err(FileError::UnsupportedSuffix { .. })
        ));
        let no_ext = write(&dir, "noextension", b"just some text");
        assert!(matches!(
            UserFile::read(&no_ext),
            Err(FileError::UnsupportedSuffix { .. })
        ));
    }

    #[test]
    fn truncated_and_oversized_wads_are_rejected() {
        let dir = TempDir::new().unwrap();
        let short = write(&dir, "short.wad", b"IWAD");
        assert!(matches!(
            UserFile::read(&short),
            Err(FileError::InvalidContent { .. })
        ));

        let mut bogus = Vec::new();
        bogus.extend_from_slice(b"IWAD");
        bogus.extend_from_slice(&100_i32.to_le_bytes());
        bogus.extend_from_slice(&12_i32.to_le_bytes());
        let bad = write(&dir, "bad.wad", &bogus);
        assert!(matches!(
            UserFile::read(&bad),
            Err(FileError::InvalidContent { .. })
        ));

        let path = dir.path().join("huge.wad");
        let file = File::create(&path).unwrap();
        file.set_len(MAX_USER_FILE_BYTES + 1).unwrap();
        assert!(matches!(
            UserFile::read(&path),
            Err(FileError::Oversize { .. })
        ));
    }

    #[test]
    fn directories_relative_paths_and_missing_files_are_rejected() {
        let dir = TempDir::new().unwrap();
        assert!(matches!(
            UserFile::read(Path::new("relative.wad")),
            Err(FileError::NotAbsolute(_))
        ));
        assert!(matches!(
            UserFile::read(dir.path()),
            Err(FileError::NotRegular(_))
        ));
        assert!(matches!(
            UserFile::read(&dir.path().join("gone.wad")),
            Err(FileError::Missing(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_directory_is_rejected_but_a_symlink_to_a_file_is_kept() {
        let dir = TempDir::new().unwrap();
        let target_dir = dir.path().join("real-dir");
        fs::create_dir_all(&target_dir).unwrap();
        let link_dir = dir.path().join("link.wad");
        std::os::unix::fs::symlink(&target_dir, &link_dir).unwrap();
        assert!(matches!(
            UserFile::read(&link_dir),
            Err(FileError::NotRegular(_))
        ));

        let target = write(&dir, "real.wad", &wad_bytes(b"IWAD", &["PLAYPAL"]));
        let link = dir.path().join("link-file.wad");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert_eq!(UserFile::read(&link).unwrap().kind, Kind::Wad);
    }

    #[test]
    fn identity_check_reports_missing_and_changed_files() {
        let dir = TempDir::new().unwrap();
        let path = write(&dir, "map.wad", &wad_bytes(b"PWAD", &["MAP01"]));
        let file = UserFile::read(&path).unwrap();
        assert!(file.verify().is_ok());

        fs::write(&path, wad_bytes(b"PWAD", &["MAP02"])).unwrap();
        assert!(matches!(file.verify(), Err(IdentityError::Changed { .. })));

        fs::remove_file(&path).unwrap();
        assert!(matches!(file.verify(), Err(IdentityError::Missing { .. })));

        // Re-reading after an explicit confirmation records the new identity.
        let path = write(&dir, "map.wad", &wad_bytes(b"PWAD", &["MAP02"]));
        let refreshed = file.rehash().unwrap();
        assert_ne!(refreshed.sha256, file.sha256);
        assert!(refreshed.verify().is_ok());
        let _ = path;
    }

    #[test]
    fn iwad_header_is_detected_from_content() {
        let dir = TempDir::new().unwrap();
        let iwad = write(&dir, "my.wad", &wad_bytes(b"IWAD", &["PLAYPAL"]));
        let pwad = write(&dir, "mod.wad", &wad_bytes(b"PWAD", &["MAP01"]));
        assert!(is_iwad_header(&iwad).unwrap());
        assert!(!is_iwad_header(&pwad).unwrap());
    }
}
