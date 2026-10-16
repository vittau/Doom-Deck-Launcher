//! Safe ZIP extraction by explicit member list (spec 006, R4/R5).
//!
//! Nothing is extracted unless the caller names it — the user never unzips
//! anything and DDL never auto-loads whatever happens to be in an archive
//! (D-010, `OVERVIEW.md` §7). A member path may cross **nested** archives with
//! `!/`, up to [`ExtractLimits::max_nesting_depth`] levels, which is how
//! AUGER;ZENITH ships its add-ons (`DBP37_Addons.zip!/DBP37_AlternateBlaster.wad`).
//!
//! Safety rules enforced here (spec 006, R5):
//!
//! - Member names are matched **exactly** and case-sensitively; a miss reports
//!   the closest case-insensitive match.
//! - Only **stored** and **deflate** entries are supported; anything else is a
//!   typed error.
//! - Member names must be relative, with no `..`, backslash, drive prefix or
//!   NUL; symlink entries are rejected outright.
//! - Total uncompressed size, per-member size, member count, compression ratio
//!   and nesting depth are all bounded (zip-bomb guard).
//! - Content is written into a fresh staging directory under DDL's data dir and
//!   only moved into place by [`Extraction::commit`], which uses `rename`.
//!   Extracted files never get an executable bit, and nothing is ever executed.

use std::fs::{self, File, OpenOptions};
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256 as Sha256Digest};
use zip::{CompressionMethod, ZipArchive};

use super::error::{Error, LimitKind};
use super::hash::Sha256;

/// Read size while decompressing a member.
const EXTRACT_BUFFER: usize = 64 * 1024;

/// The safety bounds applied to an archive (spec 006, R5). The defaults are
/// generous enough for real mapsets (AUGER;ZENITH's PWAD is ~49 MB) while still
/// stopping a zip bomb; callers may tighten them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractLimits {
    /// Maximum uncompressed bytes across the whole extraction.
    pub max_total_uncompressed: u64,
    /// Maximum uncompressed bytes of a single member.
    pub max_member_uncompressed: u64,
    /// Maximum number of entries in any one archive.
    pub max_member_count: usize,
    /// Maximum uncompressed-to-compressed ratio for a member.
    pub max_compression_ratio: u64,
    /// Maximum number of nested archives crossed by a `!/` member path.
    pub max_nesting_depth: u8,
}

impl Default for ExtractLimits {
    fn default() -> Self {
        Self {
            max_total_uncompressed: 4 * 1024 * 1024 * 1024,
            max_member_uncompressed: 2 * 1024 * 1024 * 1024,
            max_member_count: 100_000,
            max_compression_ratio: 200,
            max_nesting_depth: 2,
        }
    }
}

/// A member the caller wants out of a package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRequest {
    /// Exact, case-sensitive path inside the package, e.g. `DBP37_AUGZEN.wad`
    /// or `DBP37_Addons.zip!/DBP37_AlternateBlaster.wad`.
    pub path: String,
    /// Optional exact size to verify.
    pub expected_size: Option<u64>,
    /// Optional SHA-256 to verify.
    pub expected_sha256: Option<Sha256>,
}

impl MemberRequest {
    /// A request for `path` with no size or hash check.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            expected_size: None,
            expected_sha256: None,
        }
    }

    /// Adds an expected size.
    #[must_use]
    pub fn with_size(mut self, size: u64) -> Self {
        self.expected_size = Some(size);
        self
    }

    /// Adds an expected SHA-256.
    #[must_use]
    pub fn with_sha256(mut self, sha256: Sha256) -> Self {
        self.expected_sha256 = Some(sha256);
        self
    }
}

/// A member that was extracted into the staging directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMember {
    /// The path the caller asked for (including any `!/` nesting).
    pub requested: String,
    /// Where the member sits inside the staging directory.
    pub relative_path: PathBuf,
    /// Uncompressed size in bytes.
    pub size: u64,
    /// SHA-256 of the extracted bytes.
    pub sha256: Sha256,
}

/// The result of one extraction, staged and ready to commit.
///
/// If it is dropped without [`Extraction::commit`], the staging directory is
/// removed, so a failed or abandoned install leaves nothing behind.
#[derive(Debug)]
pub struct Extraction {
    staging_dir: PathBuf,
    members: Vec<ExtractedMember>,
    committed: bool,
}

impl Extraction {
    /// The fresh staging directory holding the extracted files.
    #[must_use]
    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }

    /// The extracted members, in request order.
    #[must_use]
    pub fn members(&self) -> &[ExtractedMember] {
        &self.members
    }

    /// Atomically moves every staged member into `dest_dir`, preserving the
    /// member's in-archive path, then removes the staging directory. Returns
    /// the final paths.
    ///
    /// The staging directory is under DDL's data dir, so this is a `rename`
    /// within one filesystem.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if a destination directory cannot be created or a
    /// rename fails.
    pub fn commit(mut self, dest_dir: &Path) -> Result<Vec<PathBuf>, Error> {
        fs::create_dir_all(dest_dir)
            .map_err(|error| io_error("create install directory", error))?;
        let mut installed = Vec::with_capacity(self.members.len());
        for member in &self.members {
            let from = self.staging_dir.join(&member.relative_path);
            let to = dest_dir.join(&member.relative_path);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| io_error("create install subdirectory", error))?;
            }
            fs::rename(&from, &to).map_err(|error| io_error("install extracted member", error))?;
            installed.push(to);
        }
        self.committed = true;
        let _ = fs::remove_dir_all(&self.staging_dir);
        Ok(installed)
    }
}

impl Drop for Extraction {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_dir_all(&self.staging_dir);
        }
    }
}

/// Extracts explicitly named members from ZIP packages.
pub struct Extractor {
    staging_root: PathBuf,
    limits: ExtractLimits,
    cancel: super::CancellationToken,
}

impl Extractor {
    /// Creates an extractor whose staging directories live under `staging_root`
    /// (DDL's data dir, e.g. `<data>/staging`).
    #[must_use]
    pub fn new(staging_root: impl Into<PathBuf>) -> Self {
        Self {
            staging_root: staging_root.into(),
            limits: ExtractLimits::default(),
            cancel: super::CancellationToken::new(),
        }
    }

    /// Cooperatively cancels between decompression chunks and archive members.
    #[must_use]
    pub fn with_cancellation(mut self, cancel: super::CancellationToken) -> Self {
        self.cancel = cancel;
        self
    }
    fn check_cancelled(&self) -> Result<(), Error> {
        if self.cancel.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }

    /// Replaces the safety limits.
    #[must_use]
    pub fn with_limits(mut self, limits: ExtractLimits) -> Self {
        self.limits = limits;
        self
    }

    /// The configured safety limits.
    #[must_use]
    pub fn limits(&self) -> ExtractLimits {
        self.limits
    }

    /// Extracts `members` from the ZIP at `package` into a fresh staging
    /// directory. The caller then [`Extraction::commit`]s it.
    ///
    /// # Errors
    ///
    /// See [`Error`]. On any failure the staging directory is removed and
    /// nothing is written outside it.
    pub fn extract(&self, package: &Path, members: &[MemberRequest]) -> Result<Extraction, Error> {
        self.check_cancelled()?;
        let staging = create_staging_dir(&self.staging_root)?;
        let mut counters = Counters { total: 0 };
        let mut extracted: Vec<ExtractedMember> = Vec::with_capacity(members.len());

        let outcome = self.extract_all(package, members, &staging, &mut counters, &mut extracted);
        match outcome {
            Ok(()) => Ok(Extraction {
                staging_dir: staging,
                members: extracted,
                committed: false,
            }),
            Err(error) => {
                let _ = fs::remove_dir_all(&staging);
                Err(error)
            }
        }
    }

    /// Extracts and immediately commits into `dest_dir`.
    ///
    /// # Errors
    ///
    /// See [`Error`]; nothing is left in `dest_dir` beyond files whose rename
    /// already succeeded.
    pub fn extract_into(
        &self,
        package: &Path,
        members: &[MemberRequest],
        dest_dir: &Path,
    ) -> Result<Vec<PathBuf>, Error> {
        self.extract(package, members)?.commit(dest_dir)
    }

    fn extract_all(
        &self,
        package: &Path,
        members: &[MemberRequest],
        staging: &Path,
        counters: &mut Counters,
        extracted: &mut Vec<ExtractedMember>,
    ) -> Result<(), Error> {
        let file = File::open(package).map_err(|error| io_error("open package", error))?;
        let mut archive =
            ZipArchive::new(file).map_err(|error| archive_corrupt(&error.to_string()))?;
        self.check_archive_limits(&archive)?;

        for member in members {
            self.check_cancelled()?;
            let segments = parse_member_path(&member.path)?;
            let depth = segments.len() - 1;
            if depth > usize::from(self.limits.max_nesting_depth) {
                return Err(Error::LimitExceeded {
                    kind: LimitKind::NestingDepth,
                    detail: format!(
                        "'{}' crosses {depth} nested archives, limit {}",
                        member.path, self.limits.max_nesting_depth
                    ),
                });
            }

            // The output path is the member's path inside its innermost
            // archive. Detect a collision before extracting so two members can
            // never fight over one staged file.
            let relative_path = PathBuf::from(segments.last().expect("at least one segment"));
            if let Some(previous) = extracted
                .iter()
                .find(|existing| existing.relative_path == relative_path)
            {
                return Err(Error::MemberConflict {
                    first: previous.requested.clone(),
                    second: member.path.clone(),
                });
            }

            let (resolved, size, sha256) =
                self.resolve(&mut archive, &segments, staging, counters)?;
            debug_assert_eq!(resolved, relative_path);

            if let Some(expected) = member.expected_size
                && size != expected
            {
                return Err(Error::SizeExceeded {
                    limit: expected,
                    got: size,
                });
            }
            if let Some(expected) = member.expected_sha256
                && sha256 != expected
            {
                return Err(Error::HashMismatch {
                    expected,
                    actual: sha256,
                });
            }

            extracted.push(ExtractedMember {
                requested: member.path.clone(),
                relative_path,
                size,
                sha256,
            });
        }
        Ok(())
    }

    /// Walks one `!/` path, reading nested archives into memory.
    fn resolve<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        segments: &[String],
        staging: &Path,
        counters: &mut Counters,
    ) -> Result<(PathBuf, u64, Sha256), Error> {
        let head = &segments[0];
        let tail = &segments[1..];
        if tail.is_empty() {
            return self.extract_file(archive, head, staging, counters);
        }

        let bytes = self.read_nested(archive, head, counters)?;
        let mut inner = ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| archive_corrupt(&error.to_string()))?;
        self.check_archive_limits(&inner)?;
        self.resolve(&mut inner, tail, staging, counters)
    }

    /// Extracts the final file of a path and hashes it.
    fn extract_file<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        name: &str,
        staging: &Path,
        counters: &mut Counters,
    ) -> Result<(PathBuf, u64, Sha256), Error> {
        let mut file = self.open_member(archive, name)?;

        let relative_path = PathBuf::from(name);
        let out_path = staging.join(&relative_path);
        if !out_path.starts_with(staging) {
            return Err(Error::UnsafeMember {
                name: name.to_owned(),
                reason: "escapes the staging directory".to_owned(),
            });
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| io_error("create staging subdirectory", error))?;
        }
        let mut out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&out_path)
            .map_err(|error| io_error("create staged member", error))?;

        let mut hasher = Sha256Digest::new();
        let mut written: u64 = 0;
        let mut buffer = vec![0u8; EXTRACT_BUFFER];
        loop {
            self.check_cancelled()?;
            let read = file
                .read(&mut buffer)
                .map_err(|error| archive_corrupt(&error.to_string()))?;
            if read == 0 {
                break;
            }
            written += read as u64;
            if written > self.limits.max_member_uncompressed {
                return Err(Error::LimitExceeded {
                    kind: LimitKind::MemberSize,
                    detail: format!(
                        "member '{name}' exceeds {} bytes",
                        self.limits.max_member_uncompressed
                    ),
                });
            }
            if counters.total + written > self.limits.max_total_uncompressed {
                return Err(Error::LimitExceeded {
                    kind: LimitKind::TotalSize,
                    detail: format!(
                        "extraction exceeds {} uncompressed bytes",
                        self.limits.max_total_uncompressed
                    ),
                });
            }
            hasher.update(&buffer[..read]);
            out.write_all(&buffer[..read])
                .map_err(|error| io_error("write staged member", error))?;
        }
        out.flush()
            .map_err(|error| io_error("write staged member", error))?;
        drop(out);

        let declared = file.size();
        if written != declared {
            return Err(archive_corrupt(&format!(
                "member '{name}' declared {declared} bytes but produced {written}"
            )));
        }

        // Never set an executable bit on extracted content (spec 006, R5).
        restrict_permissions(&out_path)?;

        counters.total += written;
        Ok((relative_path, written, Sha256::finish(hasher)))
    }

    /// Reads a nested archive's bytes into memory, bounded by the limits.
    fn read_nested<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        name: &str,
        counters: &mut Counters,
    ) -> Result<Vec<u8>, Error> {
        let mut file = self.open_member(archive, name)?;
        let declared = file.size();
        let mut bytes = Vec::new();
        let mut written: u64 = 0;
        let mut buffer = vec![0u8; EXTRACT_BUFFER];
        loop {
            self.check_cancelled()?;
            let read = file
                .read(&mut buffer)
                .map_err(|error| archive_corrupt(&error.to_string()))?;
            if read == 0 {
                break;
            }
            written += read as u64;
            if written > self.limits.max_member_uncompressed {
                return Err(Error::LimitExceeded {
                    kind: LimitKind::MemberSize,
                    detail: format!(
                        "nested archive '{name}' exceeds {} bytes",
                        self.limits.max_member_uncompressed
                    ),
                });
            }
            if counters.total + written > self.limits.max_total_uncompressed {
                return Err(Error::LimitExceeded {
                    kind: LimitKind::TotalSize,
                    detail: format!(
                        "extraction exceeds {} uncompressed bytes",
                        self.limits.max_total_uncompressed
                    ),
                });
            }
            bytes.extend_from_slice(&buffer[..read]);
        }
        if written != declared {
            return Err(archive_corrupt(&format!(
                "nested archive '{name}' declared {declared} bytes but produced {written}"
            )));
        }
        counters.total += written;
        Ok(bytes)
    }

    /// Validates an entry and returns it, applying the per-member checks.
    fn open_member<'a, R: Read + Seek>(
        &self,
        archive: &'a mut ZipArchive<R>,
        name: &str,
    ) -> Result<zip::read::ZipFile<'a, R>, Error> {
        validate_member_name(name)?;

        // Check existence with a read-only lookup first: it releases the
        // borrow before `by_name`, and lets a miss scan for the case hint.
        if archive.index_for_name(name).is_none() {
            return Err(Error::MemberMissing {
                requested: name.to_owned(),
                closest: closest_match(archive, name),
            });
        }
        let file = match archive.by_name(name) {
            Ok(file) => file,
            Err(error) => return Err(archive_corrupt(&error.to_string())),
        };

        if file.is_symlink() {
            return Err(Error::UnsafeMember {
                name: name.to_owned(),
                reason: "symbolic link entry".to_owned(),
            });
        }
        if file.is_dir() {
            return Err(Error::MemberNotAFile {
                name: name.to_owned(),
            });
        }
        if file.enclosed_name().is_none() {
            return Err(Error::UnsafeMember {
                name: name.to_owned(),
                reason: "path is not enclosed".to_owned(),
            });
        }
        match file.compression() {
            CompressionMethod::Stored | CompressionMethod::Deflated => {}
            other => {
                return Err(Error::UnsupportedCompression {
                    member: name.to_owned(),
                    method: format!("{other:?}"),
                });
            }
        }

        let size = file.size();
        if size > self.limits.max_member_uncompressed {
            return Err(Error::LimitExceeded {
                kind: LimitKind::MemberSize,
                detail: format!(
                    "member '{name}' is {size} bytes, limit {}",
                    self.limits.max_member_uncompressed
                ),
            });
        }
        let compressed = file.compressed_size();
        if compressed > 0
            && u128::from(size)
                > u128::from(compressed) * u128::from(self.limits.max_compression_ratio)
        {
            return Err(Error::LimitExceeded {
                kind: LimitKind::CompressionRatio,
                detail: format!(
                    "member '{name}' expands {size} bytes from {compressed} (limit {}x)",
                    self.limits.max_compression_ratio
                ),
            });
        }
        Ok(file)
    }

    /// Rejects an archive with too many entries (zip-bomb guard).
    fn check_archive_limits<R: Read + Seek>(&self, archive: &ZipArchive<R>) -> Result<(), Error> {
        if archive.len() > self.limits.max_member_count {
            return Err(Error::LimitExceeded {
                kind: LimitKind::MemberCount,
                detail: format!(
                    "archive has {} entries, limit {}",
                    archive.len(),
                    self.limits.max_member_count
                ),
            });
        }
        Ok(())
    }
}

/// Uncompressed bytes consumed so far, shared across nesting levels.
struct Counters {
    total: u64,
}

/// Splits a `!/`-separated member path and validates every segment.
fn parse_member_path(path: &str) -> Result<Vec<String>, Error> {
    if path.is_empty() {
        return Err(Error::UnsafeMember {
            name: String::new(),
            reason: "empty member path".to_owned(),
        });
    }
    let segments: Vec<String> = path.split("!/").map(str::to_owned).collect();
    for segment in &segments {
        validate_member_name(segment)?;
    }
    Ok(segments)
}

/// Rejects a member name that is absolute, traverses, or hides a separator.
fn validate_member_name(name: &str) -> Result<(), Error> {
    let unsafe_member = |reason: &str| {
        Err(Error::UnsafeMember {
            name: name.to_owned(),
            reason: reason.to_owned(),
        })
    };
    if name.is_empty() {
        return unsafe_member("empty member name");
    }
    if name.contains('\0') {
        return unsafe_member("contains a NUL byte");
    }
    if name.starts_with('/') {
        return unsafe_member("absolute path");
    }
    if name.contains('\\') {
        return unsafe_member("backslash separator");
    }
    if name.contains(':') {
        return unsafe_member("drive prefix");
    }
    if name.split('/').any(|component| component == "..") {
        return unsafe_member("parent-directory component");
    }
    Ok(())
}

/// The nearest case-insensitive entry name, for a helpful miss message.
fn closest_match<R: Read + Seek>(archive: &ZipArchive<R>, name: &str) -> Option<String> {
    for index in 0..archive.len() {
        if let Some(Ok(candidate)) = archive.name_for_index(index)
            && candidate.eq_ignore_ascii_case(name)
        {
            return Some(candidate.into_owned());
        }
    }
    None
}

/// Creates a fresh, unique staging directory under `root`.
fn create_staging_dir(root: &Path) -> Result<PathBuf, Error> {
    fs::create_dir_all(root).map_err(|error| io_error("create staging root", error))?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    for _ in 0..100 {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!("extract-{}-{unique}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_error("create staging directory", error)),
        }
    }
    Err(Error::Io {
        context: "create staging directory".to_owned(),
        detail: "could not find a unique name".to_owned(),
    })
}

/// Strips every permission bit that could make an extracted file executable.
#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644))
        .map_err(|error| io_error("set member permissions", error))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), Error> {
    Ok(())
}

fn archive_corrupt(detail: &str) -> Error {
    Error::ArchiveCorrupt {
        detail: detail.to_owned(),
    }
}

fn io_error(context: &str, error: std::io::Error) -> Error {
    Error::Io {
        context: context.to_owned(),
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;
    use crate::downloads::test_http::TestDir;

    /// One entry to place in a generated archive.
    struct Entry {
        name: String,
        data: Vec<u8>,
        method: CompressionMethod,
    }

    impl Entry {
        fn stored(name: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
            Self {
                name: name.into(),
                data: data.into(),
                method: CompressionMethod::Stored,
            }
        }

        fn deflated(name: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
            Self {
                name: name.into(),
                data: data.into(),
                method: CompressionMethod::Deflated,
            }
        }
    }

    fn zip_bytes(entries: &[Entry]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for entry in entries {
            let options = SimpleFileOptions::default().compression_method(entry.method);
            writer.start_file(&entry.name, options).expect("start file");
            writer.write_all(&entry.data).expect("write entry");
        }
        writer.finish().expect("finish zip").into_inner()
    }

    fn write_zip(dir: &Path, file_name: &str, entries: &[Entry]) -> PathBuf {
        let path = dir.join(file_name);
        fs::write(&path, zip_bytes(entries)).expect("write zip");
        path
    }

    #[test]
    fn extracts_a_flat_member_with_hash_and_size_checks() {
        let dir = TestDir::new("flat");
        let data = b"flat wad contents".to_vec();
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::deflated("flat.wad", data.clone())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));

        let member = MemberRequest::new("flat.wad")
            .with_size(data.len() as u64)
            .with_sha256(Sha256::of(&data));
        let extraction = extractor.extract(&package, &[member]).expect("extract");
        assert_eq!(extraction.members().len(), 1);
        let extracted = &extraction.members()[0];
        assert_eq!(extracted.relative_path, PathBuf::from("flat.wad"));
        assert_eq!(extracted.size, data.len() as u64);
        assert_eq!(
            fs::read(extraction.staging_dir().join("flat.wad")).unwrap(),
            data
        );
        // Content is not executable.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(extraction.staging_dir().join("flat.wad"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o111, 0, "no executable bits");
        }
    }

    #[test]
    fn extracts_a_member_from_a_nested_archive() {
        let dir = TestDir::new("nested");
        let inner_data = b"addon wad".to_vec();
        let inner = zip_bytes(&[Entry::stored(
            "DBP37_AlternateBlaster.wad",
            inner_data.clone(),
        )]);
        let outer = zip_bytes(&[
            Entry::deflated("DBP37_AUGZEN.wad", b"main wad".to_vec()),
            Entry::stored("DBP37_Addons.zip", inner),
        ]);
        let package = dir.path().join("pkg.zip");
        fs::write(&package, outer).unwrap();
        let extractor = Extractor::new(dir.path().join("staging"));

        let member = MemberRequest::new("DBP37_Addons.zip!/DBP37_AlternateBlaster.wad")
            .with_sha256(Sha256::of(&inner_data));
        let extraction = extractor.extract(&package, &[member]).expect("extract");
        let extracted = &extraction.members()[0];
        assert_eq!(
            extracted.relative_path,
            PathBuf::from("DBP37_AlternateBlaster.wad")
        );
        assert_eq!(
            fs::read(extraction.staging_dir().join(&extracted.relative_path)).unwrap(),
            inner_data
        );
    }

    #[test]
    fn a_missing_member_reports_the_case_insensitive_match() {
        let dir = TestDir::new("case-hint");
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::stored("wads/Main.WAD", b"x".to_vec())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));
        let error = extractor
            .extract(&package, &[MemberRequest::new("wads/main.wad")])
            .expect_err("missing member");
        match error {
            Error::MemberMissing { requested, closest } => {
                assert_eq!(requested, "wads/main.wad");
                assert_eq!(closest.as_deref(), Some("wads/Main.WAD"));
            }
            other => panic!("expected MemberMissing, got {other}"),
        }
    }

    #[test]
    fn rejects_a_wrong_member_hash() {
        let dir = TestDir::new("member-hash");
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::stored("a.wad", b"abc".to_vec())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));
        let member = MemberRequest::new("a.wad").with_sha256(Sha256::of(b"different"));
        let error = extractor.extract(&package, &[member]).expect_err("hash");
        assert!(matches!(error, Error::HashMismatch { .. }), "{error}");
    }

    #[test]
    fn rejects_a_wrong_member_size() {
        let dir = TestDir::new("member-size");
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::stored("a.wad", b"abcd".to_vec())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));
        let member = MemberRequest::new("a.wad").with_size(99);
        let error = extractor.extract(&package, &[member]).expect_err("size");
        assert!(
            matches!(error, Error::SizeExceeded { limit: 99, got: 4 }),
            "{error}"
        );
    }

    #[test]
    fn rejects_traversal_absolute_backslash_and_drive_names() {
        let dir = TestDir::new("unsafe");
        let extractor = Extractor::new(dir.path().join("staging"));

        for (label, name) in [
            ("traversal", "../evil.wad"),
            ("absolute", "/etc/evil.wad"),
            ("backslash", "..\\evil.wad"),
            ("drive", "C:evil.wad"),
        ] {
            let package = write_zip(
                dir.path(),
                &format!("{label}.zip"),
                &[Entry::stored(name, b"evil".to_vec())],
            );
            let error = extractor
                .extract(&package, &[MemberRequest::new(name)])
                .unwrap_err();
            assert!(
                matches!(error, Error::UnsafeMember { .. }),
                "{label}: expected UnsafeMember, got {error}"
            );
        }

        // Nothing was written outside the staging root.
        assert!(!dir.path().join("evil.wad").exists());
        assert!(!dir.path().join("etc").exists());
    }

    #[test]
    fn rejects_a_symlink_entry() {
        let dir = TestDir::new("symlink");
        // `ZipWriter` drops the file-type bits from `unix_permissions`, so make
        // the entry a symlink by patching the central directory directly.
        let mut bytes = zip_bytes(&[Entry::stored("link.wad", "/etc/passwd")]);
        patch_symlink(&mut bytes);
        let package = dir.path().join("pkg.zip");
        fs::write(&package, &bytes).unwrap();
        let extractor = Extractor::new(dir.path().join("staging"));
        let error = extractor
            .extract(&package, &[MemberRequest::new("link.wad")])
            .expect_err("symlink");
        assert!(matches!(error, Error::UnsafeMember { .. }), "{error}");
    }

    #[test]
    fn rejects_a_zip_bomb_ratio() {
        let dir = TestDir::new("bomb");
        // 4 MiB of zeros compresses to a few KiB: a large ratio.
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::deflated("bomb.wad", vec![0u8; 4 * 1024 * 1024])],
        );
        let limits = ExtractLimits {
            max_total_uncompressed: 16 * 1024 * 1024,
            max_member_uncompressed: 16 * 1024 * 1024,
            max_compression_ratio: 10,
            ..ExtractLimits::default()
        };
        let extractor = Extractor::new(dir.path().join("staging")).with_limits(limits);
        let error = extractor
            .extract(&package, &[MemberRequest::new("bomb.wad")])
            .expect_err("ratio");
        assert!(
            matches!(
                error,
                Error::LimitExceeded {
                    kind: LimitKind::CompressionRatio,
                    ..
                }
            ),
            "{error}"
        );
    }

    #[test]
    fn rejects_too_many_members() {
        let dir = TestDir::new("count");
        let named: Vec<Entry> = (0..5)
            .map(|i| Entry::stored(format!("file{i}.wad"), vec![i as u8]))
            .collect();
        let package = write_zip(dir.path(), "pkg.zip", &named);
        let limits = ExtractLimits {
            max_member_count: 3,
            ..ExtractLimits::default()
        };
        let extractor = Extractor::new(dir.path().join("staging")).with_limits(limits);
        let error = extractor
            .extract(&package, &[MemberRequest::new("file0.wad")])
            .expect_err("count");
        assert!(
            matches!(
                error,
                Error::LimitExceeded {
                    kind: LimitKind::MemberCount,
                    ..
                }
            ),
            "{error}"
        );
    }

    #[test]
    fn rejects_nesting_deeper_than_the_limit() {
        let dir = TestDir::new("depth");
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::stored("a.wad", b"x".to_vec())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));
        let error = extractor
            .extract(
                &package,
                &[MemberRequest::new("a.zip!/b.zip!/c.zip!/d.wad")],
            )
            .expect_err("depth");
        assert!(
            matches!(
                error,
                Error::LimitExceeded {
                    kind: LimitKind::NestingDepth,
                    ..
                }
            ),
            "{error}"
        );
    }

    #[test]
    fn rejects_an_unsupported_compression_method() {
        let dir = TestDir::new("method");
        let mut bytes = zip_bytes(&[Entry::stored("a.wad", b"x".to_vec())]);
        patch_compression_method(&mut bytes, 12); // 12 = bzip2, not enabled
        let package = dir.path().join("pkg.zip");
        fs::write(&package, &bytes).unwrap();
        let extractor = Extractor::new(dir.path().join("staging"));
        let error = extractor
            .extract(&package, &[MemberRequest::new("a.wad")])
            .unwrap_err();
        // Either the method is reported directly or the archive is rejected as
        // corrupt; both are typed and explainable.
        assert!(
            matches!(
                error,
                Error::UnsupportedCompression { .. } | Error::ArchiveCorrupt { .. }
            ),
            "{error}"
        );
    }

    #[test]
    fn commit_moves_members_into_place_and_removes_staging() {
        let dir = TestDir::new("commit");
        let data = b"committed data".to_vec();
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::stored("sub/a.wad", data.clone())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));
        let dest = dir.path().join("install");
        let installed = extractor
            .extract_into(&package, &[MemberRequest::new("sub/a.wad")], &dest)
            .expect("commit");
        assert_eq!(installed, vec![dest.join("sub/a.wad")]);
        assert_eq!(fs::read(&installed[0]).unwrap(), data);
        let entries: Vec<_> = fs::read_dir(dir.path().join("staging")).unwrap().collect();
        assert!(entries.is_empty(), "staging not cleaned: {entries:?}");
    }

    #[test]
    fn dropping_an_uncommitted_extraction_removes_staging() {
        let dir = TestDir::new("drop");
        let package = write_zip(
            dir.path(),
            "pkg.zip",
            &[Entry::stored("a.wad", b"x".to_vec())],
        );
        let extractor = Extractor::new(dir.path().join("staging"));
        let extraction = extractor
            .extract(&package, &[MemberRequest::new("a.wad")])
            .unwrap();
        let staging = extraction.staging_dir().to_path_buf();
        assert!(staging.exists());
        drop(extraction);
        assert!(!staging.exists(), "staging should be removed on drop");
    }

    #[test]
    fn rejects_conflicting_output_paths() {
        let dir = TestDir::new("conflict");
        let inner = zip_bytes(&[Entry::stored("a.wad", b"inner".to_vec())]);
        let outer = zip_bytes(&[
            Entry::stored("a.wad", b"outer".to_vec()),
            Entry::stored("inner.zip", inner),
        ]);
        let package = dir.path().join("pkg.zip");
        fs::write(&package, outer).unwrap();
        let extractor = Extractor::new(dir.path().join("staging"));
        let error = extractor
            .extract(
                &package,
                &[
                    MemberRequest::new("a.wad"),
                    MemberRequest::new("inner.zip!/a.wad"),
                ],
            )
            .expect_err("conflict");
        assert!(matches!(error, Error::MemberConflict { .. }), "{error}");
    }

    /// Rewrites the compression-method field of every ZIP header.
    fn patch_compression_method(bytes: &mut [u8], method: u16) {
        let local = [0x50u8, 0x4b, 0x03, 0x04];
        let central = [0x50u8, 0x4b, 0x01, 0x02];
        for index in 0..bytes.len().saturating_sub(4) {
            if bytes[index..index + 4] == local {
                bytes[index + 8..index + 10].copy_from_slice(&method.to_le_bytes());
            } else if bytes[index..index + 4] == central {
                bytes[index + 10..index + 12].copy_from_slice(&method.to_le_bytes());
            }
        }
    }

    /// Marks every central-directory entry as a Unix symlink (`S_IFLNK`).
    fn patch_symlink(bytes: &mut [u8]) {
        let central = [0x50u8, 0x4b, 0x01, 0x02];
        for index in 0..bytes.len().saturating_sub(46) {
            if bytes[index..index + 4] == central {
                // `version made by`: byte 5 is the system (3 = Unix).
                bytes[index + 5] = 3;
                // External attributes: high 16 bits are the Unix mode.
                let attributes = 0o120_777u32 << 16;
                bytes[index + 38..index + 42].copy_from_slice(&attributes.to_le_bytes());
            }
        }
    }
}

#[cfg(test)]
mod cancellation_tests {
    #[test]
    fn cancelled_extraction_does_not_read_or_create_staging() {
        let dir = tempfile::tempdir().unwrap();
        let cancel = crate::downloads::CancellationToken::new();
        cancel.cancel();
        let staging = dir.path().join("staging");
        let extractor = super::Extractor::new(&staging).with_cancellation(cancel);
        assert!(matches!(
            extractor.extract(&dir.path().join("absent.zip"), &[]),
            Err(super::Error::Cancelled)
        ));
        assert!(!staging.exists());
    }
}
