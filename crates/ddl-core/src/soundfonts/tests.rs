use super::*;
use std::io::Cursor;
use zip::write::SimpleFileOptions;

fn service(dir: &Path) -> Soundfonts {
    Soundfonts::new(&Paths {
        config: dir.join("config"),
        data: dir.join("data"),
        cache: dir.join("cache"),
        state: dir.join("state"),
    })
}
fn sf2() -> Vec<u8> {
    b"RIFF\x04\0\0\0sfbk".to_vec()
}
fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn members(font: &[u8]) -> Vec<MemberRequest> {
    [
        (FONT_FILE, font),
        ("Read Me.txt", b"notice".as_slice()),
        (
            "Documentation/English.html",
            b"<script>inert</script>".as_slice(),
        ),
    ]
    .iter()
    .map(|(path, data)| {
        MemberRequest::new(*path)
            .with_size(data.len() as u64)
            .with_sha256(Sha256::of(data))
    })
    .collect()
}
fn archive(dir: &Path, font: &[u8]) -> (PathBuf, Vec<u8>) {
    let bytes = zip_bytes(&[
        (FONT_FILE, font),
        ("Read Me.txt", b"notice"),
        ("Documentation/English.html", b"<script>inert</script>"),
        ("unrequested.mid", b"not installed"),
    ]);
    let path = dir.join("test.zip");
    fs::write(&path, &bytes).unwrap();
    (path, bytes)
}
fn install(
    service: &Soundfonts,
    package: &Path,
    bytes: &[u8],
    members: &[MemberRequest],
) -> Result<(), Error> {
    service.install_archive(
        package,
        bytes.len() as u64,
        Sha256::of(bytes),
        members,
        &CancellationToken::new(),
        &mut |_| {},
    )
}

#[test]
fn preferences_default_explicit_default_and_restart() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    assert_eq!(fonts.preferences().unwrap().choice, None);
    assert_eq!(fonts.effective().unwrap().choice, None);
    fonts.select(Choice::Default).unwrap();
    assert_eq!(
        service(dir.path()).preferences().unwrap().choice,
        Some(Choice::Default)
    );
    assert!(fonts.select(Choice::Arachno).is_err());
    assert_eq!(fonts.preferences().unwrap().choice, Some(Choice::Default));
}
#[test]
fn verified_generation_contains_only_reviewed_members_without_selecting() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    install(&fonts, &path, &bytes, &members).unwrap();
    let installed = fonts.verify_members(&members).unwrap();
    assert_eq!(fs::read(&installed).unwrap(), font);
    assert!(!installed.parent().unwrap().join("unrequested.mid").exists());
    assert_eq!(fonts.preferences().unwrap().choice, None);
    assert_eq!(
        fs::read(
            installed
                .parent()
                .unwrap()
                .join("Documentation/English.html")
        )
        .unwrap(),
        b"<script>inert</script>"
    );
}
#[test]
fn cancellation_bad_hash_and_retry_preserve_published_install_and_choice() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    install(&fonts, &path, &bytes, &members).unwrap();
    fonts.select(Choice::Default).unwrap();
    let pointer = fs::read(fonts.root.join("installed.json")).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        fonts
            .install_archive(
                &path,
                bytes.len() as u64,
                Sha256::of(&bytes),
                &members,
                &cancel,
                &mut |_| {}
            )
            .is_err()
    );
    assert!(
        fonts
            .install_archive(
                &path,
                bytes.len() as u64,
                Sha256::of(b"wrong"),
                &members,
                &CancellationToken::new(),
                &mut |_| {}
            )
            .is_err()
    );
    let mut wrong = members.clone();
    wrong[0].expected_sha256 = Some(Sha256::of(b"wrong member"));
    assert!(install(&fonts, &path, &bytes, &wrong).is_err());
    assert_eq!(
        fs::read(fonts.root.join("installed.json")).unwrap(),
        pointer
    );
    assert_eq!(fonts.preferences().unwrap().choice, Some(Choice::Default));
    install(&fonts, &path, &bytes, &members).unwrap();
    fonts.verify_members(&members).unwrap();
}
#[test]
fn invalid_riff_never_publishes_even_with_matching_hash() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    for font in [
        b"bad content!".as_slice(),
        b"RIFF\x05\0\0\0sfbk",
        b"RIFF\x04\0\0\0WAVE",
    ] {
        let (path, bytes) = archive(dir.path(), font);
        assert!(install(&fonts, &path, &bytes, &members(font)).is_err());
        assert!(!fonts.root.join("installed.json").exists());
    }
}
#[test]
fn interrupted_unpublished_generation_does_not_replace_previous_pointer() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    install(&fonts, &path, &bytes, &members).unwrap();
    let previous = fonts.verify_members(&members).unwrap();
    fs::create_dir(fonts.root.join("generation-orphan")).unwrap();
    fs::write(fonts.root.join("pending-abandoned"), b"partial JSON").unwrap();
    assert_eq!(
        service(dir.path()).verify_members(&members).unwrap(),
        previous
    );
}
#[test]
fn selected_missing_or_corrupt_font_falls_back_without_losing_selection() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    atomic_json(
        &fonts.preferences,
        &Preferences {
            schema_version: 1,
            choice: Some(Choice::Arachno),
        },
    )
    .unwrap();
    let effective = fonts.effective().unwrap();
    assert_eq!(effective.choice, Some(Choice::Default));
    assert_eq!(effective.path, None);
    assert!(effective.warning.unwrap().contains("reinstall"));
    assert_eq!(fonts.preferences().unwrap().choice, Some(Choice::Arachno));
}
#[test]
fn rejects_unsafe_unrequested_entries_and_whole_archive_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.zip");
    for name in [
        "../escape",
        "/absolute",
        "dir/../escape",
        "dir\\escape",
        "C:escape",
        "./escape",
    ] {
        fs::write(&path, zip_bytes(&[("safe", b"ok"), (name, b"bad")])).unwrap();
        assert!(
            inspect_archive(&path, &CancellationToken::new()).is_err(),
            "{name}"
        );
    }
    let entries: Vec<_> = (0..455)
        .map(|i| (format!("{i}"), b"x".as_slice()))
        .collect();
    let refs: Vec<_> = entries
        .iter()
        .map(|(name, data)| (name.as_str(), *data))
        .collect();
    fs::write(&path, zip_bytes(&refs)).unwrap();
    assert!(inspect_archive(&path, &CancellationToken::new()).is_err());
    // Patch an unrequested entry's declared expanded size; the directory scan
    // rejects it before decompression or writing anything.
    let mut bytes = zip_bytes(&[("safe", b"ok")]);
    let index = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[index + 24..index + 28].copy_from_slice(&u32::MAX.to_le_bytes());
    fs::write(&path, bytes).unwrap();
    assert!(inspect_archive(&path, &CancellationToken::new()).is_err());
}
#[test]
fn rejects_duplicate_and_symlink_unrequested_zip_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.zip");
    let mut bytes = zip_bytes(&[("first", b"ok"), ("other", b"ok")]);
    // Same-length replacement in local and central headers permits a duplicate
    // without relying on ZipWriter accepting duplicate names.
    for index in 0..bytes.len() - 5 {
        if &bytes[index..index + 5] == b"other" {
            bytes[index..index + 5].copy_from_slice(b"first");
        }
    }
    fs::write(&path, bytes).unwrap();
    assert!(inspect_archive(&path, &CancellationToken::new()).is_err());
    let mut bytes = zip_bytes(&[("safe", b"ok")]);
    let index = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[index + 5] = 3;
    bytes[index + 38..index + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
    fs::write(&path, bytes).unwrap();
    assert!(inspect_archive(&path, &CancellationToken::new()).is_err());
}
#[test]
fn rejects_unsupported_state_and_escaping_install_pointer() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    atomic_json(
        &fonts.preferences,
        &Preferences {
            schema_version: 2,
            choice: None,
        },
    )
    .unwrap();
    assert!(fonts.preferences().is_err());
    atomic_json(
        &fonts.root.join("installed.json"),
        &Installed {
            schema_version: 1,
            generation: "generation-../../escape".into(),
        },
    )
    .unwrap();
    assert!(fonts.verified_font().is_err());
}
#[test]
fn dsda_path_accepts_spaces_and_rejects_unrepresentable_strings() {
    assert_eq!(
        config_path(Path::new("/data/my fonts/font.sf2")).unwrap(),
        "/data/my fonts/font.sf2"
    );
    for path in [
        "relative.sf2",
        "/data/\"font.sf2",
        "/data/\nfont.sf2",
        "/data/\rfont.sf2",
        "/data/\0font.sf2",
    ] {
        assert!(config_path(Path::new(path)).is_err());
    }
}
#[cfg(unix)]
#[test]
fn installed_symlink_is_never_accepted() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    install(&fonts, &path, &bytes, &members).unwrap();
    let font_path = fonts.verify_members(&members).unwrap();
    fs::remove_file(&font_path).unwrap();
    let external = dir.path().join("external.sf2");
    fs::write(&external, font).unwrap();
    symlink(external, font_path).unwrap();
    assert!(fonts.verify_members(&members).is_err());
}
