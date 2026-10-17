//! Content-based IWAD identification (spec 007, R2).
//!
//! [`Identifier::identify`] never trusts a file name. It performs a bounded
//! header + lump-directory read, rejects anything that is not a WAD or is too
//! large, matches the file against the strongest published hash of every
//! same-sized table entry, and otherwise falls back to DSDA-Doom's own lump
//! rules (research §2) to classify the *family*.
//!
//! Two deliberate quirks, both from the research:
//!
//! * A file whose magic is `PWAD` is not rejected when its hash matches a known
//!   variant — the BFG Edition and European PSN IWADs are mis-tagged `PWAD`.
//!   A `PWAD` that matches nothing is treated as `Unknown` (it is a mod, not a
//!   base IWAD), so lump heuristics only run for `IWAD`-tagged files.
//! * Within the Doom II family (Doom II / TNT / Plutonia) no lump distinguishes
//!   the games, so a hash-less file is reported as [`Outcome::Ambiguous`] for
//!   the user to resolve; DDL never guesses.
//!
//! SHA-256 is always computed and returned; SHA-1/MD5/CRC-32 are computed only
//! when a same-sized table entry actually needs them (the size pre-filter).

use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use sha2::Digest;

use super::table::{Game, HashKind, IwadTable};

/// Largest file DDL will identify, in bytes. Real IWADs are under 20 MiB.
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Largest lump directory DDL will read (1 MiB of 16-byte entries).
pub const MAX_LUMPS: u32 = 65_536;

const HEADER_LEN: u64 = 12;
const LUMP_LEN: usize = 16;

/// The game family DSDA-Doom's lump rules can tell apart (research §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// A single shareware episode (`DOOM1.WAD`).
    Shareware,
    /// Registered DOOM (episodes 1–3).
    Registered,
    /// Ultimate DOOM (adds episode 4).
    Ultimate,
    /// The Doom II family (Doom II, TNT, Plutonia), which lumps cannot split.
    Commercial,
}

impl Family {
    /// A short lowercase label for storage and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Family::Shareware => "shareware",
            Family::Registered => "registered",
            Family::Ultimate => "ultimate",
            Family::Commercial => "commercial",
        }
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What identification concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A published hash matched exactly one table variant.
    Known {
        /// The matched variant id.
        id: String,
        /// The game that variant belongs to.
        game: Game,
    },
    /// The family is known but the exact variant is not; the user decides.
    Ambiguous {
        /// The family the lumps implied.
        family: Family,
        /// Candidate variant ids, in table order.
        candidates: Vec<String>,
    },
    /// A WAD, but not one DDL recognises as a base game.
    Unknown,
}

/// The result of identifying a file. SHA-256 is always present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identification {
    /// SHA-256 of the file, always computed.
    pub sha256: String,
    /// File size in bytes.
    pub size: u64,
    /// What identification concluded.
    pub outcome: Outcome,
}

impl Identification {
    /// The variant id when identification was exact.
    #[must_use]
    pub fn variant_id(&self) -> Option<&str> {
        match &self.outcome {
            Outcome::Known { id, .. } => Some(id),
            _ => None,
        }
    }

    /// The game when identification was exact.
    #[must_use]
    pub fn game(&self) -> Option<Game> {
        match &self.outcome {
            Outcome::Known { game, .. } => Some(*game),
            _ => None,
        }
    }
}

/// Why identification failed.
#[derive(Debug)]
pub enum IdentifyError {
    /// The file could not be read.
    Io(io::Error),
    /// The path is not a regular file.
    NotAFile,
    /// The 4-byte magic is neither `IWAD` nor `PWAD`, or the header is bogus.
    NotWad,
    /// The file exceeds [`MAX_FILE_BYTES`].
    Oversize {
        /// The offending size.
        size: u64,
        /// The limit that was exceeded.
        limit: u64,
    },
    /// The lump directory declares more entries than [`MAX_LUMPS`].
    DirectoryTooLarge {
        /// The declared lump count.
        lumps: u32,
        /// The limit that was exceeded.
        limit: u32,
    },
}

impl fmt::Display for IdentifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifyError::Io(error) => write!(f, "cannot read the file: {error}"),
            IdentifyError::NotAFile => f.write_str("not a regular file"),
            IdentifyError::NotWad => {
                f.write_str("not a WAD: header magic is not IWAD or PWAD, or offsets are invalid")
            }
            IdentifyError::Oversize { size, limit } => {
                write!(f, "file is {size} bytes, over the {limit}-byte limit")
            }
            IdentifyError::DirectoryTooLarge { lumps, limit } => {
                write!(f, "WAD declares {lumps} lumps, over the {limit}-lump limit")
            }
        }
    }
}

impl std::error::Error for IdentifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IdentifyError::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Identifies IWADs against a loaded [`IwadTable`].
#[derive(Debug, Clone)]
pub struct Identifier {
    table: IwadTable,
}

impl Identifier {
    /// Builds an identifier over `table`.
    #[must_use]
    pub fn new(table: IwadTable) -> Self {
        Self { table }
    }

    /// Builds an identifier over the table embedded in the binary.
    #[must_use]
    pub fn embedded() -> Self {
        Self::new(IwadTable::embedded())
    }

    /// The table in use.
    #[must_use]
    pub fn table(&self) -> &IwadTable {
        &self.table
    }

    /// Identifies the file at `path`.
    ///
    /// # Errors
    ///
    /// Returns an [`IdentifyError`] when the file cannot be read, is not a WAD,
    /// or exceeds the size or directory limits.
    pub fn identify(&self, path: &Path) -> Result<Identification, IdentifyError> {
        identify(path, &self.table)
    }
}

/// Convenience wrapper around [`Identifier::identify`].
///
/// # Errors
///
/// See [`Identifier::identify`].
pub fn identify(path: &Path, table: &IwadTable) -> Result<Identification, IdentifyError> {
    let metadata = std::fs::metadata(path).map_err(IdentifyError::Io)?;
    if !metadata.is_file() {
        return Err(IdentifyError::NotAFile);
    }
    let size = metadata.len();
    if size > MAX_FILE_BYTES {
        return Err(IdentifyError::Oversize {
            size,
            limit: MAX_FILE_BYTES,
        });
    }

    let header = read_header(path, size)?;
    let lumps = read_directory(path, &header, size)?;

    let size_matches = table.variants.iter().any(|variant| variant.size == size);
    let wanted = wanted_hashes(table, size, size_matches);
    let digests = hash_file(path, &wanted).map_err(IdentifyError::Io)?;
    let sha256 = digests.sha256.clone().expect("SHA-256 is always computed");

    if size_matches {
        let matches: Vec<&str> = table
            .variants
            .iter()
            .filter(|variant| variant.size == size)
            .filter(|variant| {
                variant.strongest_hash().is_some_and(|(kind, expected)| {
                    digests
                        .get(kind)
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
                })
            })
            .map(|variant| variant.id.as_str())
            .collect();

        match matches.as_slice() {
            [] => {}
            [id] => {
                let game = table.get(id).map_or(Game::Doom, |variant| variant.game);
                return Ok(Identification {
                    sha256,
                    size,
                    outcome: Outcome::Known {
                        id: (*id).to_owned(),
                        game,
                    },
                });
            }
            many => {
                let family = classify(&lumps).unwrap_or(Family::Commercial);
                return Ok(Identification {
                    sha256,
                    size,
                    outcome: Outcome::Ambiguous {
                        family,
                        candidates: many.iter().map(|id| (*id).to_owned()).collect(),
                    },
                });
            }
        }
    }

    // No hash match. A PWAD is a mod unless it matched a known variant above.
    let family = match header.magic {
        Magic::Pwad => None,
        Magic::Iwad => classify(&lumps),
    };

    let outcome = match family {
        Some(family) => {
            let candidates = candidates_for(table, family);
            if candidates.is_empty() {
                Outcome::Unknown
            } else {
                Outcome::Ambiguous { family, candidates }
            }
        }
        None => Outcome::Unknown,
    };

    Ok(Identification {
        sha256,
        size,
        outcome,
    })
}

/// Computes a file's SHA-256 without the rest of identification.
///
/// # Errors
///
/// Returns the underlying I/O error when the file cannot be read.
pub fn sha256_file(path: &Path) -> io::Result<String> {
    hash_file(path, &Wanted::default())
        .map(|digests| digests.sha256.expect("SHA-256 is always computed"))
}

/// Computes the SHA-256 of an in-memory buffer, as lowercase hex.
#[must_use]
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    to_hex(&hasher.finalize())
}

/// Which optional digests a pass must compute.
#[derive(Debug, Default, Clone, Copy)]
struct Wanted {
    sha1: bool,
    md5: bool,
    crc32: bool,
}

/// The digests produced by a hashing pass.
#[derive(Debug, Default, Clone)]
struct Digests {
    sha256: Option<String>,
    sha1: Option<String>,
    md5: Option<String>,
    crc32: Option<String>,
}

impl Digests {
    fn get(&self, kind: HashKind) -> Option<&str> {
        match kind {
            HashKind::Sha256 => self.sha256.as_deref(),
            HashKind::Sha1 => self.sha1.as_deref(),
            HashKind::Md5 => self.md5.as_deref(),
            HashKind::Crc32 => self.crc32.as_deref(),
        }
    }
}

/// Decides which digests a same-sized table entry needs (the size pre-filter).
fn wanted_hashes(table: &IwadTable, size: u64, size_matches: bool) -> Wanted {
    let mut wanted = Wanted::default();
    if !size_matches {
        return wanted;
    }
    for variant in table.variants.iter().filter(|variant| variant.size == size) {
        match variant.strongest_hash() {
            Some((HashKind::Sha1, _)) => wanted.sha1 = true,
            Some((HashKind::Md5, _)) => wanted.md5 = true,
            Some((HashKind::Crc32, _)) => wanted.crc32 = true,
            _ => {}
        }
    }
    wanted
}

/// Streams `path` once, computing SHA-256 and any requested extra digests.
fn hash_file(path: &Path, wanted: &Wanted) -> io::Result<Digests> {
    let mut file = File::open(path)?;
    let mut sha256 = sha2::Sha256::new();
    let mut sha1 = wanted.sha1.then(sha1::Sha1::new);
    let mut md5 = wanted.md5.then(md5::Md5::new);
    let mut crc32 = wanted.crc32.then(crc32fast::Hasher::new);

    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        sha256.update(chunk);
        if let Some(hasher) = sha1.as_mut() {
            hasher.update(chunk);
        }
        if let Some(hasher) = md5.as_mut() {
            hasher.update(chunk);
        }
        if let Some(hasher) = crc32.as_mut() {
            hasher.update(chunk);
        }
    }

    Ok(Digests {
        sha256: Some(to_hex(&sha256.finalize())),
        sha1: sha1.map(|hasher| to_hex(&hasher.finalize())),
        md5: md5.map(|hasher| to_hex(&hasher.finalize())),
        crc32: crc32.map(|hasher| format!("{:08x}", hasher.finalize())),
    })
}

/// The 4-byte header magic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Magic {
    Iwad,
    Pwad,
}

/// A validated WAD header.
#[derive(Debug, Clone, Copy)]
struct Header {
    magic: Magic,
    numlumps: u32,
    infotableofs: u32,
}

/// Reads and sanity-checks the 12-byte header.
fn read_header(path: &Path, size: u64) -> Result<Header, IdentifyError> {
    if size < HEADER_LEN {
        return Err(IdentifyError::NotWad);
    }
    let mut file = File::open(path).map_err(IdentifyError::Io)?;
    let mut raw = [0_u8; HEADER_LEN as usize];
    file.read_exact(&mut raw).map_err(IdentifyError::Io)?;

    let magic = match &raw[..4] {
        b"IWAD" => Magic::Iwad,
        b"PWAD" => Magic::Pwad,
        _ => return Err(IdentifyError::NotWad),
    };

    let numlumps_raw = i32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
    let infotableofs_raw = i32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]);
    if numlumps_raw < 0 || infotableofs_raw < 0 {
        return Err(IdentifyError::NotWad);
    }
    let numlumps = numlumps_raw as u32;
    if numlumps > MAX_LUMPS {
        return Err(IdentifyError::DirectoryTooLarge {
            lumps: numlumps,
            limit: MAX_LUMPS,
        });
    }
    let infotableofs = infotableofs_raw as u32;
    let directory_end = u64::from(infotableofs) + u64::from(numlumps) * LUMP_LEN as u64;
    if u64::from(infotableofs) < HEADER_LEN || directory_end > size {
        return Err(IdentifyError::NotWad);
    }

    Ok(Header {
        magic,
        numlumps,
        infotableofs,
    })
}

/// Reads the lump directory and returns the lump names.
fn read_directory(path: &Path, header: &Header, size: u64) -> Result<Vec<String>, IdentifyError> {
    if header.numlumps == 0 {
        return Ok(Vec::new());
    }
    let bytes = u64::from(header.numlumps) * LUMP_LEN as u64;
    let mut file = File::open(path).map_err(IdentifyError::Io)?;
    file.seek(SeekFrom::Start(u64::from(header.infotableofs)))
        .map_err(IdentifyError::Io)?;
    let mut raw = vec![0_u8; bytes as usize];
    file.read_exact(&mut raw).map_err(IdentifyError::Io)?;
    debug_assert!(bytes <= size);

    let mut names = Vec::with_capacity(header.numlumps as usize);
    for entry in raw.as_chunks::<LUMP_LEN>().0 {
        let name = &entry[8..16];
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        names.push(String::from_utf8_lossy(&name[..end]).into_owned());
    }
    Ok(names)
}

/// Applies DSDA-Doom's lump rules (research §2) to classify the family.
///
/// Returns `None` for content DDL does not treat as a base game (FreeDM, Hacx,
/// or anything whose markers do not fire).
fn classify(lumps: &[String]) -> Option<Family> {
    let mut sw = 0_u32;
    let mut rg = 0_u32;
    let mut ud = 0_u32;
    let mut cm = 0_u32;
    let mut hacx = false;
    let mut freedm = false;

    for name in lumps {
        if episode_map(name, b'1') {
            sw += 1;
        } else if episode_map(name, b'2') || episode_map(name, b'3') {
            rg += 1;
        } else if episode_map(name, b'4') {
            ud += 1;
        }
        if map_index(name).is_some() {
            cm += 1;
        }
        if name.starts_with("HACX") {
            hacx = true;
        } else if name.starts_with("FREEDM") {
            freedm = true;
        }
    }

    if hacx || freedm {
        return None;
    }
    if cm >= 30 {
        Some(Family::Commercial)
    } else if ud >= 9 {
        Some(Family::Ultimate)
    } else if rg >= 18 {
        Some(Family::Registered)
    } else if sw >= 9 {
        Some(Family::Shareware)
    } else {
        None
    }
}

/// `true` for `E<episode>M1` … `E<episode>M9`.
fn episode_map(name: &str, episode: u8) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 4
        && bytes[0] == b'E'
        && bytes[1] == episode
        && bytes[2] == b'M'
        && (b'1'..=b'9').contains(&bytes[3])
}

/// The `xx` of a `MAPxx` lump with `01 <= xx <= 32`.
fn map_index(name: &str) -> Option<u32> {
    let bytes = name.as_bytes();
    if bytes.len() != 5 || &bytes[..3] != b"MAP" {
        return None;
    }
    if !bytes[3].is_ascii_digit() || !bytes[4].is_ascii_digit() {
        return None;
    }
    let index = u32::from(bytes[3] - b'0') * 10 + u32::from(bytes[4] - b'0');
    (1..=32).contains(&index).then_some(index)
}

/// The table variants a family's candidates can be drawn from.
fn candidates_for(table: &IwadTable, family: Family) -> Vec<String> {
    table
        .variants
        .iter()
        .filter(|variant| match family {
            Family::Shareware => variant.game == Game::Doom && variant.filename == "doom1.wad",
            Family::Registered => variant.game == Game::Doom && variant.filename == "doom.wad",
            Family::Ultimate => variant.game == Game::Doom && variant.filename == "doomu.wad",
            Family::Commercial => matches!(variant.game, Game::Doom2 | Game::Tnt | Game::Plutonia),
        })
        .map(|variant| variant.id.clone())
        .collect()
}

/// Lowercase hex for a digest.
fn to_hex(bytes: &[u8]) -> String {
    use fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;
    use crate::iwads::table::IwadVariant;

    fn wad_bytes(magic: &[u8; 4], lumps: &[&str]) -> Vec<u8> {
        let mut directory = Vec::new();
        for name in lumps {
            directory.extend_from_slice(&0_i32.to_le_bytes()); // filepos
            directory.extend_from_slice(&0_i32.to_le_bytes()); // size
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

    fn episodes(range: std::ops::RangeInclusive<u8>) -> Vec<String> {
        let mut lumps = Vec::new();
        for episode in range {
            for map in 1..=9 {
                lumps.push(format!("E{episode}M{map}"));
            }
        }
        lumps
    }

    fn names(lumps: &[String]) -> Vec<&str> {
        lumps.iter().map(String::as_str).collect()
    }

    fn write_wad(dir: &TempDir, name: &str, magic: &[u8; 4], lumps: &[&str]) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, wad_bytes(magic, lumps)).expect("write fixture");
        path
    }

    fn test_variant(
        id: &str,
        game: Game,
        filename: &str,
        size: u64,
        sha256: String,
    ) -> IwadVariant {
        IwadVariant {
            id: id.to_owned(),
            game,
            title: id.to_owned(),
            filename: filename.to_owned(),
            size,
            sha256: Some(sha256),
            md5: None,
            sha1: None,
            crc32: None,
            default_complevel: 2,
            playable: true,
            source: "https://example.invalid/test".to_owned(),
            hash_sources: BTreeMap::new(),
        }
    }

    fn sha1_bytes(bytes: &[u8]) -> String {
        use sha1::Digest as _;
        let mut hasher = sha1::Sha1::new();
        hasher.update(bytes);
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn ultimate_doom_lumps_are_ambiguous() {
        let dir = TempDir::new().expect("tempdir");
        let lumps = episodes(1..=4);
        let path = write_wad(&dir, "maybe.wad", b"IWAD", &names(&lumps));

        let identification = Identifier::embedded().identify(&path).expect("identifies");
        assert_eq!(identification.sha256.len(), 64);
        assert_eq!(identification.size, 12 + 16 * lumps.len() as u64);
        match identification.outcome {
            Outcome::Ambiguous { family, candidates } => {
                assert_eq!(family, Family::Ultimate);
                assert!(candidates.iter().any(|id| id == "doom-1.9ud"));
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn shareware_lumps_are_ambiguous() {
        let dir = TempDir::new().expect("tempdir");
        let lumps = episodes(1..=1);
        let path = write_wad(&dir, "shareware.wad", b"IWAD", &names(&lumps));

        let identification = Identifier::embedded().identify(&path).expect("identifies");
        match identification.outcome {
            Outcome::Ambiguous { family, candidates } => {
                assert_eq!(family, Family::Shareware);
                assert_eq!(candidates, vec!["doom-shareware-1.9".to_owned()]);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn doom2_lumps_are_ambiguous_across_the_family() {
        let dir = TempDir::new().expect("tempdir");
        let mut lumps = Vec::new();
        for map in 1..=32 {
            lumps.push(format!("MAP{map:02}"));
        }
        let path = write_wad(&dir, "maybe2.wad", b"IWAD", &names(&lumps));

        let identification = Identifier::embedded().identify(&path).expect("identifies");
        match identification.outcome {
            Outcome::Ambiguous { family, candidates } => {
                assert_eq!(family, Family::Commercial);
                assert!(candidates.iter().any(|id| id == "doom2-1.9"));
                assert!(candidates.iter().any(|id| id == "tnt-1.9"));
                assert!(candidates.iter().any(|id| id == "plutonia-1.9"));
                assert!(!candidates.iter().any(|id| id.starts_with("nerve")));
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn anthology_revisions_identify_by_size_and_sha1() {
        // A fixture cannot carry a commercial IWAD's published hash, so mirror
        // each new table entry onto a synthetic file: keep the entry's identity
        // (id/game/filename) and its strongest hash kind (SHA-1, the one
        // identification matches), substituting the fixture's real size and
        // SHA-1. The published literals themselves are asserted in table.rs.
        for (id, game, filename) in [
            ("tnt-anthology", Game::Tnt, "tnt.wad"),
            ("plutonia-anthology", Game::Plutonia, "plutonia.wad"),
        ] {
            let dir = TempDir::new().expect("tempdir");
            let lumps: Vec<String> = (1..=32).map(|map| format!("MAP{map:02}")).collect();
            let path = write_wad(&dir, filename, b"IWAD", &names(&lumps));
            let bytes = fs::read(&path).expect("read fixture");

            let mut variant = IwadTable::embedded()
                .get(id)
                .expect("new entry present")
                .clone();
            variant.size = bytes.len() as u64;
            variant.sha1 = Some(sha1_bytes(&bytes));

            let table = IwadTable {
                schema_version: 1,
                variants: vec![variant],
            };

            let identification = identify(&path, &table).expect("identifies");
            assert_eq!(
                identification.outcome,
                Outcome::Known {
                    id: id.to_owned(),
                    game,
                },
                "{id} must identify by size and SHA-1"
            );
        }
    }

    #[test]
    fn iwad_without_markers_is_unknown() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_wad(&dir, "junk.wad", b"IWAD", &["PLAYPAL", "COLORMAP"]);

        let identification = Identifier::embedded().identify(&path).expect("identifies");
        assert_eq!(identification.outcome, Outcome::Unknown);
        assert_eq!(identification.sha256.len(), 64);
    }

    #[test]
    fn pwad_without_a_hash_match_is_unknown() {
        let dir = TempDir::new().expect("tempdir");
        let mut lumps = Vec::new();
        for map in 1..=32 {
            lumps.push(format!("MAP{map:02}"));
        }
        let path = write_wad(&dir, "mod.wad", b"PWAD", &names(&lumps));

        let identification = Identifier::embedded().identify(&path).expect("identifies");
        assert_eq!(
            identification.outcome,
            Outcome::Unknown,
            "a hash-less PWAD is a mod, not an IWAD"
        );
    }

    #[test]
    fn pwad_magic_is_tolerated_when_the_hash_matches() {
        // The BFG / PSN quirk: a real IWAD tagged PWAD must still identify.
        let dir = TempDir::new().expect("tempdir");
        let path = write_wad(&dir, "bfg.wad", b"PWAD", &["PLAYPAL"]);
        let bytes = fs::read(&path).expect("read fixture");

        let variant = test_variant(
            "test-only-pwad",
            Game::Doom2,
            "doom2.wad",
            bytes.len() as u64,
            sha256_bytes(&bytes),
        );
        let table = IwadTable {
            schema_version: 1,
            variants: vec![variant],
        };

        let identification = identify(&path, &table).expect("identifies");
        assert_eq!(
            identification.outcome,
            Outcome::Known {
                id: "test-only-pwad".to_owned(),
                game: Game::Doom2,
            }
        );
    }

    #[test]
    fn hash_match_beats_the_size_pre_filter_only_for_equal_sizes() {
        let dir = TempDir::new().expect("tempdir");
        let path = write_wad(&dir, "m.wad", b"IWAD", &["PLAYPAL"]);
        let bytes = fs::read(&path).expect("read fixture");

        // Same hash, deliberately wrong size: the size pre-filter must skip it.
        let variant = test_variant(
            "wrong-size",
            Game::Doom2,
            "doom2.wad",
            bytes.len() as u64 + 1,
            sha256_bytes(&bytes),
        );
        let table = IwadTable {
            schema_version: 1,
            variants: vec![variant],
        };

        let identification = identify(&path, &table).expect("identifies");
        assert_eq!(identification.outcome, Outcome::Unknown);
    }

    #[test]
    fn oversize_files_are_rejected_before_reading() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("huge.wad");
        let file = File::create(&path).expect("create");
        file.set_len(MAX_FILE_BYTES + 1).expect("sparse file");

        let error = Identifier::embedded().identify(&path).expect_err("too big");
        assert!(matches!(error, IdentifyError::Oversize { .. }));
    }

    #[test]
    fn directory_larger_than_the_limit_is_rejected() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("many.wad");
        let mut header = Vec::new();
        header.extend_from_slice(b"IWAD");
        header.extend_from_slice(&((MAX_LUMPS + 1) as i32).to_le_bytes());
        header.extend_from_slice(&12_i32.to_le_bytes());
        fs::write(&path, &header).expect("write");

        let error = Identifier::embedded()
            .identify(&path)
            .expect_err("too many lumps");
        assert!(matches!(error, IdentifyError::DirectoryTooLarge { .. }));
    }

    #[test]
    fn non_wad_files_are_rejected() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("notawad.bin");
        fs::write(&path, b"this is definitely not a wad file").expect("write");

        let error = Identifier::embedded()
            .identify(&path)
            .expect_err("not a wad");
        assert!(matches!(error, IdentifyError::NotWad));
    }

    #[test]
    fn truncated_header_is_rejected() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("short.wad");
        fs::write(&path, b"IWAD").expect("write");

        let error = Identifier::embedded()
            .identify(&path)
            .expect_err("truncated");
        assert!(matches!(error, IdentifyError::NotWad));
    }

    #[test]
    fn reader_rejects_a_directory_that_runs_past_the_file() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("badoffset.wad");
        let mut header = Vec::new();
        header.extend_from_slice(b"IWAD");
        header.extend_from_slice(&100_i32.to_le_bytes()); // 100 lumps declared
        header.extend_from_slice(&12_i32.to_le_bytes()); // but no room for them
        fs::write(&path, &header).expect("write");

        let error = Identifier::embedded()
            .identify(&path)
            .expect_err("bad offset");
        assert!(matches!(error, IdentifyError::NotWad));
    }

    #[test]
    fn classification_matches_dsda_thresholds() {
        assert_eq!(classify(&episodes(1..=1)), Some(Family::Shareware));
        assert_eq!(classify(&episodes(1..=3)), Some(Family::Registered));
        assert_eq!(classify(&episodes(1..=4)), Some(Family::Ultimate));

        let mut freedm = episodes(1..=1);
        freedm.push("FREEDM".to_owned());
        assert_eq!(classify(&freedm), None);
    }
}
