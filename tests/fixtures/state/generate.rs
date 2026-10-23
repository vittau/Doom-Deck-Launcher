#[cfg(test)]
mod release_state_fixture {
    use super::*;
    #[test]
    fn generate_release_state_fixture() {
        let root = PathBuf::from(std::env::var("DDL_FIXTURE_OUTPUT").unwrap());
        let config = root.join("config");
        let data = root.join("data");
        fs::create_dir_all(&config).unwrap();
        let catalog = Catalog::embedded();
        let mut state = AppState::default();
        state.folders = vec!["/ddl-fixture/iwads-one".into(), "/ddl-fixture/iwads-two".into()];
        state.catalog_auto_update = false;
        state.search_standard_folders = true;
        state.favorites.insert("auger-zenith".into());
        state.last_profiles.insert("auger-zenith".into(), "fixture-alternate".into());
        let profiles = ProfileStore::new(&config, &data);
        for id in ["auger-zenith", "sigil"] {
            let recipe = catalog.recipe(id).unwrap().clone();
            state.kept_recipes.insert(id.into(), recipe_content_sha256(&recipe).unwrap());
            state.recipes.insert(id.into(), recipe.clone());
            let dir = data.join("library").join(id).join("generations/fixture");
            fs::create_dir_all(&dir).unwrap();
            let manifest = InstallManifest {
                schema_version: 1, recipe_id: id.into(), release: recipe.release.clone(),
                recipe_sha256: recipe_content_sha256(&recipe).unwrap(), installed_at: 1700000000,
                packages: recipe.packages.iter().map(|p| ddl_core::library::InstalledPackage {
                    id: p.id.clone(), sha256: p.sha256.clone(), size: p.size,
                }).collect(),
                members: recipe.packages.iter().flat_map(|p| p.members.iter().map(move |m| ddl_core::library::InstalledMember {
                    package: Some(p.id.clone()), member: Some(m.id.clone()), component: None,
                    path: PathBuf::from("/ddl-fixture/data/library").join(id).join("generations/fixture").join(&p.id).join(&m.path),
                    size: m.size, sha256: m.sha256.clone(), keep: m.keep,
                })).collect(),
            };
            manifest.save(&dir.join("install.json")).unwrap();
            ActiveInstall {
                schema_version: 1, recipe: recipe.clone(), manifest,
                previous_recipe: Some(recipe.clone()), pending_profiles: vec![], notices: vec![],
                retired_dirs: vec![], uninstalled: false,
            }.save(&data.join("library").join(id).join("active.json")).unwrap();
            let mut profile = Profile::new(format!("fixture-{id}"), "Original", id);
            profile.components = recipe.default_components();
            profiles.save(&profile).unwrap();
            if id == "auger-zenith" {
                profile.id = "fixture-alternate".into();
                profile.name = "Alternate".into();
                profile.skill = 2;
                if let Some(optional) = recipe.components.iter().find(|c| c.optional && !c.user_supplied) {
                    profile.components = vec![optional.id.clone()];
                }
                profiles.save(&profile).unwrap();
            }
        }
        save_json(&config.join("app.json"), &state).unwrap();
        let mut registry = Registry::new();
        for folder in ["iwads-one", "iwads-two"] {
            registry.insert(RegistryEntry {
                path: PathBuf::from("/ddl-fixture").join(folder).join("doom2.wad"),
                variant: Some("doom2-1.9".into()), candidates: vec![], game: Some(iwads::Game::Doom2),
                sha256: "0".repeat(64), size: 12,
                mtime: iwads::FileTime { secs: 1700000000, nanos: 0 }, source: iwads::Source::UserFolder,
            });
        }
        registry.save(&config.join("iwads.json")).unwrap();
        let mut engine = dsda::InstalledState::default();
        engine.active = Some("0.30.1".into());
        engine.installed.push(dsda::InstalledEngine {
            version: "0.30.1".into(), path: "/ddl-fixture/dsda".into(),
            mode: dsda::Mode::Manual, sha256: "0".repeat(64), source_url: None,
            tag: Some("v0.30.1".into()), reviewed: true, origin: dsda::Origin::Manual,
            installed_at: 1700000000,
        });
        fs::create_dir_all(data.join("engines/dsda-doom")).unwrap();
        engine.save(&data.join("engines/dsda-doom/installed.json")).unwrap();
        DsdaConfigManager::new(&config).unwrap().provision().unwrap();
        // AUDIO_START
        // An explicit Default uses the release's production preferences writer
        // without downloading or fabricating an installed soundfont.
        Soundfonts::new(&Paths {
            config: config.clone(), data: data.clone(),
            cache: root.join("cache"), state: root.join("state"),
        }).select(Choice::Default).unwrap();
        // AUDIO_END
        // VIEW_START
        // Use this tag's production writer to preserve an explicit List choice.
        super::view_preferences::save_view_format(&Environment {
            paths: Paths {
                config: config.clone(), data: data.clone(),
                cache: root.join("cache"), state: root.join("state"),
            },
            home: "/ddl-fixture".into(), screen: None, test: None,
        }, ViewFormat::List).unwrap();
        // VIEW_END
        // PICKER_START
        // Use this tag's production local-picker writer with a folder both
        // supported hosts can enumerate, so the saved preference is real state
        // and not hand-written JSON. It always falls back to Home when absent.
        ddl_core::local::picker::remember(&config, Path::new("/tmp"))
            .expect("the fixture host can enumerate /tmp");
        // PICKER_END
        SetupStore::new(&data).save(&CustomSetup::new(
            "fixture-custom", "My custom setup", CustomIwad::Identified { variant: "doom2-1.9".into() },
            Compatibility::Auto, 1700000000,
        )).unwrap();
    }
}
