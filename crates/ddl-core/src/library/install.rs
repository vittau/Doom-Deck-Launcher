//! Installing a mapset from a catalog recipe (spec 009, R1–R3).
//!
//! [`Installer::install`] downloads each package the selected components need
//! through spec 006's [`Fetcher`] (HTTPS-only, allowlisted hosts, mirrors,
//! bounded and SHA-256 verified, cached by hash), extracts **exactly** the
//! members those components name plus every `keep = true` member, and places
//! them under `<data>/library/<recipe id>/<release>/`. User-supplied components
//! are never downloaded (D-014); [`Installer::import_user_component`] copies the
//! player's own file into the install instead.
//!
//! Cached packages are keyed by SHA-256, so installing one more component later
//! re-uses the archive already on disk and never re-downloads it (spec 006, R3).

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256 as Sha256Digest};

use crate::catalog::{Member, Package, PackageKind, Recipe};
use crate::downloads::{
    CancellationToken, DownloadRequest, Extractor, Fetcher, HostAllowlist, MemberRequest, Sha256,
};

use super::manifest::{
    InstallManifest, InstalledMember, InstalledPackage, MANIFEST_FILE, ManifestError,
    SCHEMA_VERSION,
};

/// Directory under DDL's data dir that holds installed mapsets.
pub const LIBRARY_DIR: &str = "library";
/// Subdirectory of an install holding user-imported files.
pub const IMPORT_DIR: &str = "imports";

/// Hard upper bound on a single installed member (matches the catalog lint cap).
pub const MAX_MEMBER_BYTES: u64 = 256 * 1024 * 1024;

/// A source of wall-clock time, injectable so `installed_at` is deterministic
/// in tests.
pub trait Clock {
    /// Seconds since the Unix epoch.
    fn now_unix_secs(&self) -> u64;
}

/// The real clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}

/// What [`Installer::import_user_component`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOutcome {
    /// Whether the file matched one of the component's known SHA-256 values.
    /// An unrecognised file is still accepted (D-014).
    pub matched: bool,
    /// SHA-256 of the imported file.
    pub sha256: String,
    /// Size of the imported file in bytes.
    pub size: u64,
    /// Where the file was placed inside the install.
    pub path: PathBuf,
}

/// Installs and updates mapset content.
pub struct Installer {
    data_dir: PathBuf,
    cache_dir: PathBuf,
    hosts: HostAllowlist,
    fetcher: Fetcher,
    extractor: Extractor,
    clock: Box<dyn Clock>,
}

impl Installer {
    /// Creates an installer rooted at `data_dir`, caching downloads under
    /// `cache_dir` and only contacting hosts in `hosts`.
    #[must_use]
    pub fn new(
        data_dir: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
        hosts: HostAllowlist,
    ) -> Self {
        let data_dir = data_dir.into();
        let cache_dir = cache_dir.into();
        Self {
            fetcher: Fetcher::new(&cache_dir),
            extractor: Extractor::new(data_dir.join("staging")),
            data_dir,
            cache_dir,
            hosts,
            clock: Box::new(SystemClock),
        }
    }

    /// Replaces the clock (tests).
    #[must_use]
    pub fn with_clock(mut self, clock: Box<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Replaces the fetcher (tests reach a loopback server through the spec 006
    /// test seam).
    #[cfg(any(test, feature = "test-support"))]
    pub fn with_fetcher(mut self, fetcher: Fetcher) -> Self {
        self.fetcher = fetcher;
        self
    }

    /// DDL's data dir this installer writes under.
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// The download cache dir this installer uses.
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// `<data>/library/<recipe id>/<release>`, the install directory.
    ///
    /// # Errors
    ///
    /// Returns [`InstallError::UnsafePath`] when the recipe id or release is not
    /// a safe single path component.
    pub fn install_dir(&self, recipe: &Recipe) -> Result<PathBuf, InstallError> {
        let id = safe_component(&recipe.id).ok_or_else(|| InstallError::UnsafePath {
            detail: format!("recipe id `{}` is not a safe path component", recipe.id),
        })?;
        let release = safe_component(&recipe.release).ok_or_else(|| InstallError::UnsafePath {
            detail: format!(
                "recipe release `{}` is not a safe path component",
                recipe.release
            ),
        })?;
        Ok(self.data_dir.join(LIBRARY_DIR).join(id).join(release))
    }

    /// The install record path for `recipe`.
    ///
    /// # Errors
    ///
    /// As [`Installer::install_dir`].
    pub fn manifest_path(&self, recipe: &Recipe) -> Result<PathBuf, InstallError> {
        Ok(self.install_dir(recipe)?.join(MANIFEST_FILE))
    }

    /// Loads the install record for `recipe`, if it is installed.
    ///
    /// # Errors
    ///
    /// Returns [`InstallError`] when the record cannot be read.
    pub fn load_manifest(&self, recipe: &Recipe) -> Result<Option<InstallManifest>, InstallError> {
        Ok(InstallManifest::load_optional(
            &self.manifest_path(recipe)?,
        )?)
    }

    /// Installs `recipe` for the selected components.
    ///
    /// `enabled` names the optional component ids to enable; `main` is implicit,
    /// exactly as [`Recipe::load_order`] expects. Re-installing with more
    /// components merges into the existing install and re-uses cached packages.
    ///
    /// # Errors
    ///
    /// Returns [`InstallError`] on a download, extraction, integrity or I/O
    /// failure, when the recipe's content identity changed since it was
    /// installed, or when the selection is invalid.
    pub fn install(
        &self,
        recipe: &Recipe,
        enabled: &[&str],
    ) -> Result<InstallManifest, InstallError> {
        self.install_with_progress(recipe, enabled, &CancellationToken::new(), &mut |_| {})
    }

    /// Cancellable install; cancellation is deferred during atomic commit.
    pub fn install_with_progress(
        &self,
        recipe: &Recipe,
        enabled: &[&str],
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(crate::downloads::InstallProgress),
    ) -> Result<InstallManifest, InstallError> {
        use crate::downloads::{InstallProgress, InstallStage};
        if cancel.is_cancelled() {
            return Err(InstallError::Download(crate::downloads::Error::Cancelled));
        }
        let install_dir = self.install_dir(recipe)?;
        let manifest_path = install_dir.join(MANIFEST_FILE);
        let existing = InstallManifest::load_optional(&manifest_path)?;
        let recipe_sha256 = recipe_content_sha256(recipe)?;
        if let Some(held) = &existing
            && held.recipe_sha256 != recipe_sha256
        {
            return Err(InstallError::RecipeChanged {
                id: recipe.id.clone(),
                installed: held.recipe_sha256.clone(),
                current: recipe_sha256,
            });
        }

        let active = recipe
            .active_components(enabled)
            .map_err(|detail| InstallError::InvalidRecipe { detail })?;

        // The packages and members this selection needs. A keep member always
        // travels with its package so permission text stays intact (spec 002).
        let mut needed_packages: Vec<&Package> = Vec::new();
        let mut desired: Vec<(&Package, &Member)> = Vec::new();
        for component in &active {
            if component.user_supplied {
                continue; // never downloaded (D-014)
            }
            for reference in &component.members {
                let (Some(package_id), Some(member_id)) =
                    (reference.package.as_deref(), reference.member.as_deref())
                else {
                    return Err(InstallError::InvalidRecipe {
                        detail: format!(
                            "component `{}` has a member without a package reference",
                            component.id
                        ),
                    });
                };
                let package = recipe
                    .packages
                    .iter()
                    .find(|package| package.id == package_id)
                    .ok_or_else(|| InstallError::InvalidRecipe {
                        detail: format!("unknown package `{package_id}`"),
                    })?;
                let member = package
                    .members
                    .iter()
                    .find(|member| member.id == member_id)
                    .ok_or_else(|| InstallError::InvalidRecipe {
                        detail: format!("unknown member `{member_id}` in package `{package_id}`"),
                    })?;
                if !needed_packages.iter().any(|held| held.id == package.id) {
                    needed_packages.push(package);
                }
                if !desired
                    .iter()
                    .any(|(held, existing)| held.id == package.id && existing.id == member.id)
                {
                    desired.push((package, member));
                }
            }
        }
        for package in &needed_packages {
            for member in package.members.iter().filter(|member| member.keep) {
                if !desired
                    .iter()
                    .any(|(held, existing)| held.id == package.id && existing.id == member.id)
                {
                    desired.push((package, member));
                }
            }
        }

        // A member already installed with the same identity is reused as-is.
        let mut members: Vec<InstalledMember> = Vec::new();
        let mut missing: Vec<(&Package, &Member)> = Vec::new();
        for (package, member) in &desired {
            let path = member_local_path(&install_dir, &package.id, &member.path);
            let reused = existing
                .as_ref()
                .and_then(|held| held.find_downloaded(&package.id, &member.id))
                .filter(|stored| {
                    stored.sha256.eq_ignore_ascii_case(&member.sha256)
                        && stored.path == path
                        && path.is_file()
                });
            match reused {
                Some(stored) if record_matches(stored, cancel)? => members.push(stored.clone()),
                Some(stored) => {
                    tracing::warn!(path=%stored.path.display(),"replacing an invalid installed member");
                    missing.push((package, member));
                }
                None => missing.push((package, member)),
            }
        }

        // A brand-new install is built in a staging directory and moved into
        // place with one rename; an update extracts only what is missing.
        let fresh = existing.is_none();
        let base = if fresh {
            create_staging_dir(&self.data_dir.join(LIBRARY_DIR))?
        } else {
            install_dir.clone()
        };

        for package in &needed_packages {
            let package_missing: Vec<&Member> = missing
                .iter()
                .filter(|(held, _)| held.id == package.id)
                .map(|(_, member)| *member)
                .collect();
            if package_missing.is_empty() {
                continue;
            }
            if cancel.is_cancelled() {
                return Err(InstallError::Download(crate::downloads::Error::Cancelled));
            }
            progress(InstallProgress::transfer(crate::downloads::Progress {
                downloaded: 0,
                total: Some(package.size),
            }));
            let cached = self.download_package(package, cancel, progress)?;
            progress(InstallProgress::stage(InstallStage::Extract));
            let dest = base.join(&package.id);
            match package.kind {
                PackageKind::Zip => {
                    let requests = package_missing
                        .iter()
                        .map(|member| member_request(member))
                        .collect::<Result<Vec<_>, _>>()?;
                    Extractor::new(self.data_dir.join("staging"))
                        .with_limits(self.extractor.limits())
                        .with_cancellation(cancel.clone())
                        .extract_into(&cached, &requests, &dest)
                        .map_err(InstallError::Download)?;
                }
                PackageKind::Wad => copy_bare_wad(&cached, &dest, package_missing[0])?,
            }
        }

        if cancel.is_cancelled() {
            return Err(InstallError::Download(crate::downloads::Error::Cancelled));
        }
        progress(InstallProgress::stage(InstallStage::Verify));
        for (package, member) in &missing {
            members.push(InstalledMember {
                package: Some(package.id.clone()),
                member: Some(member.id.clone()),
                component: None,
                path: member_local_path(&install_dir, &package.id, &member.path),
                size: member.size,
                sha256: member.sha256.clone(),
                keep: member.keep,
            });
        }

        // The manifest records everything still installed, not just this launch selection.
        // Changing to a replacement must not forget a previously installed main or add-on.
        if let Some(held) = &existing {
            for entry in &held.members {
                if members.iter().any(|m| {
                    m.package == entry.package
                        && m.member == entry.member
                        && m.component == entry.component
                }) {
                    continue;
                }
                if entry.path.starts_with(&install_dir) && record_matches(entry, cancel)? {
                    members.push(entry.clone());
                } else {
                    tracing::warn!(path=%entry.path.display(),"dropping an invalid prior install record");
                }
            }
        }
        let mut installed_packages: Vec<InstalledPackage> = needed_packages
            .iter()
            .map(|p| InstalledPackage {
                id: p.id.clone(),
                sha256: p.sha256.clone(),
                size: p.size,
            })
            .collect();
        if let Some(held) = &existing {
            for package in &held.packages {
                if !installed_packages.iter().any(|p| p.id == package.id)
                    && members
                        .iter()
                        .any(|m| m.package.as_deref() == Some(&package.id))
                {
                    installed_packages.push(package.clone());
                }
            }
        }

        let manifest = InstallManifest {
            schema_version: SCHEMA_VERSION,
            recipe_id: recipe.id.clone(),
            release: recipe.release.clone(),
            recipe_sha256,
            installed_at: existing
                .as_ref()
                .map_or_else(|| self.clock.now_unix_secs(), |held| held.installed_at),
            packages: installed_packages,
            members,
        };

        if cancel.is_cancelled() {
            return Err(InstallError::Download(crate::downloads::Error::Cancelled));
        }
        progress(InstallProgress::stage(InstallStage::Commit));
        if fresh {
            manifest.save(&base.join(MANIFEST_FILE))?;
            if let Some(parent) = install_dir.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| io_error("create library directory", error))?;
            }
            if install_dir.exists() {
                fs::remove_dir_all(&install_dir)
                    .map_err(|error| io_error("replace a stale install", error))?;
            }
            fs::rename(&base, &install_dir)
                .map_err(|error| io_error("place the install atomically", error))?;
        } else {
            manifest.save(&manifest_path)?;
        }

        Ok(manifest)
    }

    /// Copies a player-supplied file into the install for a `user_supplied`
    /// component (spec 009, R3; D-014).
    ///
    /// The file's SHA-256 is matched against the component's known hashes when
    /// the recipe lists any; an unknown hash is never rejected, only reported.
    ///
    /// # Errors
    ///
    /// Returns [`InstallError`] when the component is unknown or not
    /// user-supplied, the mapset is not installed, the file is empty or too
    /// large, or it cannot be copied.
    pub fn import_user_component(
        &self,
        recipe: &Recipe,
        component_id: &str,
        source: &Path,
    ) -> Result<ImportOutcome, InstallError> {
        self.import_user_component_with_cancel(
            recipe,
            component_id,
            source,
            &CancellationToken::new(),
        )
    }

    /// Imports cooperatively, leaving no manifest update when cancelled before commit.
    pub fn import_user_component_with_cancel(
        &self,
        recipe: &Recipe,
        component_id: &str,
        source: &Path,
        cancel: &CancellationToken,
    ) -> Result<ImportOutcome, InstallError> {
        check_cancel(cancel)?;
        let component = recipe
            .components
            .iter()
            .find(|component| component.id == component_id)
            .ok_or_else(|| InstallError::UnknownComponent {
                component: component_id.to_owned(),
            })?;
        if !component.user_supplied {
            return Err(InstallError::NotUserSupplied {
                component: component_id.to_owned(),
            });
        }

        let install_dir = self.install_dir(recipe)?;
        let manifest_path = install_dir.join(MANIFEST_FILE);
        let mut manifest =
            InstallManifest::load_optional(&manifest_path)?.ok_or(InstallError::NotInstalled)?;

        let metadata =
            fs::metadata(source).map_err(|error| io_error("read imported file", error))?;
        if !metadata.is_file() {
            return Err(InstallError::UnsafePath {
                detail: format!("`{}` is not a regular file", source.display()),
            });
        }
        let size = metadata.len();
        if size == 0 {
            return Err(InstallError::EmptyFile {
                path: source.to_path_buf(),
            });
        }
        if size > MAX_MEMBER_BYTES {
            return Err(InstallError::ImportTooLarge {
                limit: MAX_MEMBER_BYTES,
                got: size,
            });
        }
        let sha256 = hash_file(source, cancel)?;
        let matched = component
            .known_sha256
            .iter()
            .any(|known| known.eq_ignore_ascii_case(&sha256));

        let name = source
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(safe_component)
            .ok_or_else(|| InstallError::UnsafePath {
                detail: format!("`{}` has no safe file name", source.display()),
            })?;
        let directory = install_dir
            .join(IMPORT_DIR)
            .join(safe_component(component_id).unwrap_or("component"));
        fs::create_dir_all(&directory)
            .map_err(|error| io_error("create the import directory", error))?;
        let destination = directory.join(name);
        let temporary = destination.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
        let copy = || -> Result<(), InstallError> {
            let mut input =
                fs::File::open(source).map_err(|e| io_error("read imported file", e))?;
            let mut output =
                fs::File::create(&temporary).map_err(|e| io_error("stage imported file", e))?;
            let mut bytes = [0u8; 64 * 1024];
            loop {
                check_cancel(cancel)?;
                let count = input
                    .read(&mut bytes)
                    .map_err(|e| io_error("read imported file", e))?;
                if count == 0 {
                    break;
                }
                output
                    .write_all(&bytes[..count])
                    .map_err(|e| io_error("write imported file", e))?;
            }
            output
                .sync_all()
                .map_err(|e| io_error("flush imported file", e))?;
            check_cancel(cancel)?;
            Ok(())
        };
        if let Err(error) = copy() {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        restrict_permissions(&temporary)?;
        fs::rename(&temporary, &destination)
            .map_err(|error| io_error("place the imported file", error))?;

        manifest.members.retain(|held| {
            !(held.package.is_none() && held.component.as_deref() == Some(component_id))
        });
        manifest.members.push(InstalledMember {
            package: None,
            member: None,
            component: Some(component_id.to_owned()),
            path: destination.clone(),
            size,
            sha256: sha256.clone(),
            keep: false,
        });
        manifest.save(&manifest_path)?;

        Ok(ImportOutcome {
            matched,
            sha256,
            size,
            path: destination,
        })
    }

    fn download_package(
        &self,
        package: &Package,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(crate::downloads::InstallProgress),
    ) -> Result<PathBuf, InstallError> {
        let Some(first) = package.urls.first() else {
            return Err(InstallError::InvalidRecipe {
                detail: format!("package `{}` has no download URLs", package.id),
            });
        };
        let expected =
            Sha256::parse_hex(&package.sha256).ok_or_else(|| InstallError::InvalidRecipe {
                detail: format!("package `{}` has a malformed SHA-256", package.id),
            })?;
        let request = DownloadRequest::new(
            first.clone(),
            expected,
            MAX_MEMBER_BYTES,
            self.hosts.clone(),
        )
        .with_urls(package.urls.iter().cloned())
        .with_size(package.size);
        self.fetcher
            .download(&request, cancel, &mut |p| {
                progress(crate::downloads::InstallProgress::transfer(p))
            })
            .map_err(InstallError::Download)
    }
}

/// The relative path an extracted member occupies, mirroring spec 006's
/// `Extraction::commit`: the tail after the last nested-archive separator.
fn member_relative_path(path: &str) -> &str {
    path.rsplit_once("!/").map_or(path, |(_, tail)| tail)
}

/// The absolute path a member ends up at inside `install_dir`.
fn member_local_path(install_dir: &Path, package_id: &str, member_path: &str) -> PathBuf {
    install_dir
        .join(package_id)
        .join(member_relative_path(member_path))
}

fn member_request(member: &Member) -> Result<MemberRequest, InstallError> {
    let sha256 = Sha256::parse_hex(&member.sha256).ok_or_else(|| InstallError::InvalidRecipe {
        detail: format!("member `{}` has a malformed SHA-256", member.id),
    })?;
    Ok(MemberRequest::new(member.path.clone())
        .with_size(member.size)
        .with_sha256(sha256))
}

/// Copies a bare-WAD package (spec 006's `PackageKind::Wad`) into place.
fn copy_bare_wad(cached: &Path, dest_dir: &Path, member: &Member) -> Result<(), InstallError> {
    let destination = dest_dir.join(member_relative_path(&member.path));
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| io_error("create install subdirectory", error))?;
    }
    fs::copy(cached, &destination).map_err(|error| io_error("place WAD package", error))?;
    restrict_permissions(&destination)?;
    Ok(())
}

/// Creates a unique staging directory under `library_root`.
fn create_staging_dir(library_root: &Path) -> Result<PathBuf, InstallError> {
    fs::create_dir_all(library_root)
        .map_err(|error| io_error("create library directory", error))?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    for _ in 0..100 {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = library_root.join(format!(".staging-{}-{unique}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_error("create install staging directory", error)),
        }
    }
    Err(InstallError::Io {
        context: "create install staging directory".to_owned(),
        detail: "could not find a unique name".to_owned(),
    })
}

/// The content identity of a parsed recipe: SHA-256 of its canonical TOML form.
///
/// This is what the install record pins (spec 009, R2). It is stable across
/// whitespace and comment changes but changes whenever the recipe's meaning
/// changes, so a catalog update is detected instead of silently applied.
fn recipe_content_sha256(recipe: &Recipe) -> Result<String, InstallError> {
    let text =
        toml::to_string(recipe).map_err(|error| InstallError::Serialize(error.to_string()))?;
    Ok(Sha256::of(text.as_bytes()).to_hex())
}

/// `Some(value)` when `value` is a safe single path component.
fn safe_component(value: &str) -> Option<&str> {
    if value.is_empty() || value == "." || value == ".." || value.contains(['/', '\\', '\0']) {
        None
    } else {
        Some(value)
    }
}

/// Hashes a file in bounded chunks so a large import never loads whole.
fn hash_file(path: &Path, cancel: &CancellationToken) -> Result<String, InstallError> {
    let mut file = fs::File::open(path).map_err(|error| io_error("open imported file", error))?;
    let mut hasher = Sha256Digest::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        check_cancel(cancel)?;
        let read = file
            .read(&mut buffer)
            .map_err(|error| io_error("read imported file", error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(Sha256::finish(hasher).to_hex())
}

/// Strips every permission bit that could make an installed file executable.
#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), InstallError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644))
        .map_err(|error| io_error("set installed file permissions", error))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), InstallError> {
    Ok(())
}

fn io_error(context: &str, error: io::Error) -> InstallError {
    InstallError::Io {
        context: context.to_owned(),
        detail: error.to_string(),
    }
}

/// Why an install, update or import failed.
#[derive(Debug)]
pub enum InstallError {
    /// A filesystem operation failed.
    Io {
        /// What DDL was doing.
        context: String,
        /// The underlying error.
        detail: String,
    },
    /// A download or extraction failed (spec 006).
    Download(crate::downloads::Error),
    /// The install record is invalid.
    Manifest(ManifestError),
    /// The recipe could not be serialized for its content identity.
    Serialize(String),
    /// The recipe or selection is structurally invalid.
    InvalidRecipe {
        /// What is wrong.
        detail: String,
    },
    /// A path or file name is not safe to place.
    UnsafePath {
        /// What is unsafe.
        detail: String,
    },
    /// The recipe changed after this mapset was installed.
    RecipeChanged {
        /// Recipe id.
        id: String,
        /// Content identity the install was made from.
        installed: String,
        /// Content identity of the recipe now.
        current: String,
    },
    /// The mapset has not been installed yet.
    NotInstalled,
    /// The component id is not in the recipe.
    UnknownComponent {
        /// The unknown component id.
        component: String,
    },
    /// The component is not user-supplied, so it cannot be imported.
    NotUserSupplied {
        /// The component id.
        component: String,
    },
    /// The imported file is larger than DDL accepts.
    ImportTooLarge {
        /// The accepted maximum in bytes.
        limit: u64,
        /// The actual size.
        got: u64,
    },
    /// The imported file is empty.
    EmptyFile {
        /// The offending path.
        path: PathBuf,
    },
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { context, detail } => write!(f, "{context}: {detail}"),
            Self::Download(error) => write!(f, "{error}"),
            Self::Manifest(error) => write!(f, "{error}"),
            Self::Serialize(detail) => write!(f, "could not identify the recipe: {detail}"),
            Self::InvalidRecipe { detail } => write!(f, "invalid recipe for install: {detail}"),
            Self::UnsafePath { detail } => write!(f, "unsafe install path: {detail}"),
            Self::RecipeChanged { id, .. } => write!(
                f,
                "`{id}` has changed in the catalog since it was installed; re-install it to update"
            ),
            Self::NotInstalled => f.write_str("this mapset is not installed"),
            Self::UnknownComponent { component } => {
                write!(f, "the recipe has no component `{component}`")
            }
            Self::NotUserSupplied { component } => write!(
                f,
                "component `{component}` is downloaded automatically, so it cannot be imported"
            ),
            Self::ImportTooLarge { limit, got } => write!(
                f,
                "the file is too large to import (limit {limit} bytes, got {got} bytes)"
            ),
            Self::EmptyFile { path } => {
                write!(f, "`{}` is empty and cannot be imported", path.display())
            }
        }
    }
}

impl std::error::Error for InstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Download(error) => Some(error),
            Self::Manifest(error) => Some(error),
            _ => None,
        }
    }
}

impl From<crate::downloads::Error> for InstallError {
    fn from(error: crate::downloads::Error) -> Self {
        Self::Download(error)
    }
}

impl From<ManifestError> for InstallError {
    fn from(error: ManifestError) -> Self {
        Self::Manifest(error)
    }
}

fn check_cancel(cancel: &CancellationToken) -> Result<(), InstallError> {
    if cancel.is_cancelled() {
        Err(InstallError::Download(crate::downloads::Error::Cancelled))
    } else {
        Ok(())
    }
}

fn record_matches(
    entry: &InstalledMember,
    cancel: &CancellationToken,
) -> Result<bool, InstallError> {
    check_cancel(cancel)?;
    if !fs::metadata(&entry.path)
        .is_ok_and(|m| m.is_file() && m.len() == entry.size && m.len() <= MAX_MEMBER_BYTES)
    {
        return Ok(false);
    }
    match hash_file(&entry.path, cancel) {
        Ok(hash) => Ok(hash.eq_ignore_ascii_case(&entry.sha256)),
        Err(error @ InstallError::Download(crate::downloads::Error::Cancelled)) => Err(error),
        Err(error) => {
            tracing::warn!(%error,path=%entry.path.display(),"could not verify prior installed member");
            Ok(false)
        }
    }
}
