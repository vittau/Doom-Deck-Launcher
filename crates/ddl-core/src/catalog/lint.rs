use super::*;
use std::{
    collections::HashSet,
    fmt, fs,
    path::{Path, PathBuf},
};
use url::Url;

/// Largest single package or named member the catalog accepts, and the aggregate
/// decompressed budget per package check (`fetch.rs`). Raised from 256 MiB to 512 MiB
/// so Eviternity II's 293,081,691-byte `Eviternity II.wad` fits; the exact size and
/// SHA-256 checks are the real integrity guarantee, so this is only an abuse bound.
/// Keep in sync with `library::install::MAX_MEMBER_BYTES`.
pub(crate) const MAX_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MAX_RECIPE_BYTES: u64 = 1024 * 1024;

/// Repository-relative directory holding the bundled screenshots (spec 023,
/// D-024). A recipe's `[[screenshots]]` `asset` is relative to this directory.
pub const SCREENSHOTS_DIR: &str = "crates/ddl/ui/assets/screenshots";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub hint: String,
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}; hint: {}",
            self.file.display(),
            self.line,
            self.message,
            self.hint
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub key: String,
    pub message: String,
    pub hint: String,
}

pub(crate) fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && s.as_bytes()[0].is_ascii_alphanumeric()
}
pub(crate) fn hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
pub(crate) fn safe_path(s: &str) -> bool {
    !s.is_empty()
        && !s.contains(['\\', ':', '\0'])
        && !s.starts_with('-')
        && s.split("!/").count() <= 4
        && s.split("!/").all(|part| {
            !part.contains('!')
                && part
                    .split('/')
                    .all(|c| !c.is_empty() && c != "." && c != "..")
        })
}

/// Status-bar/HUD lump-name prefixes a cosmetic-only "Widescreen status bar"
/// WAD may contain (spec 017, R4). Derived from the status-bar-only files in
/// NightFright's `doom_widehud` plus the lump families DSDA-Doom v0.30.1 draws
/// as part of the classic status bar (`st_stuff.c` `STBAR`/`STARMS`; the face
/// `STF*`, number/percent `STT*`/`STG*`/`STYS*`, key/armour `STK*`/`STC*`, the
/// Doom 1 disk/CD-ROM icons and the `BRDR_*` background tiles). Any lump that
/// does not match (maps, `DEHACKED`, `MAPINFO`, `UMAPINFO`, music, sounds,
/// textures, sprites) makes the WAD non-cosmetic.
const STATUS_BAR_LUMP_PREFIXES: &[&str] = &[
    "STBAR", "STARMS", "STF", "STT", "STG", "STK", "STYS", "STC", "STDISK", "STCDROM", "BRDR_",
];

/// A component whose label marks it as the widescreen status bar (spec 017, R4).
fn is_status_bar_component(c: &Component) -> bool {
    c.label.to_ascii_lowercase().contains("status bar")
}

/// Reads the lump names of a WAD/PWAD held in memory (spec 017, R4). Bounds the
/// header, the declared lump count and the directory so untrusted bytes cannot
/// force a large allocation or a slice panic.
pub fn wad_lump_names(bytes: &[u8]) -> Result<Vec<String>, String> {
    const HEADER: usize = 12;
    const ENTRY: usize = 16;
    const MAX_LUMPS: i32 = 200_000;
    if bytes.len() < HEADER || !matches!(&bytes[..4], b"IWAD" | b"PWAD") {
        return Err("not a WAD/PWAD (bad header magic)".into());
    }
    let numlumps = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let infotableofs = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if numlumps < 0 || infotableofs < 0 || numlumps > MAX_LUMPS {
        return Err("invalid WAD lump directory".into());
    }
    let start = infotableofs as usize;
    let end = start
        .checked_add(numlumps as usize * ENTRY)
        .ok_or("WAD lump directory overflows")?;
    if start < HEADER || end > bytes.len() {
        return Err("WAD lump directory is out of bounds".into());
    }
    let mut names = Vec::with_capacity(numlumps as usize);
    for entry in bytes[start..end].as_chunks::<ENTRY>().0 {
        let name = &entry[8..16];
        let len = name.iter().position(|b| *b == 0).unwrap_or(name.len());
        names.push(String::from_utf8_lossy(&name[..len]).into_owned());
    }
    Ok(names)
}

/// Rejects a status-bar WAD that carries any non-cosmetic lump (spec 017, R4).
pub fn check_status_bar_lumps(names: &[String]) -> Result<(), String> {
    for name in names {
        let upper = name.to_ascii_uppercase();
        if !STATUS_BAR_LUMP_PREFIXES
            .iter()
            .any(|prefix| upper.starts_with(prefix))
        {
            return Err(format!(
                "non-cosmetic lump `{name}` in a widescreen status-bar WAD"
            ));
        }
    }
    Ok(())
}

/// Whether `asset` is a safe relative screenshot path with an accepted image
/// extension (spec 023, R2).
pub(crate) fn screenshot_asset(asset: &str) -> bool {
    safe_path(asset) && !asset.contains("!/") && super::image::accepted_asset_extension(asset)
}

pub(crate) fn checked_url(s: &str, allowed: &[String]) -> Result<Url, String> {
    let u = Url::parse(s).map_err(|_| "use an absolute HTTPS URL".to_string())?;
    if u.scheme() != "https"
        || !u.username().is_empty()
        || u.password().is_some()
        || u.port().is_some_and(|p| p != 443)
        || u.fragment().is_some()
    {
        return Err("use HTTPS without credentials, custom ports or fragments".into());
    }
    if !u
        .host_str()
        .is_some_and(|host| allowed.iter().any(|a| a == host))
    {
        return Err(
            "use an exact host from catalog/hosts.toml (request review for a new host)".into(),
        );
    }
    Ok(u)
}

fn valid_date(s: &str) -> bool {
    let parts: Vec<_> = s.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let nums: Option<Vec<u32>> = parts.iter().map(|p| p.parse().ok()).collect();
    let Some(n) = nums else {
        return false;
    };
    let leap = n[0].is_multiple_of(4) && (!n[0].is_multiple_of(100) || n[0].is_multiple_of(400));
    let days = match n[1] {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    n[0] > 0 && n[2] > 0 && n[2] <= days
}

/// Linear/pairwise checks prove order for all combinations without an exponential
/// subset enumeration: each surviving pair must have distinct positions, unless
/// exclusivity or main replacement prevents that pair from ever coexisting.
pub fn validate(r: &Recipe, filename: &Path, hosts: &Hosts) -> Vec<Violation> {
    let mut errors = Vec::new();
    let mut issue = |key: String, bad: bool, message: &str, hint: &str| {
        if bad {
            errors.push(Violation {
                key,
                message: message.into(),
                hint: hint.into(),
            });
        }
    };
    issue(
        "schema_version".into(),
        r.schema_version != 1,
        "unsupported schema_version",
        "use schema_version = 1",
    );
    issue(
        "id".into(),
        !slug(&r.id),
        "invalid recipe id",
        "use a lowercase alphanumeric/hyphen slug",
    );
    issue(
        "id".into(),
        filename.extension().is_none_or(|ext| ext != "toml")
            || filename.file_stem().and_then(|s| s.to_str()) != Some(&r.id),
        "recipe id does not match file name",
        "name the file <id>.toml",
    );
    for (key, value) in [
        ("title", &r.title),
        ("release", &r.release),
        ("description", &r.description),
        ("permission", &r.permission),
    ] {
        issue(
            key.into(),
            value.trim().is_empty(),
            "missing identity/permission text",
            "supply nonempty text backed by the upstream release",
        );
    }
    issue(
        "authors".into(),
        r.authors.is_empty() || r.authors.iter().any(|a| a.trim().is_empty()),
        "missing authors",
        "credit the release authors",
    );
    issue(
        "release".into(),
        r.release.contains(['/', '\\']),
        "invalid release label",
        "release becomes an install path component; remove `/` and `\\`",
    );
    issue(
        "page".into(),
        checked_url(&r.page, &hosts.pages).is_err(),
        "invalid canonical page URL",
        "use HTTPS and an allowlisted page host",
    );
    for (i, image) in r.images.iter().enumerate() {
        issue(
            format!("images.{i}.path"),
            !safe_path(&image.path) || image.path.contains("!/"),
            "unsafe image reference",
            "use a relative path with no traversal",
        );
        issue(
            format!("images.{i}.permission"),
            image.permission.trim().is_empty(),
            "missing image permission",
            "state permission to use the image",
        );
    }
    let mut screenshot_assets = HashSet::new();
    for (i, s) in r.screenshots.iter().enumerate() {
        let key = format!("screenshots.{i}");
        issue(
            format!("{key}.asset"),
            !screenshot_asset(&s.asset) || !screenshot_assets.insert(&s.asset),
            "invalid or duplicate screenshot asset",
            "use a unique relative path like <id>/01.jpg with a .jpg or .jpeg extension (JPEG only)",
        );
        issue(
            format!("{key}.credit"),
            s.credit.trim().is_empty(),
            "missing screenshot credit",
            "credit the mapset authors or who took the screenshot",
        );
    }
    let mut variants = HashSet::new();
    for v in &r.base_game.variants {
        issue(
            "base_game.variants".into(),
            v.is_empty()
                || !v.as_bytes()[0].is_ascii_alphanumeric()
                || !v.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.'
                })
                || !variants.insert(v),
            "invalid or duplicate IWAD variant",
            "use a known IWAD id (lowercase letters, digits, dots and hyphens)",
        );
    }
    issue(
        "packages".into(),
        r.packages.is_empty(),
        "no packages",
        "declare at least one source package",
    );
    let mut package_ids = HashSet::new();
    for (i, p) in r.packages.iter().enumerate() {
        let key = format!("packages.{i}");
        issue(
            format!("{key}.id"),
            !slug(&p.id) || !package_ids.insert(&p.id),
            "invalid or duplicate package id",
            "use unique lowercase slugs",
        );
        issue(
            format!("{key}.page"),
            checked_url(&p.page, &hosts.pages).is_err(),
            "invalid package page URL",
            "use HTTPS and an allowlisted page host",
        );
        issue(
            format!("{key}.urls"),
            p.urls.is_empty()
                || p.urls
                    .iter()
                    .any(|u| checked_url(u, &hosts.downloads).is_err()),
            "invalid download URLs",
            "list direct HTTPS URLs on download hosts in hosts.toml",
        );
        issue(
            format!("{key}.size"),
            p.size == 0 || p.size > MAX_BYTES,
            "invalid package size",
            "use an exact positive byte count at most 512 MiB",
        );
        issue(
            format!("{key}.sha256"),
            !hash(&p.sha256),
            "malformed package SHA-256",
            "use exactly 64 hexadecimal digits from reviewed bytes",
        );
        issue(
            format!("{key}.idgames_path"),
            p.idgames_path
                .as_ref()
                .is_some_and(|p| !safe_path(p) || p.contains("!/")),
            "unsafe idgames path",
            "use a canonical relative archive path",
        );
        issue(
            format!("{key}.members"),
            p.members.is_empty(),
            "no named members",
            "list exactly the files DDL needs, including permission text",
        );
        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        for (j, m) in p.members.iter().enumerate() {
            let mk = format!("{key}.members.{j}");
            issue(
                format!("{mk}.id"),
                !slug(&m.id) || !ids.insert(&m.id),
                "invalid or duplicate member id",
                "use unique lowercase slugs within each package",
            );
            issue(
                format!("{mk}.path"),
                !safe_path(&m.path) || !paths.insert(&m.path),
                "unsafe or duplicate member path",
                "use an exact relative case-sensitive path; nested ZIPs use !/",
            );
            issue(
                format!("{mk}.size"),
                m.size == 0 || m.size > MAX_BYTES,
                "invalid member size",
                "use an exact positive byte count at most 512 MiB",
            );
            issue(
                format!("{mk}.sha256"),
                !hash(&m.sha256),
                "malformed member SHA-256",
                "use exactly 64 hexadecimal digits",
            );
            issue(
                format!("{mk}.path"),
                m.path
                    .split("!/")
                    .collect::<Vec<_>>()
                    .split_last()
                    .is_some_and(|(_, parents)| {
                        parents
                            .iter()
                            .any(|p| !p.to_ascii_lowercase().ends_with(".zip"))
                    }),
                "nested member crosses a non-ZIP",
                "only ZIP containers may precede !/",
            );
        }
        if p.kind == PackageKind::Wad {
            issue(
                format!("{key}.members"),
                p.members.len() != 1
                    || p.members.iter().any(|m| {
                        m.size != p.size
                            || !m.sha256.eq_ignore_ascii_case(&p.sha256)
                            || !m.path.to_ascii_lowercase().ends_with(".wad")
                            || m.path.contains("!/")
                    }),
                "bare WAD member disagrees with package",
                "declare one .wad member with the package size/hash",
            );
        }
    }
    let main = r.components.iter().find(|c| c.id == "main");
    issue(
        "components".into(),
        main.is_none(),
        "missing main component",
        "declare one required main component",
    );
    let mut component_ids = HashSet::new();
    for (i, c) in r.components.iter().enumerate() {
        let key = format!("components.{i}");
        issue(
            format!("{key}.id"),
            !slug(&c.id) || !component_ids.insert(&c.id),
            "invalid or duplicate component id",
            "use unique lowercase slugs",
        );
        issue(
            format!("{key}.label"),
            c.label.trim().is_empty() || c.description.trim().is_empty(),
            "missing component text",
            "supply label and description",
        );
        issue(
            format!("{key}.optional"),
            if c.id == "main" {
                c.optional || !c.default || c.replaces.is_some()
            } else {
                !c.optional
            },
            "invalid required/optional component",
            "main must be required and default-on; all others optional",
        );
        issue(
            format!("{key}.position"),
            c.position == 0,
            "invalid load position",
            "use a positive explicit position",
        );
        issue(
            format!("{key}.replaces"),
            c.replaces.as_ref().is_some_and(|v| v != "main"),
            "invalid replacement target",
            "only replaces = \"main\" is supported",
        );
        issue(
            format!("{key}.exclusive_group"),
            c.exclusive_group.as_ref().is_some_and(|v| !slug(v)),
            "invalid exclusive group",
            "use a lowercase slug",
        );
        issue(
            format!("{key}.members"),
            c.members.is_empty(),
            "component has no members",
            "name each loaded member and its role",
        );
        issue(
            format!("{key}.obtain"),
            if c.user_supplied {
                c.obtain.as_ref().is_none_or(|s| s.trim().is_empty()) || c.members.len() != 1
            } else {
                c.obtain.is_some() || !c.known_sha256.is_empty()
            },
            "invalid user-supplied source",
            "imports require obtain instructions and one member; downloads use package references",
        );
        issue(
            format!("{key}.known_sha256"),
            c.known_sha256.iter().any(|h| !hash(h)),
            "malformed known SHA-256",
            "use reviewed 64-digit hashes or an empty list when unknown",
        );
        let mut refs = HashSet::new();
        for (j, cm) in c.members.iter().enumerate() {
            let mk = format!("{key}.members.{j}");
            if c.user_supplied {
                issue(
                    mk,
                    cm.package.is_some() || cm.member.is_some(),
                    "user-supplied component references a download",
                    "omit package/member; the player imports this file",
                );
                continue;
            }
            let member = r
                .packages
                .iter()
                .find(|p| Some(&p.id) == cm.package.as_ref())
                .and_then(|p| p.members.iter().find(|m| Some(&m.id) == cm.member.as_ref()));
            issue(
                mk.clone(),
                member.is_none(),
                "unknown package/member reference",
                "reference a declared package id and member id",
            );
            issue(
                mk.clone(),
                !refs.insert((&cm.package, &cm.member)),
                "duplicate component member reference",
                "list each member once per component",
            );
            if let Some(m) = member {
                let ext = m.path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
                issue(
                    format!("{mk}.role"),
                    m.keep
                        || if cm.role == Role::Dehacked {
                            !["deh", "bex"].contains(&ext.as_str())
                        } else {
                            ext != "wad"
                        },
                    "file role inconsistent with member",
                    "load WADs as maps/music/graphics/sounds, DEH/BEX as dehacked; never load keep members",
                );
            }
        }
        if c.id != "main" && c.replaces.is_none() {
            issue(
                format!("{key}.position"),
                main.is_some_and(|m| c.position <= m.position),
                "add-on must load after main",
                "give the add-on a position greater than main",
            );
        }
        for other in r.components.iter().take(i) {
            let exclusive =
                c.exclusive_group.is_some() && c.exclusive_group == other.exclusive_group;
            let replacement = (c.replaces.is_some()
                && (other.id == "main" || other.replaces.is_some()))
                || (other.replaces.is_some() && c.id == "main");
            issue(
                format!("{key}.position"),
                !exclusive && !replacement && c.position == other.position,
                "ambiguous load order for an allowed combination",
                "assign distinct positions or declare exclusivity",
            );
            let shared = c.members.iter().any(|a| {
                a.package.is_some()
                    && other
                        .members
                        .iter()
                        .any(|b| a.package == b.package && a.member == b.member)
            });
            issue(
                format!("{key}.members"),
                shared && !exclusive && !replacement,
                "member loaded twice in an allowed combination",
                "remove the duplicate or make the components exclusive",
            );
        }
    }
    let defaults: Vec<_> = r
        .components
        .iter()
        .filter(|c| c.optional && c.default)
        .map(|c| c.id.as_str())
        .collect();
    issue(
        "components".into(),
        r.load_order(&defaults).is_err(),
        "invalid default component combination",
        "default-enable at most one exclusive choice or replacement",
    );
    issue(
        "compatibility.complevel".into(),
        matches!(r.compatibility.complevel, Complevel::Number(n) if n > 17 && n != 21),
        "unreviewed complevel",
        "use vanilla, 0–17, 21, boom, mbf or mbf21 (never 18–20)",
    );
    // D-030: a recipe names the level with DSDA's alias whenever one does; a
    // number stays only for a level no alias names on this recipe's IWAD.
    let named = match r.compatibility.complevel {
        Complevel::Number(n) => ComplevelAlias::naming(n, r.base_game.game).map(|alias| (n, alias)),
        Complevel::Alias(_) => None,
    };
    issue(
        "compatibility.complevel".into(),
        named.is_some(),
        &named.map_or_else(String::new, |(n, alias)| {
            format!(
                "complevel {n} is what the alias `{}` means on {}",
                alias.as_str(),
                game_name(r.base_game.game)
            )
        }),
        &named.map_or_else(String::new, |(n, alias)| {
            format!("write complevel = \"{}\" instead of {n}", alias.as_str())
        }),
    );
    let range = semver::VersionReq::parse(&r.compatibility.engine.range);
    let version = semver::Version::parse(&r.verification.engine_version);
    issue(
        "compatibility.engine.range".into(),
        range.is_err() || matches!((&range, &version), (Ok(r), Ok(v)) if !r.matches(v)),
        "invalid engine version range",
        "use a semver requirement containing the evidence engine version",
    );
    issue(
        "verification.engine_version".into(),
        version.is_err(),
        "invalid engine version",
        "record the exact inspected/tested semantic version",
    );
    issue(
        "compatibility.runtime.warp".into(),
        r.compatibility
            .runtime
            .warp
            .as_ref()
            .is_some_and(|w| w.len() > 2 || w.iter().any(|n| *n > 99)),
        "invalid warp",
        "use zero to two integers in 0–99, per DSDA v0.30.1",
    );
    issue(
        "verification.date".into(),
        !valid_date(&r.verification.date),
        "invalid verification date",
        "use a real YYYY-MM-DD date",
    );
    issue(
        "verification.tester".into(),
        r.verification.tester.trim().is_empty() || r.verification.notes.trim().is_empty(),
        "missing verification evidence",
        "record tester and nonempty notes without claiming unperformed tests",
    );
    issue(
        "verification.inspected".into(),
        r.verification.status == VerificationStatus::Inspected
            && r.verification.inspected.is_empty(),
        "missing inspection evidence",
        "record inspected archives, lumps and source documents",
    );
    issue(
        "compatibility.engine.status".into(),
        r.verification.status == VerificationStatus::Inspected
            && r.compatibility.engine.status != VerificationStatus::Inspected,
        "inspection mislabeled as engine test",
        "mark the engine range inspected when only desk research exists",
    );
    errors
}

/// True when `text` links a year page of the Doomworld Cacowards, e.g.
/// `https://www.doomworld.com/cacowards/2019/`.
fn cites_cacowards_year(text: &str) -> bool {
    const PAGE: &str = "doomworld.com/cacowards/";
    text.match_indices(PAGE).any(|(at, _)| {
        let year = &text[at + PAGE.len()..];
        year.len() >= 4 && year.as_bytes()[..4].iter().all(u8::is_ascii_digit)
    })
}

/// Tag rules for one recipe (spec 015, R2): every tag is in the vocabulary,
/// none repeats, and `cacowards` carries a citation of the year's Doomworld
/// page in the verification notes or `inspected` list. Offline and pure.
pub fn validate_tags(r: &Recipe, vocabulary: &TagVocabulary) -> Vec<Violation> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (i, tag) in r.tags.iter().enumerate() {
        let key = format!("tags.{i}");
        if vocabulary.get(tag).is_none() {
            errors.push(Violation {
                key: key.clone(),
                message: format!("unknown tag `{tag}`"),
                hint: "use an id from catalog/tags.toml, or add the tag there in the same pull request"
                    .into(),
            });
        }
        if !seen.insert(tag) {
            errors.push(Violation {
                key,
                message: format!("duplicate tag `{tag}`"),
                hint: "list each tag once".into(),
            });
        }
    }
    if r.tags.iter().any(|t| t == "cacowards")
        && !std::iter::once(&r.verification.notes)
            .chain(&r.verification.inspected)
            .any(|text| cites_cacowards_year(text))
    {
        errors.push(Violation {
            key: "tags".into(),
            message: "`cacowards` tag without a Cacowards citation".into(),
            hint: "cite that year's page, e.g. https://www.doomworld.com/cacowards/2019/, in verification.notes or verification.inspected"
                .into(),
        });
    }
    errors
}

/// Rules for `catalog/tags.toml` itself (spec 015, R1).
pub fn validate_vocabulary(v: &TagVocabulary) -> Vec<Violation> {
    let mut errors = Vec::new();
    let mut issue = |key: String, message: &str, hint: &str| {
        errors.push(Violation {
            key,
            message: message.into(),
            hint: hint.into(),
        });
    };
    if v.schema_version != 1 {
        issue(
            "schema_version".into(),
            "unsupported schema_version",
            "use schema_version = 1",
        );
    }
    let mut ids = HashSet::new();
    let mut orders = HashSet::new();
    for (i, tag) in v.tags.iter().enumerate() {
        let key = format!("tags.{i}");
        if !slug(&tag.id) || !ids.insert(&tag.id) {
            issue(
                format!("{key}.id"),
                "invalid or duplicate tag id",
                "use unique lowercase alphanumeric/hyphen slugs",
            );
        }
        if tag.label.trim().is_empty() || tag.description.trim().is_empty() {
            issue(
                format!("{key}.label"),
                "missing tag text",
                "supply an English label and a short description",
            );
        }
        if !orders.insert(tag.order) {
            issue(
                format!("{key}.order"),
                "duplicate tag order",
                "give every tag its own position",
            );
        }
    }
    errors
}

fn game_name(game: Game) -> &'static str {
    match game {
        Game::Doom => "doom",
        Game::Doom2 => "doom2",
        Game::Tnt => "tnt",
        Game::Plutonia => "plutonia",
    }
}

/// The message and hint for a recipe that fails to parse. A key the schema
/// dropped gets a precise explanation instead of serde's generic one.
pub(crate) fn parse_failure(message: &str) -> (String, &'static str) {
    if message.contains("unknown field `complevel_status`") {
        return (
            "`complevel_status` is no longer part of the recipe schema (D-030)".into(),
            "delete the `complevel_status` line; a complevel has no verification status",
        );
    }
    (
        message.to_owned(),
        "fix the schema/type; remove unknown keys and supply required fields",
    )
}

fn diag(file: &Path, line: usize, message: impl ToString, hint: &str) -> Diagnostic {
    Diagnostic {
        file: file.into(),
        line,
        message: message.to_string(),
        hint: hint.into(),
    }
}

/// Bounded reads keep even local/untrusted catalog input from exhausting memory.
fn read_text(path: &Path) -> Result<String, Diagnostic> {
    use std::io::Read;
    let file = fs::File::open(path)
        .map_err(|e| diag(path, 1, e, "check the file path and permissions"))?;
    let mut text = String::new();
    file.take(MAX_RECIPE_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|e| diag(path, 1, e, "use UTF-8 TOML"))?;
    if text.len() as u64 > MAX_RECIPE_BYTES {
        return Err(diag(
            path,
            1,
            "catalog document exceeds 1 MiB",
            "reduce the document size",
        ));
    }
    Ok(text)
}

fn parse<T: serde::de::DeserializeOwned>(path: &Path, text: &str) -> Result<T, Diagnostic> {
    toml::from_str(text).map_err(|e: toml::de::Error| {
        let line = e.span().map_or(1, |s| {
            text[..s.start].bytes().filter(|b| *b == b'\n').count() + 1
        });
        let (message, hint) = parse_failure(e.message());
        diag(path, line, message, hint)
    })
}

/// Semantic diagnostics use TOML's own spanned generic deserialization, so
/// repeated keys in different array tables still point to the correct value.
enum Located {
    Table(std::collections::BTreeMap<String, toml::Spanned<Located>>),
    Array(Vec<toml::Spanned<Located>>),
    Scalar,
}
impl<'de> serde::Deserialize<'de> for Located {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Located;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a TOML value")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Located, M::Error> {
                let mut values = std::collections::BTreeMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    values.insert(key, value);
                }
                Ok(Located::Table(values))
            }
            fn visit_seq<S: serde::de::SeqAccess<'de>>(
                self,
                mut seq: S,
            ) -> Result<Located, S::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(Located::Array(values))
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Located, E> {
                Ok(Located::Scalar)
            }
            fn visit_string<E: serde::de::Error>(self, _: String) -> Result<Located, E> {
                Ok(Located::Scalar)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<Located, E> {
                Ok(Located::Scalar)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Located, E> {
                Ok(Located::Scalar)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Located, E> {
                Ok(Located::Scalar)
            }
            fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<Located, E> {
                Ok(Located::Scalar)
            }
        }
        de.deserialize_any(Visitor)
    }
}
fn line_for(text: &str, key: &str) -> usize {
    let Ok(root) = toml::from_str::<Located>(text) else {
        return 1;
    };
    let mut current = &root;
    let mut offset = 0;
    for part in key.split('.') {
        let value = match current {
            Located::Table(t) => t.get(part),
            Located::Array(a) => part.parse::<usize>().ok().and_then(|i| a.get(i)),
            Located::Scalar => None,
        };
        let Some(value) = value else {
            break;
        };
        offset = value.span().start;
        current = value.get_ref();
    }
    text[..offset].bytes().filter(|b| *b == b'\n').count() + 1
}

/// The member paths in `package` that a widescreen-status-bar component loads,
/// so `--fetch` can check their WAD directory for non-cosmetic lumps (R4).
fn cosmetic_member_paths<'a>(r: &'a Recipe, package: &'a Package) -> Vec<&'a str> {
    r.components
        .iter()
        .filter(|c| is_status_bar_component(c))
        .flat_map(|c| c.members.iter())
        .filter(|m| m.role == Role::Graphics && m.package.as_deref() == Some(package.id.as_str()))
        .filter_map(|m| m.member.as_deref())
        .filter_map(|id| package.members.iter().find(|member| member.id == id))
        .map(|member| member.path.as_str())
        .collect()
}

/// Checks each referenced screenshot exists on disk, is a JPEG within the byte
/// and 1280×800 dimension bounds, and has a `SOURCES.toml` entry with a credit
/// and source URL (spec 023, R1/R2; spec 035, R1). Offline and filesystem-bound: used only by
/// `lint_paths`, which has real paths, never by the embedded loader.
fn validate_screenshots(r: &Recipe, recipe_path: &Path) -> Vec<Violation> {
    use super::image::{
        ImageFormat, MAX_SCREENSHOT_BYTES, MAX_SCREENSHOT_HEIGHT, MAX_SCREENSHOT_WIDTH,
        MIN_SCREENSHOT_HEIGHT, MIN_SCREENSHOT_WIDTH, image_info,
    };
    if r.screenshots.is_empty() {
        return Vec::new();
    }
    // `<catalog>/<id>.toml` -> repository root -> screenshots directory.
    let root = recipe_path
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
        .join(SCREENSHOTS_DIR);
    // A synthetic or partial tree (unit tests, a catalog fragment) has no
    // screenshots directory; the structural rules still apply, but there is
    // nothing on disk to inspect. The shipped catalog always has it, so CI
    // checks every committed image.
    if !root.is_dir() {
        return Vec::new();
    }
    let mut errors = Vec::new();
    for (i, s) in r.screenshots.iter().enumerate() {
        let key = format!("screenshots.{i}");
        let full = root.join(&s.asset);
        let bytes = match read_screenshot(&full) {
            Ok(bytes) => bytes,
            Err(reason) => {
                errors.push(Violation {
                    key: format!("{key}.asset"),
                    message: reason,
                    hint: format!(
                        "commit the image under {SCREENSHOTS_DIR}/{id}",
                        id = s.asset.rsplit_once('/').map_or("", |(dir, _)| dir)
                    ),
                });
                continue;
            }
        };
        let mut asset_issue = |message: String, hint: &str| {
            errors.push(Violation {
                key: format!("{key}.asset"),
                message,
                hint: hint.into(),
            });
        };
        if bytes.len() as u64 > MAX_SCREENSHOT_BYTES {
            asset_issue(
                format!(
                    "screenshot `{}` is {} bytes, over the {} byte limit",
                    s.asset,
                    bytes.len(),
                    MAX_SCREENSHOT_BYTES
                ),
                "re-save as JPEG at quality 60 and at most 1280x800: scripts/lean-image.sh (spec 035, R1)",
            );
        }
        match image_info(&bytes) {
            Ok((format, width, height)) => {
                if format != ImageFormat::Jpeg {
                    asset_issue(
                        format!(
                            "screenshot `{}` is a {format} image; bundled screenshots must be JPEG",
                            s.asset
                        ),
                        "convert it with scripts/lean-image.sh (spec 035, R1)",
                    );
                }
                if width < MIN_SCREENSHOT_WIDTH || height < MIN_SCREENSHOT_HEIGHT {
                    asset_issue(
                        format!(
                            "screenshot `{}` is {width}x{height}, smaller than {MIN_SCREENSHOT_WIDTH}x{MIN_SCREENSHOT_HEIGHT}",
                            s.asset
                        ),
                        "use a card-sized image at least 640x360",
                    );
                } else if width > MAX_SCREENSHOT_WIDTH || height > MAX_SCREENSHOT_HEIGHT {
                    asset_issue(
                        format!(
                            "screenshot `{}` is {width}x{height}, larger than {MAX_SCREENSHOT_WIDTH}x{MAX_SCREENSHOT_HEIGHT}",
                            s.asset
                        ),
                        "shrink it to at most 1280x800: scripts/lean-image.sh (spec 035, R1)",
                    );
                }
            }
            Err(error) => asset_issue(
                format!("screenshot `{}`: {error}", s.asset),
                "commit a JPEG image (spec 035, R1)",
            ),
        }
        match sources_entry(&full) {
            Some(source) if !source.credit.trim().is_empty() && !source.source_url.trim().is_empty() => {}
            _ => errors.push(Violation {
                key: format!("{key}.asset"),
                message: format!(
                    "screenshot `{}` has no SOURCES.toml entry with a credit and source URL",
                    s.asset
                ),
                hint: format!(
                    "add a [[screenshot]] entry to {SCREENSHOTS_DIR}/{}/SOURCES.toml (spec 023, R1)",
                    s.asset.rsplit_once('/').map_or("", |(dir, _)| dir)
                ),
            }),
        }
    }
    errors
}

/// Reads at most one byte more than the screenshot cap, so an oversized file is
/// detected without reading it whole.
fn read_screenshot(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let file = fs::File::open(path).map_err(|error| format!("missing screenshot: {error}"))?;
    let mut bytes = Vec::new();
    file.take(super::image::MAX_SCREENSHOT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("unreadable screenshot: {error}"))?;
    Ok(bytes)
}

/// The `SOURCES.toml` entry matching `asset`'s file name, when present.
fn sources_entry(asset_path: &Path) -> Option<ScreenshotSource> {
    let dir = asset_path.parent()?;
    let file = asset_path.file_name()?.to_str()?;
    let text = read_text(&dir.join("SOURCES.toml")).ok()?;
    let sources: ScreenshotSources = toml::from_str(&text).ok()?;
    if sources.schema_version != 1 {
        return None;
    }
    sources
        .screenshot
        .into_iter()
        .find(|entry| entry.file == file)
}

/// Lints directories or individual recipes. Hosts are supplied explicitly so
/// callers/tests need not depend on cwd. Network occurs only with `Some(fetch)`.
pub fn lint_paths(
    paths: &[PathBuf],
    hosts_path: &Path,
    mut fetch: Option<&mut dyn Fetch>,
) -> Vec<Diagnostic> {
    let hosts_text = match read_text(hosts_path) {
        Ok(t) => t,
        Err(e) => return vec![e],
    };
    let hosts: Hosts = match parse(hosts_path, &hosts_text) {
        Ok(h) => h,
        Err(e) => return vec![e],
    };
    if hosts.schema_version != 1
        || hosts.pages.is_empty()
        || hosts.downloads.is_empty()
        || hosts.pages.iter().chain(&hosts.downloads).any(|h| {
            Url::parse(&format!("https://{h}/"))
                .ok()
                .is_none_or(|u| u.host_str() != Some(h.as_str()) || u.path() != "/")
        })
    {
        return vec![diag(
            hosts_path,
            1,
            "invalid host allowlist",
            "use schema 1 and exact lowercase hostnames",
        )];
    }
    let mut files = Vec::new();
    let mut errors = Vec::new();
    // The vocabulary sits beside hosts.toml; a missing file is an empty one, so
    // a catalog without tags still lints and a tagged recipe reports unknown tags.
    let tags_path = hosts_path.with_file_name("tags.toml");
    let vocabulary = match read_text(&tags_path) {
        Ok(text) => match parse::<TagVocabulary>(&tags_path, &text) {
            Ok(vocabulary) => {
                for v in validate_vocabulary(&vocabulary) {
                    errors.push(diag(
                        &tags_path,
                        line_for(&text, &v.key),
                        v.message,
                        &v.hint,
                    ));
                }
                vocabulary
            }
            Err(e) => {
                errors.push(e);
                TagVocabulary::default()
            }
        },
        Err(_) => TagVocabulary::default(),
    };
    for path in paths {
        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(e)
                                if e.path().extension().is_some_and(|x| x == "toml")
                                    && e.path()
                                        .file_name()
                                        .is_none_or(|n| n != "hosts.toml" && n != "tags.toml") =>
                            {
                                files.push(e.path())
                            }
                            Ok(_) => {}
                            Err(e) => errors.push(diag(path, 1, e, "check directory permissions")),
                        }
                    }
                }
                Err(e) => errors.push(diag(path, 1, e, "check the catalog directory")),
            }
        } else {
            files.push(path.clone());
        }
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        errors.push(diag(
            paths.first().map_or(Path::new("catalog"), |p| p.as_path()),
            1,
            "no recipe files found",
            "provide catalog/ or explicit .toml files",
        ));
    }
    let mut ids = HashSet::new();
    for path in files {
        let text = match read_text(&path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let r: Recipe = match parse(&path, &text) {
            Ok(r) => r,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        let mut violations = validate(&r, &path, &hosts);
        violations.extend(validate_tags(&r, &vocabulary));
        violations.extend(validate_screenshots(&r, &path));
        let valid = violations.is_empty();
        for v in violations {
            errors.push(diag(&path, line_for(&text, &v.key), v.message, &v.hint));
        }
        if !ids.insert(r.id.clone()) {
            errors.push(diag(
                &path,
                line_for(&text, "id"),
                "duplicate recipe id",
                "use one file per stable recipe id",
            ));
        }
        if valid && let Some(fetcher) = fetch.as_deref_mut() {
            for (i, p) in r.packages.iter().enumerate() {
                let cosmetic = cosmetic_member_paths(&r, p);
                if let Err(e) = fetch::fetch_package(p, &hosts, fetcher, &cosmetic) {
                    errors.push(diag(&path, line_for(&text, &format!("packages.{i}")), e, "check upstream bytes, size, SHA-256 and exact member paths; never guess new hashes"));
                }
            }
        }
    }
    errors
}
