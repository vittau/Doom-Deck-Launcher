//! Spec 106 service boundaries; entirely offline, player data is synthetic.
use super::*;
fn environment(root: &Path) -> Environment {
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    Environment {
        paths,
        home,
        screen: None,
        test: None,
    }
    .without_catalog_auto_updates()
}
fn wad(path: &Path, extra: &[u8], iwad: bool) {
    let mut bytes = if iwad {
        b"IWAD\0\0\0\0\x0c\0\0\0".to_vec()
    } else {
        b"PWAD\0\0\0\0\x0c\0\0\0".to_vec()
    };
    bytes.extend_from_slice(extra);
    fs::write(path, bytes).unwrap();
}
fn op(s: &mut Services, command: LocalCommand) -> Result<Refresh, Problem> {
    s.execute(
        Command::Local(command),
        &CancellationToken::new(),
        &mut |_| {},
    )
}
fn setup(s: &mut Services) -> CustomSetup {
    let base = s.env.home.join("base.wad");
    wad(&base, b"base", true);
    let setup = s
        .custom_create(
            "Custom setup 1",
            Some("local"),
            NewIwad::File(base),
            Compatibility::Auto,
        )
        .unwrap();
    for name in ["first.wad", "middle.wad", "last.wad"] {
        wad(&s.env.home.join(name), name.as_bytes(), false);
    }
    s.custom_add_file(
        &setup.id,
        &[
            s.env.home.join("first.wad"),
            s.env.home.join("middle.wad"),
            s.env.home.join("last.wad"),
        ],
    )
    .unwrap()
}
#[test]
fn local_polish_browse_is_light_remembers_only_success_and_optional_corruption_is_safe() {
    let t = tempfile::tempdir().unwrap();
    let env = environment(t.path());
    let mut s = Services::open(env.clone()).unwrap();
    let _ = setup(&mut s);
    // A broken library record cannot affect a listing completion (no snapshot tail).
    fs::remove_file(env.home.join("first.wad")).unwrap();
    let folder = env.home.join("maps");
    fs::create_dir(&folder).unwrap();
    wad(&folder.join("large.wad"), b"", false);
    fs::OpenOptions::new()
        .write(true)
        .open(folder.join("large.wad"))
        .unwrap()
        .set_len(1024 * 1024 * 1024)
        .unwrap();
    fs::write(folder.join("wrong.wad"), b"text").unwrap();
    fs::write(folder.join("patch.bex"), b"[STRINGS]\n").unwrap();
    let mut listed = None;
    assert!(matches!(
        s.execute(
            Command::Local(LocalCommand::Browse(folder.clone())),
            &CancellationToken::new(),
            &mut |e| if let Event::Directory(d) = e {
                listed = Some(d)
            }
        )
        .unwrap(),
        Refresh::None
    ));
    let d = listed.unwrap();
    assert_eq!(d.files.len(), 2);
    assert_eq!(
        d.files
            .iter()
            .find(|f| f.kind == local::Kind::Wad)
            .unwrap()
            .size,
        1024 * 1024 * 1024
    );
    assert!(op(&mut s, LocalCommand::Browse(folder.join("missing"))).is_err());
    assert_eq!(
        local::picker::remembered(&env.paths.config, &env.home),
        folder
    );
    drop(s);
    let mut s = Services::open(env.clone()).unwrap();
    let mut reopened = None;
    s.execute(
        Command::Local(LocalCommand::OpenPicker(None)),
        &CancellationToken::new(),
        &mut |e| {
            if let Event::Directory(d) = e {
                reopened = Some(d.path)
            }
        },
    )
    .unwrap();
    assert_eq!(reopened, Some(folder.clone()));
    fs::remove_dir_all(&folder).unwrap();
    assert_eq!(
        local::picker::remembered(&env.paths.config, &env.home),
        env.home
    );
    fs::write(
        env.paths.config.join("local-picker.json"),
        b"corrupt optional JSON",
    )
    .unwrap();
    assert!(
        Services::open(env).is_ok(),
        "optional preference never triggers recovery"
    );
}
#[test]
fn local_polish_relocation_preserves_middle_slot_and_rejects_stale_wrong_missing_and_cancelled() {
    let t = tempfile::tempdir().unwrap();
    let mut s = Services::open(environment(t.path())).unwrap();
    let before = setup(&mut s);
    let original_bytes = fs::read(&before.files[1].path).unwrap();
    let moved = s.env.home.join("moved.wad");
    fs::rename(&before.files[1].path, &moved).unwrap();
    let target = ReplacementTarget::Custom(before.clone(), LocalSlot::File(1));
    let file = UserFile::read(&moved).unwrap();
    assert_eq!(target.file().unwrap().sha256, file.sha256);
    op(
        &mut s,
        LocalCommand::ApplyReplacement(target.clone(), file.clone()),
    )
    .unwrap();
    let after = s.custom_show("local").unwrap();
    assert_eq!(after.files[0], before.files[0]);
    assert_eq!(after.files[2], before.files[2]);
    assert_eq!(after.iwad, before.iwad);
    assert_eq!(after.id, before.id);
    assert_eq!(after.files[1].path, moved);
    assert!(
        op(&mut s, LocalCommand::ApplyReplacement(target, file.clone())).is_err(),
        "old target cannot overwrite a repaired setup"
    );
    let target = ReplacementTarget::Custom(after.clone(), LocalSlot::File(1));
    let patch = s.env.home.join("wrong.deh");
    fs::write(&patch, b"patch text").unwrap();
    assert!(
        op(
            &mut s,
            LocalCommand::PrepareReplacement(target.clone(), patch)
        )
        .is_err()
    );
    let missing = s.env.home.join("missing.wad");
    assert!(
        op(
            &mut s,
            LocalCommand::PrepareReplacement(target.clone(), missing)
        )
        .is_err()
    );
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        s.execute(
            Command::Local(LocalCommand::ApplyReplacement(target.clone(), file)),
            &cancel,
            &mut |_| {}
        )
        .is_err()
    );
    let different = s.env.home.join("different.wad");
    wad(&different, b"different bytes", false);
    let file = UserFile::read(&different).unwrap();
    let mut prepared = false;
    s.execute(
        Command::Local(LocalCommand::PrepareReplacement(
            target.clone(),
            different.clone(),
        )),
        &CancellationToken::new(),
        &mut |e| {
            if let Event::Replacement(_, _) = e {
                prepared = true
            }
        },
    )
    .unwrap();
    assert!(prepared);
    assert_eq!(
        s.custom_show("local").unwrap(),
        after,
        "preparation/cancel is read only"
    );
    wad(&different, b"mutated after confirmation", false);
    assert!(op(&mut s, LocalCommand::ApplyReplacement(target.clone(), file)).is_err());
    let file = UserFile::read(&different).unwrap();
    op(&mut s, LocalCommand::ApplyReplacement(target, file)).unwrap();
    assert_eq!(fs::read(&moved).unwrap(), original_bytes);
    assert_eq!(
        fs::read(&different).unwrap(),
        b"PWAD\0\0\0\0\x0c\0\0\0mutated after confirmation"
    );
    let reloaded = Services::open(s.env.clone())
        .unwrap()
        .custom_show("local")
        .unwrap();
    assert_eq!(reloaded.files[1].path, different);
    // IWAD slots require an IWAD header, even with the right WAD route.
    let target = ReplacementTarget::Custom(reloaded.clone(), LocalSlot::Iwad);
    assert!(
        op(
            &mut s,
            LocalCommand::PrepareReplacement(target.clone(), moved)
        )
        .is_err()
    );
    let new_base = s.env.home.join("new-base.wad");
    if let CustomIwad::File { file } = &reloaded.iwad {
        fs::rename(&file.path, &new_base).unwrap();
    }
    op(
        &mut s,
        LocalCommand::ApplyReplacement(target, UserFile::read(&new_base).unwrap()),
    )
    .unwrap();
}
#[test]
fn local_polish_extras_relocation_keeps_exact_profile_and_unrelated_saves() {
    let t = tempfile::tempdir().unwrap();
    let mut s = Services::open(environment(t.path())).unwrap();
    let base = s.env.home.join("registered.wad");
    wad(&base, b"synthetic registered base", true);
    let metadata = fs::metadata(&base).unwrap();
    s.registry.insert(RegistryEntry {
        path: base,
        variant: Some("doom2-1.9".into()),
        candidates: vec![],
        game: Some(iwads::Game::Doom2),
        sha256: "0".repeat(64),
        size: metadata.len(),
        mtime: iwads::FileTime::from_metadata(&metadata).unwrap(),
        source: iwads::Source::Manual,
    });
    for recipe in ["sigil", "base:doom2"] {
        let path = s.env.home.join("extra.wad");
        wad(&path, b"extra", false);
        let mut p = Profile::new(format!("{}-a", recipe.replace(':', "-")), "One", recipe);
        p.extras.push(UserFile::read(&path).unwrap());
        s.profiles.save(&p).unwrap();
        let mut other = p.clone();
        other.id.push_str("-other");
        other.name = "Other".into();
        s.profiles.save(&other).unwrap();
        let saves = s.profiles.dirs(&p.id).save;
        fs::create_dir_all(&saves).unwrap();
        fs::write(saves.join("slot.dsg"), b"save sentinel").unwrap();
        let replacement = s.env.home.join("relocated.wad");
        fs::rename(&path, &replacement).unwrap();
        let target = ReplacementTarget::Extras(p.clone(), 0);
        op(
            &mut s,
            LocalCommand::ApplyReplacement(target, UserFile::read(&replacement).unwrap()),
        )
        .unwrap();
        let actual = s.profiles.load(&p.id).unwrap();
        assert_eq!(actual.extras[0].path, replacement);
        assert_eq!(actual.id, p.id);
        assert_eq!(s.profiles.load(&other.id).unwrap(), other);
        assert_eq!(fs::read(saves.join("slot.dsg")).unwrap(), b"save sentinel");
        let target = ReplacementTarget::Extras(actual.clone(), 0);
        let mut newer = actual;
        newer.skill = 3;
        s.profiles.save(&newer).unwrap();
        assert!(
            op(
                &mut s,
                LocalCommand::ApplyReplacement(target, UserFile::read(&replacement).unwrap())
            )
            .is_err()
        );
    }
}
#[test]
fn local_polish_naming_only_unrenamed_new_content_and_rename_preserves_identity() {
    let t = tempfile::tempdir().unwrap();
    let mut s = Services::open(environment(t.path())).unwrap();
    let mut before = setup(&mut s);
    before.files.clear();
    s.save_custom(before.clone()).unwrap();
    let patch = s.env.home.join("patch.deh");
    fs::write(&patch, b"patch").unwrap();
    op(
        &mut s,
        LocalCommand::AddNamed(
            before.id.clone(),
            None,
            vec![patch],
            Some(before.name.clone()),
        ),
    )
    .unwrap();
    assert_eq!(s.custom_show("local").unwrap().name, before.name);
    let path = s.env.home.join("夜__maps--v2.wad");
    wad(&path, b"map", false);
    op(
        &mut s,
        LocalCommand::AddNamed(before.id.clone(), None, vec![path], Some(before.name)),
    )
    .unwrap();
    let named = s.custom_show("local").unwrap();
    assert_eq!(named.name, "夜 maps v2");
    let profiles = s.profiles.list().unwrap();
    op(
        &mut s,
        LocalCommand::Rename(named.clone(), "Chosen name".into()),
    )
    .unwrap();
    let mut expected = named;
    expected.name = "Chosen name".into();
    assert_eq!(s.custom_show("local").unwrap(), expected);
    assert_eq!(s.profiles.list().unwrap(), profiles);
    assert!(op(&mut s, LocalCommand::Rename(expected.clone(), " ".into())).is_err());
    assert_eq!(s.custom_show("local").unwrap(), expected);
}

#[test]
fn local_polish_add_failure_cancel_and_listing_mutation_never_publish_partial_content() {
    let t = tempfile::tempdir().unwrap();
    let mut s = Services::open(environment(t.path())).unwrap();
    let before = setup(&mut s);
    let selected = s.env.home.join("selected.wad");
    wad(&selected, b"first bytes", false);
    local::Inspection::read(&selected).unwrap();
    fs::write(&selected, b"now invalid").unwrap();
    let valid = s.env.home.join("valid.wad");
    wad(&valid, b"valid", false);
    assert!(
        op(
            &mut s,
            LocalCommand::AddNamed(
                before.id.clone(),
                None,
                vec![valid.clone(), selected.clone()],
                None
            )
        )
        .is_err()
    );
    assert_eq!(s.custom_show("local").unwrap(), before);
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        s.execute(
            Command::Local(LocalCommand::AddNamed(
                before.id.clone(),
                None,
                vec![valid.clone()],
                None
            )),
            &cancel,
            &mut |_| {}
        )
        .is_err()
    );
    assert_eq!(s.custom_show("local").unwrap(), before);
    wad(&selected, b"fresh valid content", false);
    op(
        &mut s,
        LocalCommand::AddNamed(before.id.clone(), None, vec![selected.clone()], None),
    )
    .unwrap();
    assert_eq!(
        s.custom_show("local").unwrap().files.last().unwrap(),
        &UserFile::read(&selected).unwrap()
    );
    fs::remove_file(&selected).unwrap();
    assert!(
        op(
            &mut s,
            LocalCommand::AddNamed(before.id, None, vec![selected], None)
        )
        .is_err()
    );
}

#[test]
fn local_polish_open_picker_propagates_home_errors_and_cancel_without_fallback() {
    let t = tempfile::tempdir().unwrap();
    let mut s = Services::open(environment(t.path())).unwrap();
    let folder = s.env.home.join("maps");
    fs::create_dir(&folder).unwrap();
    local::picker::remember(&s.env.paths.config, &folder).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut emitted = false;
    assert!(
        s.execute(
            Command::Local(LocalCommand::OpenPicker(Some(folder.clone()))),
            &cancel,
            &mut |_| emitted = true
        )
        .is_err()
    );
    assert!(!emitted);
    assert_eq!(
        local::picker::remembered(&s.env.paths.config, &s.env.home),
        folder
    );
    fs::remove_dir_all(&s.env.home).unwrap();
    assert!(op(&mut s, LocalCommand::OpenPicker(None)).is_err());
}

#[test]
fn local_polish_default_name_respects_custom_names_existing_setups_empty_stems_and_collisions() {
    let t = tempfile::tempdir().unwrap();
    let mut s = Services::open(environment(t.path())).unwrap();
    let mut setup = setup(&mut s);
    setup.files.clear();
    setup.name = "User choice".into();
    s.save_custom(setup.clone()).unwrap();
    let path = s.env.home.join("same-name.wad");
    wad(&path, b"maps", false);
    op(
        &mut s,
        LocalCommand::AddNamed(
            setup.id.clone(),
            None,
            vec![path.clone()],
            Some("Custom setup 1".into()),
        ),
    )
    .unwrap();
    assert_eq!(s.custom_show("local").unwrap().name, "User choice");
    let mut existing = setup.clone();
    existing.files.clear();
    existing.name = "Existing".into();
    s.save_custom(existing.clone()).unwrap();
    op(
        &mut s,
        LocalCommand::AddNamed(existing.id.clone(), None, vec![path.clone()], None),
    )
    .unwrap();
    assert_eq!(s.custom_show("local").unwrap().name, "Existing");
    // Two equal display names remain safe because IDs, profiles and paths do not derive from automatic naming.
    let base = match &setup.iwad {
        CustomIwad::File { file } => NewIwad::File(file.path.clone()),
        _ => unreachable!(),
    };
    s.custom_create(
        "same name",
        Some("collision"),
        base.clone(),
        Compatibility::Auto,
    )
    .unwrap();
    let second = s
        .custom_create(
            "placeholder 2",
            Some("second"),
            base.clone(),
            Compatibility::Auto,
        )
        .unwrap();
    op(
        &mut s,
        LocalCommand::AddNamed(second.id.clone(), None, vec![path], Some(second.name)),
    )
    .unwrap();
    assert_eq!(
        s.custom_show("second").unwrap().name,
        s.custom_show("collision").unwrap().name
    );
    assert_ne!(
        s.custom_show("second").unwrap().id,
        s.custom_show("collision").unwrap().id
    );
    let third = s
        .custom_create("placeholder", Some("third"), base, Compatibility::Auto)
        .unwrap();
    let empty = s.env.home.join("__--.wad");
    wad(&empty, b"maps", false);
    op(
        &mut s,
        LocalCommand::AddNamed(
            third.id.clone(),
            None,
            vec![empty],
            Some(third.name.clone()),
        ),
    )
    .unwrap();
    assert_eq!(s.custom_show("third").unwrap().name, third.name);
    assert_ne!(second.id, third.id);
}
