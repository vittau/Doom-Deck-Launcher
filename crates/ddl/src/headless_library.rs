//! Spec 061: the Library (and every tab sharing the scroll plane) with an owner-like
//! setup, rendered off-screen at 1280×800.
use super::*;
use crate::{
    runtime::{Runtime, Screen},
    services::{Command, Environment, NewIwad, Services},
};
use ddl_core::{
    downloads::CancellationToken, iwads::Game, library::InstallManifest, local::Compatibility,
};

/// Installed mapsets of the owner-like library.
const INSTALLED: usize = 7;

fn environment(home: &std::path::Path) -> Environment {
    std::fs::create_dir_all(home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    Environment {
        paths,
        home: home.into(),
        test: None,
    }
    .without_catalog_auto_updates()
}

/// A runtime whose snapshot holds `INSTALLED` installed mapsets, all base
/// games and two custom setups.
fn owner_runtime(temp: &std::path::Path, ui: &AppWindow) -> Runtime {
    let env = environment(&temp.join("home"));
    let base = env.home.join("base.wad");
    std::fs::write(&base, include_bytes!("../tests/fixtures/local/base.wad")).unwrap();
    // A WAD with E1M1–E4M9 lumps is identified as a DOOM base game.
    let owned = env.home.join("owned");
    std::fs::create_dir_all(&owned).unwrap();
    let mut wad = b"IWAD".to_vec();
    wad.extend_from_slice(&36i32.to_le_bytes());
    wad.extend_from_slice(&12i32.to_le_bytes());
    for episode in 1..=4 {
        for map in 1..=9 {
            wad.extend_from_slice(&[0; 8]);
            let mut name = [0; 8];
            let label = format!("E{episode}M{map}");
            name[..label.len()].copy_from_slice(label.as_bytes());
            wad.extend_from_slice(&name);
        }
    }
    std::fs::write(owned.join("doom.wad"), wad).unwrap();
    let mut services = Services::open(env.clone()).unwrap();
    let token = CancellationToken::new();
    services
        .execute(Command::Startup, &token, &mut |_| {})
        .unwrap();
    services
        .execute(Command::Scan(Some(owned.clone())), &token, &mut |_| {})
        .unwrap();
    // The registry keeps the canonical path (macOS temp dirs are symlinks).
    let registered = services.snapshot().unwrap().iwads[0].path.clone();
    services
        .execute(
            Command::Choose(registered, "doom-1.9ud".into()),
            &token,
            &mut |_| {},
        )
        .unwrap();
    for (name, id) in [("Night maps", "night-maps"), ("Speed runs", "speed-runs")] {
        services
            .custom_create(
                name,
                Some(id),
                NewIwad::File(base.clone()),
                Compatibility::Auto,
            )
            .unwrap();
    }
    let mut snapshot = services.snapshot().unwrap();
    assert!(
        snapshot.items.len() >= INSTALLED,
        "the embedded catalog is large enough"
    );
    for item in snapshot.items.iter_mut().take(INSTALLED) {
        item.manifest = Some(InstallManifest {
            schema_version: 1,
            recipe_id: item.recipe.id.clone(),
            release: item.recipe.release.clone(),
            recipe_sha256: "a".repeat(64),
            installed_at: 0,
            packages: vec![],
            members: vec![],
        });
    }
    // The other three base games reuse the one registered copy.
    let doom = snapshot
        .base_games
        .first()
        .cloned()
        .expect("a registered base game");
    for game in [Game::Doom2, Game::Tnt, Game::Plutonia] {
        let mut item = doom.clone();
        item.base.game = game;
        snapshot.base_games.push(item);
    }
    assert_eq!(snapshot.base_games.len(), 4, "the four base games");
    assert_eq!(snapshot.custom.len(), 2);
    let mut runtime = Runtime::new(env);
    runtime.snapshot = Some(snapshot);
    runtime.screen = Screen::Library;
    runtime.focus = -2;
    runtime.sync(ui);
    runtime
}

#[test]
fn library_owner_scene_renders() {
    if isolated(module_path!(), "library_owner_scene_renders") {
        return;
    }
    use slint::{Model, ModelRc, VecModel};
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    let offset = ui.get_scroll_offset();
    // The gamepad timer invokes scroll even with a neutral stick, before Slint
    // has evaluated content-y. It must not erase a newly assigned focus offset.
    ui.invoke_scroll(0., 0.);
    assert_eq!(
        ui.get_scroll_offset(),
        offset,
        "neutral tick preserves focus-follow"
    );
    let rest = render(&window);
    write_png("061-library-rest.png", &rest);
    let nodes: Vec<_> = ui.get_nodes().iter().collect();
    let heading = nodes
        .iter()
        .find(|n| n.label == "Mapsets and custom setups")
        .unwrap();
    assert_eq!(
        heading.y - offset,
        -4.,
        "section heading at the normal top margin"
    );
    let cards: Vec<_> = nodes
        .iter()
        .filter(|n| n.actionable && n.kind == 1 && n.y == 298.)
        .collect();
    assert_eq!(cards.len(), 3, "three full-width columns");
    assert_eq!(
        cards.iter().map(|n| n.x).collect::<Vec<_>>(),
        [12., 420., 828.]
    );
    ui.set_nodes(ModelRc::new(VecModel::from(Vec::<crate::ViewNode>::new())));
    let background = render(&window);
    // Compare against the same wall and header with the content removed. This
    // detects actual rendered rails, including SVG frame pixels, rather than
    // relying on a particular bronze RGB value.
    for y in 210..218 {
        for x in 20..1260 {
            let index = y * WIDTH as usize + x;
            assert_eq!(
                rest.as_slice()[index],
                background.as_slice()[index],
                "stray frame at {x},{y}"
            );
        }
    }
    for x in [40, 448, 856] {
        let changed = (270..670)
            .flat_map(|y| (x..x + 380).map(move |x| y * WIDTH as usize + x))
            .filter(|&i| rest.as_slice()[i] != background.as_slice()[i])
            .count();
        assert!(changed > 100_000, "rendered card fills column at {x}");
    }
    ui.set_nodes(ModelRc::new(VecModel::from(nodes)));
    for (name, focus) in [("row-two", 13), ("row-three", 16)] {
        r.focus = focus;
        r.sync(&ui);
        let node = ui.get_nodes().iter().find(|n| n.id == focus).unwrap();
        assert_eq!(
            ui.get_scroll_offset(),
            node.y - 8.,
            "later rows align whole cards"
        );
        write_png(&format!("061-library-{name}.png"), &render(&window));
    }
    let focus = r.focus;
    let before = ui.get_scroll_offset();
    ui.invoke_scroll(0., -80.);
    assert_eq!(ui.get_scroll_offset(), before - 80.);
    r.sync(&ui);
    assert_eq!(r.focus, focus);
    assert_eq!(
        ui.get_scroll_offset(),
        before - 80.,
        "service refresh preserves manual scroll"
    );
    let scrolled = render(&window);
    write_png("061-library-stick.png", &scrolled);
    // All frame pieces stay inside the plane, including on manual partial rows.
    for y in (0..210).chain(732..800) {
        for x in 0..WIDTH as usize {
            let i = y * WIDTH as usize + x;
            assert_eq!(
                scrolled.as_slice()[i],
                rest.as_slice()[i],
                "clipped frame escaped at {x},{y}"
            );
        }
    }
    for (screen, name) in [(Screen::Home, "home"), (Screen::Discover, "discover")] {
        r.screen = screen;
        r.focus = 1;
        r.sync(&ui);
        write_png(&format!("061-{name}-rest.png"), &render(&window));
    }
}
