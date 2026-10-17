use super::*;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

const CATALOG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog");
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/catalog");
fn hosts() -> Hosts {
    toml::from_str(&fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap()).unwrap()
}
fn recipe(id: &str) -> Recipe {
    toml::from_str(&fs::read_to_string(format!("{CATALOG}/{id}.toml")).unwrap()).unwrap()
}
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn reference_recipes_round_trip_and_lint_offline() {
    for id in [
        "ancient-aliens",
        "auger-zenith",
        "eviternity",
        "sigil",
        "sigil-ii",
        "sunlust",
    ] {
        let r = recipe(id);
        assert_eq!(
            toml::from_str::<Recipe>(&toml::to_string_pretty(&r).unwrap()).unwrap(),
            r
        );
        assert_eq!(
            validate(&r, Path::new(&format!("{id}.toml")), &hosts()),
            vec![]
        );
        assert_eq!(r.verification.status, VerificationStatus::Inspected);
        assert_eq!(r.compatibility.complevel_status, EvidenceStatus::Proposed);
    }
    assert!(
        lint_paths(
            &[CATALOG.into()],
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )
        .is_empty()
    );
}

#[test]
fn reference_combinations_have_explicit_order_and_replacements() {
    let auger = recipe("auger-zenith");
    assert_eq!(auger.load_order(&[]).unwrap().files.len(), 1);
    for addon in ["alternate-blaster", "deckard-blaster"] {
        let order = auger.load_order(&[addon]).unwrap();
        assert_eq!(order.files.len(), 2);
        assert_eq!(order.files[0].member.as_deref(), Some("maps"));
        assert!(order.deh.is_empty()); // WAD-embedded DEHACKED is automatic.
    }
    assert!(
        auger
            .load_order(&["alternate-blaster", "deckard-blaster"])
            .unwrap_err()
            .contains("group")
    );
    for id in ["sigil", "sigil-ii"] {
        let r = recipe(id);
        let order = r.load_order(&["soundtrack"]).unwrap();
        assert_eq!(order.files.len(), 1);
        assert!(order.files[0].package.is_none());
        assert_eq!(r.base_game.variants, ["doom-1.9ud"]);
    }
    let sigil = recipe("sigil");
    assert_eq!(
        sigil.load_order(&["compat"]).unwrap().files[0]
            .member
            .as_deref(),
        Some("compat")
    );
    assert!(sigil.load_order(&["compat", "soundtrack"]).is_err());
    assert!(sigil.load_order(&["unknown"]).is_err());
    assert!(sigil.load_order(&["compat", "compat"]).is_err());
}

#[test]
fn every_semantic_rule_rejects_a_bad_recipe() {
    type Mutation = fn(&mut Recipe);
    let cases: &[(&str, Mutation)] = &[
        ("schema_version", |r| r.schema_version = 2),
        ("invalid recipe id", |r| r.id = "SIGIL".into()),
        ("file name", |r| r.id = "different".into()),
        ("identity/permission", |r| r.permission.clear()),
        ("missing authors", |r| r.authors.clear()),
        ("canonical page", |r| {
            r.page = "http://romero.com/sigil".into()
        }),
        ("unsafe image", |r| {
            r.images.push(Image {
                path: "../x.png".into(),
                permission: "yes".into(),
            })
        }),
        ("image permission", |r| {
            r.images.push(Image {
                path: "x.png".into(),
                permission: "".into(),
            })
        }),
        ("IWAD variant", |r| {
            r.base_game.variants = vec!["bad/name".into()]
        }),
        ("IWAD variant", |r| {
            r.base_game.variants.push(r.base_game.variants[0].clone())
        }),
        ("no packages", |r| r.packages.clear()),
        ("duplicate package", |r| {
            r.packages.push(r.packages[0].clone())
        }),
        ("package page", |r| {
            r.packages[0].page = "https://unlisted.invalid".into()
        }),
        ("download URLs", |r| r.packages[0].urls.clear()),
        ("download URLs", |r| {
            r.packages[0].urls = vec!["https://www.doomworld.com/file.zip".into()]
        }),
        ("package size", |r| r.packages[0].size = 0),
        ("package size", |r| r.packages[0].size = u64::MAX),
        ("package SHA", |r| r.packages[0].sha256 = "0".repeat(63)),
        ("idgames path", |r| {
            r.packages[0].idgames_path = Some("../x".into())
        }),
        ("no named members", |r| r.packages[0].members.clear()),
        ("duplicate member id", |r| {
            let m = r.packages[0].members[0].clone();
            r.packages[0].members.push(m);
        }),
        ("duplicate member path", |r| {
            r.packages[0].members[0].path = "../evil.wad".into()
        }),
        ("member size", |r| r.packages[0].members[0].size = 0),
        ("member SHA", |r| {
            r.packages[0].members[0].sha256 = "g".repeat(64)
        }),
        ("non-ZIP", |r| {
            r.packages[0].members[0].path = "x.wad!/x.wad".into()
        }),
        ("bare WAD", |r| r.packages[0].kind = PackageKind::Wad),
        ("missing main", |r| r.components[0].id = "other".into()),
        ("duplicate component", |r| {
            r.components.push(r.components[0].clone())
        }),
        ("component text", |r| r.components[0].label.clear()),
        ("required/optional", |r| r.components[0].optional = true),
        ("load position", |r| r.components[0].position = 0),
        ("replacement target", |r| {
            r.components[1].replaces = Some("other".into())
        }),
        ("exclusive group", |r| {
            r.components[1].exclusive_group = Some("?".into())
        }),
        ("no members", |r| r.components[0].members.clear()),
        ("user-supplied source", |r| r.components[2].obtain = None),
        ("known SHA", |r| {
            r.components[2].known_sha256 = vec!["bad".into()]
        }),
        ("references a download", |r| {
            r.components[2].members[0].package = Some("free".into())
        }),
        ("unknown package/member", |r| {
            r.components[0].members[0].member = Some("missing".into())
        }),
        ("duplicate component member", |r| {
            let m = r.components[0].members[0].clone();
            r.components[0].members.push(m);
        }),
        ("role inconsistent", |r| {
            r.components[0].members[0].role = Role::Dehacked
        }),
        ("role inconsistent", |r| {
            r.components[0].members[0].member = Some("credits".into())
        }),
        ("add-on must load after", |r| {
            r.components[1].replaces = None;
            r.components[1].position = 1;
        }),
        ("ambiguous load order", |r| {
            r.components[1].replaces = None;
            r.components[1].position = 2;
            r.components[2].replaces = None;
            r.components[2].position = 2;
            r.components[2].exclusive_group = None;
        }),
        ("loaded twice", |r| {
            r.components[1].replaces = None;
            r.components[1].position = 2;
            r.components[1].members[0].member = Some("maps".into());
        }),
        ("default component", |r| {
            r.components[1].default = true;
            r.components[2].default = true;
        }),
        ("unreviewed complevel", |r| {
            r.compatibility.complevel = Complevel::Number(18)
        }),
        ("engine version range", |r| {
            r.compatibility.engine.range = "invalid".into()
        }),
        ("engine version range", |r| {
            r.compatibility.engine.range = "<0.30.1".into()
        }),
        ("invalid engine version", |r| {
            r.verification.engine_version = "not a version".into()
        }),
        ("invalid warp", |r| {
            r.compatibility.runtime.warp = Some(vec![100])
        }),
        ("verification date", |r| {
            r.verification.date = "2026-02-30".into()
        }),
        ("verification evidence", |r| r.verification.tester.clear()),
        ("inspection evidence", |r| r.verification.inspected.clear()),
        ("inspection mislabeled", |r| {
            r.compatibility.engine.status = VerificationStatus::Verified
        }),
    ];
    for (message, mutation) in cases {
        let mut r = recipe("sigil");
        mutation(&mut r);
        let errors = validate(&r, Path::new("sigil.toml"), &hosts());
        assert!(
            errors.iter().any(|e| e.message.contains(message)),
            "expected {message}, got {errors:?}"
        );
    }
}

#[test]
fn schema_rejects_unknown_fields_at_every_level_and_wrong_enums() {
    let source = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    for header in [
        "",
        "[base_game]",
        "[compatibility]",
        "[compatibility.engine]",
        "[compatibility.runtime]",
        "[verification]",
        "[[packages]]",
        "[[packages.members]]",
        "[[components]]",
        "[[components.members]]",
    ] {
        let text = if header.is_empty() {
            format!("unknown = true\n{source}")
        } else {
            source.replacen(header, &format!("{header}\nunknown = true"), 1)
        };
        assert!(
            toml::from_str::<Recipe>(&text)
                .unwrap_err()
                .message()
                .contains("unknown field"),
            "{header}"
        );
    }
    for (old, new) in [
        ("game = \"doom\"", "game = \"shareware\""),
        ("family = \"limit-removing\"", "family = \"gzdoom\""),
        ("status = \"inspected\"", "status = \"untested\""),
        ("kind = \"zip\"", "kind = \"exe\""),
        ("role = \"maps\"", "role = \"script\""),
        ("complevel = 3", "complevel = \"auto\""),
    ] {
        assert!(toml::from_str::<Recipe>(&source.replacen(old, new, 1)).is_err());
    }
}

#[test]
fn reviewed_complevels_runtime_defaults_and_dates() {
    let mut r = recipe("sigil");
    for n in 0..=21 {
        r.compatibility.complevel = Complevel::Number(n);
        assert_eq!(
            validate(&r, Path::new("sigil.toml"), &hosts()).is_empty(),
            n <= 17 || n == 21
        );
    }
    for alias in [
        ComplevelAlias::Vanilla,
        ComplevelAlias::Boom,
        ComplevelAlias::Mbf,
        ComplevelAlias::Mbf21,
    ] {
        r.compatibility.complevel = Complevel::Alias(alias);
        r.compatibility.runtime.warp = Some(vec![5, 1]);
        r.compatibility.runtime.pistol_start = Some(true);
        r.verification.date = "2024-02-29".into();
        assert!(validate(&r, Path::new("sigil.toml"), &hosts()).is_empty());
    }
}

#[test]
fn https_hosts_paths_and_hashes_are_strict() {
    for url in [
        "http://romero.com/x",
        "https://romero.com.evil/x",
        "https://evil@romero.com/x",
        "https://romero.com:444/x",
        "https://romero.com/x#fragment",
        "file:///etc/passwd",
        "https://www.doomworld.com/x",
    ] {
        assert!(lint::checked_url(url, &hosts().downloads).is_err(), "{url}");
    }
    for path in [
        "/x",
        "../x",
        "x/../y",
        "C:/x",
        "x\\y",
        "x//y",
        "./x",
        "x.zip!/../y",
        "x.zip!/y.zip!/z.zip!/a.zip!/b",
        "-x.wad",
    ] {
        assert!(!lint::safe_path(path), "{path}");
    }
    assert!(lint::safe_path(
        "DBP37_Addons.zip!/DBP37_AlternateBlaster.wad"
    ));
    assert!(lint::hash(&"A".repeat(64)));
}

#[test]
fn failing_fixtures_have_precise_lines_messages_and_hints() {
    for (folder, message, line) in [
        ("invalid-hash", "malformed package SHA-256", 42),
        ("unknown-key", "unknown field", 1),
    ] {
        let errors = lint_paths(
            &[Path::new(FIXTURES).join(folder).join("sigil.toml")],
            &Path::new(CATALOG).join("hosts.toml"),
            None,
        );
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains(message), "{errors:?}");
        assert_eq!(errors[0].line, line, "{errors:?}");
        assert!(!errors[0].hint.is_empty());
        assert!(
            errors[0]
                .to_string()
                .contains(&format!("sigil.toml:{line}:"))
        );
    }
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ddl-catalog-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn lint_reports_duplicate_recipe_ids_io_empty_and_bad_hosts() {
    let temp = Temp::new();
    for name in ["sigil.toml", "duplicate.toml"] {
        fs::write(
            temp.0.join(name),
            toml::to_string(&recipe("sigil")).unwrap(),
        )
        .unwrap();
    }
    let errors = lint_paths(
        std::slice::from_ref(&temp.0),
        &Path::new(CATALOG).join("hosts.toml"),
        None,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("duplicate recipe id"))
    );
    assert!(
        !lint_paths(
            &[temp.0.join("missing")],
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )
        .is_empty()
    );
    let empty = Temp::new();
    assert!(
        lint_paths(
            std::slice::from_ref(&empty.0),
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )[0]
        .message
        .contains("no recipe")
    );
    fs::write(
        empty.0.join("hosts.toml"),
        "schema_version=2\npages=[]\ndownloads=[]",
    )
    .unwrap();
    assert!(
        lint_paths(
            std::slice::from_ref(&empty.0),
            &empty.0.join("hosts.toml"),
            None
        )[0]
        .message
        .contains("invalid host")
    );
    fs::write(
        empty.0.join("large.toml"),
        " ".repeat(lint::MAX_RECIPE_BYTES as usize + 1),
    )
    .unwrap();
    assert!(
        lint_paths(
            &[empty.0.join("large.toml")],
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )[0]
        .message
        .contains("exceeds 1 MiB")
    );
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        writer
            .start_file(
                *name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
fn package(bytes: &[u8], path: &str, member: &[u8]) -> Package {
    Package {
        id: "fixture".into(),
        page: "https://romero.com/sigil".into(),
        urls: vec!["https://romero.com/one.zip".into()],
        size: bytes.len() as u64,
        sha256: sha(bytes),
        kind: PackageKind::Zip,
        idgames_path: None,
        members: vec![Member {
            id: "main".into(),
            path: path.into(),
            size: member.len() as u64,
            sha256: sha(member),
            keep: false,
        }],
    }
}

#[test]
fn fetch_checks_nested_zip_exact_case_size_and_hash_without_network() {
    let inner = zip_bytes(&[("Main.WAD", b"fixture")]);
    let outer = zip_bytes(&[("inner.zip", &inner)]);
    let p = package(&outer, "inner.zip!/Main.WAD", b"fixture");
    assert!(check_package(&p, &outer).is_ok());
    for (expected, mutation) in [
        ("size mismatch", 0),
        ("SHA-256 mismatch", 1),
        ("size mismatch", 2),
        ("SHA-256 mismatch", 3),
        ("missing exact member", 4),
    ] {
        let mut bad = p.clone();
        match mutation {
            0 => bad.size += 1,
            1 => bad.sha256 = "0".repeat(64),
            2 => bad.members[0].size += 1,
            3 => bad.members[0].sha256 = "0".repeat(64),
            _ => bad.members[0].path = "inner.zip!/main.wad".into(),
        }
        assert!(check_package(&bad, &outer).unwrap_err().contains(expected));
    }
    let mut bare = package(b"wad", "main.wad", b"wad");
    bare.kind = PackageKind::Wad;
    assert!(check_package(&bare, b"wad").is_ok());
    let corrupt = package(b"not a zip", "main.wad", b"wad");
    assert!(
        check_package(&corrupt, b"not a zip")
            .unwrap_err()
            .contains("invalid ZIP")
    );
}

#[test]
fn fetch_rejects_archive_traversal_symlinks_and_bombs() {
    for name in ["../evil.wad", "/evil.wad", "C:/evil.wad"] {
        let bytes = zip_bytes(&[(name, b"evil"), ("main.wad", b"good")]);
        assert!(
            check_package(&package(&bytes, "main.wad", b"good"), &bytes)
                .unwrap_err()
                .contains("unsafe ZIP entry")
        );
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "main.wad",
            "/outside",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    assert!(
        check_package(&package(&bytes, "main.wad", b"good"), &bytes)
            .unwrap_err()
            .contains("symlink")
    );
    // Patch only the central-directory uncompressed size; avoid allocating a bomb.
    let mut bytes = zip_bytes(&[("main.wad", b"good")]);
    let central = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[central + 24..central + 28]
        .copy_from_slice(&((lint::MAX_BYTES + 1) as u32).to_le_bytes());
    assert!(
        check_package(&package(&bytes, "main.wad", b"good"), &bytes)
            .unwrap_err()
            .contains("budget")
    );
}

#[test]
fn deh_order_is_separate_and_members_are_never_implicitly_loaded() {
    let mut r = recipe("sigil");
    let mut patch = r.packages[0].members[0].clone();
    patch.id = "patch".into();
    patch.path = "patch.bex".into();
    r.packages[0].members.push(patch);
    r.components[0].members.push(ComponentMember {
        package: Some("free".into()),
        member: Some("patch".into()),
        role: Role::Dehacked,
    });
    assert!(validate(&r, Path::new("sigil.toml"), &hosts()).is_empty());
    let order = r.load_order(&[]).unwrap();
    assert_eq!(order.files.len(), 1);
    assert_eq!(order.deh.len(), 1);
    assert_eq!(order.deh[0].member.as_deref(), Some("patch"));
}

#[test]
fn injected_fetch_verifies_declared_bytes_and_reports_failures() {
    struct Fake {
        bytes: Vec<u8>,
        calls: usize,
    }
    impl Fetch for Fake {
        fn download(&mut self, _package: &Package, _hosts: &Hosts) -> Result<Vec<u8>, String> {
            self.calls += 1;
            Ok(self.bytes.clone())
        }
    }

    let bytes = zip_bytes(&[("main.wad", b"good")]);
    let package = package(&bytes, "main.wad", b"good");
    let mut fake = Fake {
        bytes: bytes.clone(),
        calls: 0,
    };
    assert!(fetch::fetch_package(&package, &hosts(), &mut fake).is_ok());
    assert_eq!(fake.calls, 1);

    // A fetch that returns the wrong bytes is reported with the package id.
    fake.bytes = vec![];
    fake.calls = 0;
    let error = fetch::fetch_package(&package, &hosts(), &mut fake).unwrap_err();
    assert!(error.contains("size mismatch"), "{error}");
    assert!(error.contains("package `fixture`"), "{error}");
    assert_eq!(fake.calls, 1);
}

#[test]
fn lint_fetch_calls_the_injected_boundary_only_when_requested() {
    struct Fake {
        bytes: Vec<u8>,
        calls: usize,
    }
    impl Fetch for Fake {
        fn download(&mut self, _package: &Package, _hosts: &Hosts) -> Result<Vec<u8>, String> {
            self.calls += 1;
            Ok(self.bytes.clone())
        }
    }

    let bytes = zip_bytes(&[("main.wad", b"good")]);
    let temp = Temp::new();
    let mut recipe = recipe("sigil");
    recipe.packages = vec![package(&bytes, "main.wad", b"good")];
    recipe.packages[0].id = "free".into();
    recipe.packages[0].members[0].id = "maps".into();
    recipe.components.truncate(1);
    fs::write(temp.0.join("sigil.toml"), toml::to_string(&recipe).unwrap()).unwrap();

    let mut fake = Fake { bytes, calls: 0 };
    // Offline lint (None) never touches the boundary.
    assert!(
        lint_paths(
            std::slice::from_ref(&temp.0),
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )
        .is_empty()
    );
    assert_eq!(fake.calls, 0);
    // --fetch invokes it once per package.
    assert!(
        lint_paths(
            std::slice::from_ref(&temp.0),
            &Path::new(CATALOG).join("hosts.toml"),
            Some(&mut fake)
        )
        .is_empty()
    );
    assert_eq!(fake.calls, 1);
}

#[test]
fn https_fetch_downloads_through_the_shared_006_pipeline() {
    use crate::downloads::test_http::{Response, Server, TestDir};

    let body = zip_bytes(&[("main.wad", b"good")]);
    let served = body.clone();
    let server = Server::start(move |_req| Response::ok(served.clone()));
    let dir = TestDir::new("catalog-fetch");

    let mut package = package(&body, "main.wad", b"good");
    package.urls = vec![server.url("/pkg.zip")];
    let hosts = Hosts {
        schema_version: 1,
        pages: vec!["127.0.0.1".into()],
        downloads: vec!["127.0.0.1".into()],
    };

    let mut fetch = HttpsFetch::new(dir.path()).allow_http_for_tests();
    fetch
        .download(&package, &hosts)
        .expect("the shared fetcher downloads and verifies the package");

    // A wrong declared hash is a typed fetcher failure, not a silent success.
    package.sha256 = "0".repeat(64);
    let error = fetch.download(&package, &hosts).unwrap_err();
    assert!(error.contains("SHA-256"), "{error}");
}

#[test]
fn embedded_catalog_loads_the_reference_recipes_without_skips() {
    let catalog = Catalog::embedded();
    assert!(
        catalog.skipped.is_empty(),
        "the shipped catalog must be clean: {:?}",
        catalog.skipped
    );
    let mut ids: Vec<&str> = catalog.ids().collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        [
            "ancient-aliens",
            "auger-zenith",
            "eviternity",
            "sigil",
            "sigil-ii",
            "sunlust"
        ]
    );
    for id in [
        "ancient-aliens",
        "auger-zenith",
        "eviternity",
        "sigil",
        "sigil-ii",
        "sunlust",
    ] {
        let loaded = catalog.get(id).expect("recipe present");
        assert!(loaded.file.ends_with(&format!("{id}.toml")));
        assert_eq!(loaded.sha256.len(), 64);
        assert_eq!(
            loaded.sha256,
            sha(fs::read_to_string(format!("{CATALOG}/{}", loaded.file))
                .unwrap()
                .as_bytes())
        );
    }
    assert!(!catalog.hosts.downloads.is_empty());
}

#[test]
fn invalid_recipes_are_skipped_with_a_logged_reason_never_crashing() {
    let hosts_text = fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap();
    let good = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    // A recipe whose package hash is malformed must be skipped, not accepted.
    let broken = good.replace(
        "57c35a6bcd722433fc97344314e52c06c91fdfde33b18c6d1c14a4d29e229105",
        "not-a-hash",
    );
    // A file that is not even TOML.
    let garbage = "this is not toml".to_owned();

    let catalog = Catalog::from_sources(
        &hosts_text,
        &[
            ("sigil.toml", good.as_str()),
            ("broken.toml", broken.as_str()),
            ("garbage.toml", garbage.as_str()),
        ],
    );
    assert_eq!(catalog.ids().collect::<Vec<_>>(), ["sigil"]);
    assert_eq!(catalog.skipped.len(), 2, "{:?}", catalog.skipped);
    let broken = catalog
        .skipped
        .iter()
        .find(|entry| entry.file == "broken.toml")
        .expect("broken recipe reported");
    assert!(broken.reason.contains("SHA-256"), "{broken:?}");
    let garbage = catalog
        .skipped
        .iter()
        .find(|entry| entry.file == "garbage.toml")
        .expect("garbage recipe reported");
    assert!(!garbage.reason.is_empty());
}

#[test]
fn duplicate_recipe_ids_keep_the_first_file() {
    let hosts_text = fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap();
    let good = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    let catalog = Catalog::from_sources(
        &hosts_text,
        &[("sigil.toml", good.as_str()), ("sigil.toml", good.as_str())],
    );
    assert_eq!(catalog.recipes.len(), 1);
    assert_eq!(catalog.recipes[0].file, "sigil.toml");
    assert_eq!(catalog.skipped.len(), 1);
    assert!(catalog.skipped[0].reason.contains("duplicate recipe id"));
}

#[test]
fn semantic_diagnostics_locate_repeated_nested_keys() {
    let temp = Temp::new();
    let mut r = recipe("auger-zenith");
    r.packages[0].members[3].sha256 = "bad".into();
    let text = toml::to_string_pretty(&r).unwrap();
    let expected = text
        .lines()
        .position(|line| line.contains("\"bad\""))
        .unwrap()
        + 1;
    fs::write(temp.0.join("auger-zenith.toml"), &text).unwrap();
    let errors = lint_paths(
        std::slice::from_ref(&temp.0),
        &Path::new(CATALOG).join("hosts.toml"),
        None,
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line, expected, "{errors:?}");
}
