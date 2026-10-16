use super::{
    Hosts, Package, PackageKind,
    lint::{MAX_BYTES, checked_url, safe_path},
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    io::{Cursor, Read},
    time::{Duration, Instant},
};

/// Injectable network boundary; tests use in-memory bytes and never access a socket.
pub trait Fetch {
    fn get(&mut self, url: &str, max_bytes: u64, hosts: &Hosts) -> Result<Vec<u8>, String>;
}

pub struct HttpsFetch;
impl Fetch for HttpsFetch {
    fn get(&mut self, url: &str, max_bytes: u64, hosts: &Hosts) -> Result<Vec<u8>, String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut url = checked_url(url, &hosts.downloads)?;
        for _ in 0..=5 {
            let timeout = deadline
                .checked_duration_since(Instant::now())
                .ok_or("download timed out")?;
            let agent = ureq::Agent::config_builder()
                .max_redirects(0)
                .timeout_global(Some(timeout))
                .build()
                .new_agent();
            let mut response = agent.get(url.as_str()).call().map_err(|e| e.to_string())?;
            if response.status().is_redirection() {
                let location = response
                    .headers()
                    .get("location")
                    .ok_or("redirect has no Location")?
                    .to_str()
                    .map_err(|_| "invalid redirect Location")?;
                let next = url.join(location).map_err(|e| e.to_string())?;
                url = checked_url(next.as_str(), &hosts.downloads)?;
                continue;
            }
            if !response.status().is_success() {
                return Err(format!("unexpected HTTP status {}", response.status()));
            }
            let mut bytes = Vec::new();
            response
                .body_mut()
                .as_reader()
                .take(max_bytes.min(MAX_BYTES) + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > max_bytes.min(MAX_BYTES) {
                return Err("download exceeds declared size/256 MiB limit".into());
            }
            return Ok(bytes);
        }
        Err("too many redirects (maximum 5)".into())
    }
}

pub(crate) fn fetch_package(
    p: &Package,
    hosts: &Hosts,
    fetch: &mut dyn Fetch,
) -> Result<(), String> {
    let mut errors = Vec::new();
    for url in &p.urls {
        let result = checked_url(url, &hosts.downloads)
            .and_then(|_| fetch.get(url, p.size, hosts))
            .and_then(|bytes| check_package(p, &bytes));
        match result {
            Ok(()) => return Ok(()),
            Err(e) => errors.push(format!("{url}: {e}")),
        }
    }
    Err(format!("package `{}`: {}", p.id, errors.join("; ")))
}

fn check_bytes(bytes: &[u8], size: u64, hash: &str, label: &str) -> Result<(), String> {
    if size > MAX_BYTES || bytes.len() as u64 != size {
        return Err(format!(
            "{label}: size mismatch (expected {size}, got {})",
            bytes.len()
        ));
    }
    let actual = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if !actual.eq_ignore_ascii_case(hash) {
        return Err(format!("{label}: SHA-256 mismatch"));
    }
    Ok(())
}

/// Checks declared bytes and every named member, without writing or executing
/// archive content. Reads only named members, rejects symlinks/unsafe names,
/// caps ZIP entry count, nesting, each read and aggregate decompressed bytes.
pub fn check_package(p: &Package, bytes: &[u8]) -> Result<(), String> {
    check_bytes(bytes, p.size, &p.sha256, &p.id)?;
    let mut budget = MAX_BYTES;
    for m in &p.members {
        if !safe_path(&m.path) {
            return Err(format!("unsafe member path `{}`", m.path));
        }
        let member = match p.kind {
            PackageKind::Zip => zip_member(bytes, &m.path, &mut budget)?,
            PackageKind::Wad => bytes.to_vec(),
        };
        check_bytes(&member, m.size, &m.sha256, &m.path)?;
    }
    Ok(())
}

fn zip_member(bytes: &[u8], path: &str, budget: &mut u64) -> Result<Vec<u8>, String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("invalid ZIP: {e}"))?;
    if archive.len() > 10_000 {
        return Err("ZIP exceeds 10000 entries".into());
    }
    let mut names = HashSet::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let entry_name = entry.name().map_err(|e| e.to_string())?;
        let name = entry_name.strip_suffix('/').unwrap_or(&entry_name);
        if !safe_path(name) || name.contains("!/") {
            return Err(format!("unsafe ZIP entry `{}`", entry_name));
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(format!("symlink ZIP entry `{}`", entry_name));
        }
        if !names.insert(entry_name.to_string()) {
            return Err(format!("duplicate ZIP entry `{}`", entry_name));
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
