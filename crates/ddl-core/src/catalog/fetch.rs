//! Opt-in package verification for `ddl catalog lint --fetch` (spec 002, R9).
//!
//! Since spec 009 (R8) this module no longer carries its own HTTP client: the
//! real [`HttpsFetch`] runs on [`crate::downloads`], the single hardened
//! download path DDL uses for engine AppImages and mapset packages. That gives
//! the lint the same HTTPS-only policy, exact host allowlist, redirect limit,
//! size bounds, mirror fallback and SHA-256 verification as an install.
//!
//! [`Fetch`] remains as the injection seam, so lint tests still never touch a
//! socket. [`check_package`] is unchanged: it re-checks the declared archive
//! size and hash and reads **only** the named members, refusing traversal,
//! symlinks and over-large decompression (spec 002, R4).

use std::path::PathBuf;

use crate::downloads::{CancellationToken, DownloadRequest, Fetcher, HostAllowlist, Sha256};

use super::lint::MAX_BYTES;
use super::{Hosts, Package, PackageKind};

/// The download boundary used by `catalog lint --fetch`.
pub trait Fetch {
    /// Downloads `package`, enforcing HTTPS, the host allowlist, the size bound
    /// and the declared SHA-256, and returns its bytes.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason suitable for a lint diagnostic.
    fn download(&mut self, package: &Package, hosts: &Hosts) -> Result<Vec<u8>, String>;
}

/// The real boundary, backed by spec 006's [`Fetcher`].
pub struct HttpsFetch {
    cache_dir: PathBuf,
    #[cfg(test)]
    allow_http: bool,
}

impl HttpsFetch {
    /// Creates a fetcher whose completed downloads are cached under
    /// `cache_dir/downloads` (spec 006, R3).
    #[must_use]
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            #[cfg(test)]
            allow_http: false,
        }
    }

    /// Allows plain-HTTP URLs. Test-only: production is HTTPS-only.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn allow_http_for_tests(mut self) -> Self {
        self.allow_http = true;
        self
    }
}

impl Fetch for HttpsFetch {
    fn download(&mut self, package: &Package, hosts: &Hosts) -> Result<Vec<u8>, String> {
        // `lint::validate` already requires non-empty URLs, but this trait can be
        // called directly, so guard before indexing.
        let Some(first) = package.urls.first() else {
            return Err(format!("package `{}` lists no download URLs", package.id));
        };
        // Scheme and host policy live in the fetcher itself (spec 006, R1):
        // production refuses plain HTTP and contacts only allowlisted hosts, and
        // every redirect target is re-checked. This is exactly the single
        // hardened path R8 asks lint to share.
        let expected = Sha256::parse_hex(&package.sha256)
            .ok_or_else(|| format!("package `{}` has a malformed SHA-256", package.id))?;

        let allowlist = HostAllowlist::new(hosts.downloads.iter().cloned());
        let request = DownloadRequest::new(first.clone(), expected, MAX_BYTES, allowlist)
            .with_urls(package.urls.iter().cloned())
            .with_size(package.size);

        #[allow(unused_mut)]
        let mut fetcher = Fetcher::new(&self.cache_dir);
        #[cfg(test)]
        {
            if self.allow_http {
                fetcher = fetcher.allow_http_for_tests();
            }
        }

        let cancel = CancellationToken::new();
        let mut progress = |_progress| {};
        let path = fetcher
            .download(&request, &cancel, &mut progress)
            .map_err(|error| error.to_string())?;
        std::fs::read(&path).map_err(|error| format!("read cached package: {error}"))
    }
}

pub(crate) fn fetch_package(
    package: &Package,
    hosts: &Hosts,
    fetch: &mut dyn Fetch,
) -> Result<(), String> {
    let bytes = fetch
        .download(package, hosts)
        .map_err(|error| format!("package `{}`: {error}", package.id))?;
    check_package(package, &bytes).map_err(|error| format!("package `{}`: {error}", package.id))
}

fn check_bytes(bytes: &[u8], size: u64, hash: &str, label: &str) -> Result<(), String> {
    if size > MAX_BYTES || bytes.len() as u64 != size {
        return Err(format!(
            "{label}: size mismatch (expected {size}, got {})",
            bytes.len()
        ));
    }
    let actual = Sha256::of(bytes).to_hex();
    if !actual.eq_ignore_ascii_case(hash) {
        return Err(format!("{label}: SHA-256 mismatch"));
    }
    Ok(())
}

/// Checks declared bytes and every named member, without writing or executing
/// archive content. Reads only named members, rejects symlinks/unsafe names,
/// caps ZIP entry count, nesting, each read and aggregate decompressed bytes.
pub fn check_package(package: &Package, bytes: &[u8]) -> Result<(), String> {
    check_bytes(bytes, package.size, &package.sha256, &package.id)?;
    let mut budget = MAX_BYTES;
    for member in &package.members {
        if !super::lint::safe_path(&member.path) {
            return Err(format!("unsafe member path `{}`", member.path));
        }
        let data = match package.kind {
            PackageKind::Zip => zip_member(bytes, &member.path, &mut budget)?,
            PackageKind::Wad => bytes.to_vec(),
        };
        check_bytes(&data, member.size, &member.sha256, &member.path)?;
    }
    Ok(())
}

fn zip_member(bytes: &[u8], path: &str, budget: &mut u64) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("invalid ZIP: {e}"))?;
    if archive.len() > 10_000 {
        return Err("ZIP exceeds 10000 entries".into());
    }
    let mut names = std::collections::HashSet::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let entry_name = entry.name().map_err(|e| e.to_string())?;
        let name = entry_name.strip_suffix('/').unwrap_or(&entry_name);
        if !super::lint::safe_path(name) || name.contains("!/") {
            return Err(format!("unsafe ZIP entry `{entry_name}`"));
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(format!("symlink ZIP entry `{entry_name}`"));
        }
        if !names.insert(entry_name.to_string()) {
            return Err(format!("duplicate ZIP entry `{entry_name}`"));
        }
    }
    let (name, nested) = path
        .split_once("!/")
        .map_or((path, None), |(a, b)| (a, Some(b)));
    let mut entry = archive
        .by_name(name)
        .map_err(|_| format!("missing exact member `{name}`"))?;
    if entry.is_dir() {
        return Err(format!("member `{name}` is a directory"));
    }
    if entry.size() > *budget {
        return Err("decompressed ZIP data exceeds 256 MiB budget".into());
    }
    let mut data = Vec::new();
    use std::io::Read as _;
    (&mut entry)
        .take(*budget + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() as u64 > *budget {
        return Err("decompressed ZIP data exceeds 256 MiB budget".into());
    }
    *budget -= data.len() as u64;
    match nested {
        Some(path) => zip_member(&data, path, budget),
        None => Ok(data),
    }
}
