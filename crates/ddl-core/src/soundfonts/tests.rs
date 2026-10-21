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
    assert_eq!(fonts.effective().choice, None);
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
    let error = fonts
        .install_archive(
            &path,
            bytes.len() as u64,
            Sha256::of(&bytes),
            &members,
            &cancel,
            &mut |_| {},
        )
        .expect_err("a cancelled verification");
    assert_eq!(error.kind(), Kind::Cancelled, "{error}");
    let error = fonts
        .install_archive(
            &path,
            bytes.len() as u64,
            Sha256::of(b"wrong"),
            &members,
            &CancellationToken::new(),
            &mut |_| {},
        )
        .expect_err("an archive hash mismatch");
    assert_eq!(error.kind(), Kind::Checksum, "{error}");
    let error = fonts
        .install_archive(
            &path,
            bytes.len() as u64 + 1,
            Sha256::of(&bytes),
            &members,
            &CancellationToken::new(),
            &mut |_| {},
        )
        .expect_err("an archive size mismatch");
    assert_eq!(error.kind(), Kind::Checksum, "{error}");
    let mut wrong = members.clone();
    wrong[0].expected_sha256 = Some(Sha256::of(b"wrong member"));
    let error = install(&fonts, &path, &bytes, &wrong).expect_err("a member hash mismatch");
    assert_eq!(error.kind(), Kind::Checksum, "{error}");
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
        let error = install(&fonts, &path, &bytes, &members(font)).expect_err("invalid RIFF");
        assert_eq!(error.kind(), Kind::UnsafeArchive, "{error}");
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
    let effective = fonts.effective();
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
        let error =
            inspect_archive(&path, &CancellationToken::new()).expect_err("an unsafe archive name");
        assert_eq!(error.kind(), Kind::UnsafeArchive, "{name}: {error}");
    }
    let entries: Vec<_> = (0..455)
        .map(|i| (format!("{i}"), b"x".as_slice()))
        .collect();
    let refs: Vec<_> = entries
        .iter()
        .map(|(name, data)| (name.as_str(), *data))
        .collect();
    fs::write(&path, zip_bytes(&refs)).unwrap();
    let error = inspect_archive(&path, &CancellationToken::new()).expect_err("too many entries");
    assert_eq!(error.kind(), Kind::UnsafeArchive, "{error}");
    // Patch an unrequested entry's declared expanded size; the directory scan
    // rejects it before decompression or writing anything.
    let mut bytes = zip_bytes(&[("safe", b"ok")]);
    let index = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[index + 24..index + 28].copy_from_slice(&u32::MAX.to_le_bytes());
    fs::write(&path, bytes).unwrap();
    let error = inspect_archive(&path, &CancellationToken::new()).expect_err("expansion bound");
    assert_eq!(error.kind(), Kind::UnsafeArchive, "{error}");
    fs::write(&path, b"not a ZIP at all").unwrap();
    let error = inspect_archive(&path, &CancellationToken::new()).expect_err("malformed ZIP");
    assert_eq!(error.kind(), Kind::UnsafeArchive, "{error}");
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
    let error = inspect_archive(&path, &CancellationToken::new()).expect_err("duplicate names");
    assert_eq!(error.kind(), Kind::UnsafeArchive, "{error}");
    let mut bytes = zip_bytes(&[("safe", b"ok")]);
    let index = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[index + 5] = 3;
    bytes[index + 38..index + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
    fs::write(&path, bytes).unwrap();
    let error = inspect_archive(&path, &CancellationToken::new()).expect_err("a symlink entry");
    assert_eq!(error.kind(), Kind::UnsafeArchive, "{error}");
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

#[test]
fn snapshot_stamps_invalidate_on_member_and_pointer_changes() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    install(&fonts, &path, &bytes, &members(&font)).unwrap();
    // Use the production reviewed notice path for the metadata-only stamp test.
    let installed: Installed = read_json(&fonts.root.join("installed.json")).unwrap();
    let root = fonts.root.join(&installed.generation);
    let notice = root.join(MEMBERS[2].0);
    fs::write(&notice, b"notice").unwrap();
    let before = fonts.file_stamps().unwrap();
    fs::write(&notice, b"edited").unwrap();
    assert_ne!(fonts.file_stamps().unwrap(), before);
    fs::remove_file(root.join(FONT_FILE)).unwrap();
    assert!(fonts.file_stamps().is_err());
}

/// Spec 067 review: the safe point is before the atomic publication boundary.
/// A cancel requested once Commit starts must not publish a partial generation;
/// this fixture has already passed verification and extraction, so publication
/// completes and the published files verify.
#[test]
fn cancellation_at_publication_does_not_interrupt_the_atomic_commit() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    let cancel = CancellationToken::new();
    fonts
        .install_archive(
            &path,
            bytes.len() as u64,
            Sha256::of(&bytes),
            &members,
            &cancel,
            &mut |p| {
                if p.stage == InstallStage::Commit {
                    cancel.cancel();
                }
            },
        )
        .expect("publication completes once the atomic boundary is entered");
    fonts.verify_members(&members).unwrap();
    assert!(fonts.install_present_with(&members));
}

#[test]
fn cancelling_at_extraction_boundary_leaves_previous_install_intact() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    install(&fonts, &path, &bytes, &members).unwrap();
    let previous = fonts.verify_members(&members).unwrap();
    let cancel = CancellationToken::new();
    let error = fonts
        .install_archive(
            &path,
            bytes.len() as u64,
            Sha256::of(&bytes),
            &members,
            &cancel,
            &mut |p| {
                if p.stage == InstallStage::Extract {
                    cancel.cancel();
                }
            },
        )
        .expect_err("a late extraction cancellation");
    assert_eq!(error.kind(), Kind::Cancelled, "{error}");
    assert_eq!(fonts.verify_members(&members).unwrap(), previous);
}

#[test]
fn explicit_arachno_selection_survives_restart_and_returns_the_verified_path() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    install(&fonts, &path, &bytes, &members).unwrap();
    fonts.select_members(Choice::Arachno, &members).unwrap();

    let effective = fonts.effective_with(&members);
    assert_eq!(effective.choice, Some(Choice::Arachno));
    assert_eq!(
        effective.path,
        Some(fonts.verify_members(&members).unwrap()),
        "the explicit choice owns the checked absolute path"
    );
    assert!(effective.warning.is_none());

    let restarted = service(dir.path());
    assert_eq!(
        restarted.preferences().unwrap().choice,
        Some(Choice::Arachno)
    );
    assert_eq!(
        restarted.effective_with(&members).path,
        effective.path,
        "a restart restores the selection"
    );

    fs::write(effective.path.unwrap(), b"RIFF\x04\0\0\0sfbx").unwrap();
    assert!(
        restarted.select_members(Choice::Arachno, &members).is_err(),
        "a damaged font is never selectable"
    );
    assert_eq!(
        restarted.effective_with(&members).choice,
        Some(Choice::Default),
        "a damaged selected font falls back, keeping the persisted selection"
    );
    assert_eq!(
        restarted.preferences().unwrap().choice,
        Some(Choice::Arachno)
    );
}

#[test]
fn install_present_checks_pointer_and_sizes_but_is_not_a_verification() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    let font = sf2();
    let (path, bytes) = archive(dir.path(), &font);
    let members = members(&font);
    assert!(!fonts.install_present_with(&members));
    install(&fonts, &path, &bytes, &members).unwrap();
    assert!(fonts.install_present_with(&members));

    // Same length, different bytes: the cheap snapshot still calls it present,
    // while the launch-grade check rejects it.
    let installed = fonts.verify_members(&members).unwrap();
    fs::write(&installed, b"RIFF\x04\0\0\0sfbx").unwrap();
    assert!(fonts.install_present_with(&members));
    assert!(fonts.verify_members(&members).is_err());
    fs::remove_file(installed).unwrap();
    assert!(
        !fonts.install_present_with(&members),
        "a missing reviewed file"
    );
}

#[test]
fn corrupt_audio_settings_preserve_the_engine_and_report_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let fonts = service(dir.path());
    fs::create_dir_all(fonts.preferences.parent().unwrap()).unwrap();
    fs::write(&fonts.preferences, b"{ not json").unwrap();
    let effective = fonts.effective();
    assert_eq!(
        effective.choice, None,
        "DDL writes nothing it cannot interpret"
    );
    assert_eq!(effective.path, None);
    let warning = effective.warning.unwrap();
    assert!(warning.contains("could not be read"), "{warning}");
    assert!(fonts.preferences().is_err());
    fonts.select(Choice::Default).unwrap();
    assert_eq!(
        fonts.preferences().unwrap().choice,
        Some(Choice::Default),
        "the player can still make an explicit choice"
    );
}

#[test]
fn reviewed_source_constants_match_the_pinned_research() {
    assert_eq!(
        SOURCE,
        "https://www.dropbox.com/s/2rnpya9ecb9m4jh/arachno-soundfont-10-sf2.zip?dl=1"
    );
    assert_eq!(
        TERMS,
        "https://www.arachnosoft.com/main/soundfont.php?documentation="
    );
    assert_eq!(AUTHOR, "Maxime Abbey");
    assert_eq!(DOWNLOAD_BYTES, 143356181);
    assert_eq!(ZIP_HASH.len(), 64);
    assert_eq!(FONT_HASH.len(), 64);
    assert_eq!(MEMBERS[0].1, 155405818);
    assert_eq!(MEMBERS[1].1, 3388);
    assert_eq!(MEMBERS[2].1, 441987);
    let limits = limits();
    assert_eq!(limits.max_total_uncompressed, 173559660);
    assert_eq!(limits.max_member_uncompressed, 155405818);
    assert_eq!(limits.max_member_count, 454);
    assert_eq!(limits.max_nesting_depth, 0);
    assert!(limits.max_compression_ratio <= 64);
}

#[test]
fn rejects_member_expansion_beyond_the_reviewed_compression_ratio() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bomb.zip");
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("filler.bin", options).unwrap();
    zip.write_all(&vec![0u8; 256 * 1024]).unwrap();
    fs::write(&path, zip.finish().unwrap().into_inner()).unwrap();
    assert!(inspect_archive(&path, &CancellationToken::new()).is_err());
}
