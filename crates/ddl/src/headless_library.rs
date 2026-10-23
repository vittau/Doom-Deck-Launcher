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
        screen: None,
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
    runtime.focus = runtime
        .snapshot
        .as_ref()
        .unwrap()
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.manifest.is_some())
        .min_by(|(_, a), (_, b)| ddl_core::catalog::compare_browsing_order(&a.recipe, &b.recipe))
        .map_or(6300, |(index, _)| 10 + index as i32);
    runtime.sync(ui);
    runtime
}

/// Discover's expected card ids for `tag` (spec 112, R1): the
/// `(installed, favorite)` tuple descending, with chronology inside each group.
fn expected_discover(items: &[crate::services::Item], tag: &str) -> Vec<i32> {
    let mut shown: Vec<usize> = (0..items.len())
        .filter(|&i| ddl_core::catalog::shows_in_category(&items[i].recipe, tag))
        .collect();
    shown.sort_by(|&a, &b| {
        let (a, b) = (&items[a], &items[b]);
        ddl_core::catalog::compare_discover_order(
            &a.recipe,
            a.manifest.is_some(),
            a.favorite,
            &b.recipe,
            b.manifest.is_some(),
            b.favorite,
        )
    });
    shown.iter().map(|&i| 10 + i as i32).collect()
}

/// Card (Cards) or row (List) ids in the order the plane draws them.
fn drawn_ids(ui: &AppWindow, list: bool) -> Vec<i32> {
    let kind = if list { 24 } else { 1 };
    ui.get_nodes()
        .iter()
        .filter(|n| n.actionable && n.kind == kind)
        .map(|n| n.id)
        .collect()
}

/// One mapset per group in the megawad category, in chronological order: an
/// installed favorite, an installed mapset, a favorite-only mapset and the rest.
/// Returns the three leading group indexes plus the last chronological item of
/// the category, which sits in the trailing group.
fn mark_discover_groups(runtime: &mut Runtime) -> (usize, usize, usize, usize) {
    let snapshot = runtime.snapshot.as_mut().unwrap();
    let mut indexes: Vec<usize> = (0..snapshot.items.len())
        .filter(|&i| ddl_core::catalog::shows_in_category(&snapshot.items[i].recipe, "megawad"))
        .collect();
    assert!(indexes.len() >= 6, "megawad has several catalog recipes");
    indexes.sort_by(|&a, &b| {
        ddl_core::catalog::compare_browsing_order(
            &snapshot.items[a].recipe,
            &snapshot.items[b].recipe,
        )
    });
    for item in &mut snapshot.items {
        item.favorite = false;
        item.manifest = None;
    }
    for &index in &indexes[..2] {
        let item = &mut snapshot.items[index];
        let (id, release) = (item.recipe.id.clone(), item.recipe.release.clone());
        item.manifest = Some(InstallManifest {
            schema_version: 1,
            recipe_id: id,
            release,
            recipe_sha256: "a".repeat(64),
            installed_at: 0,
            packages: vec![],
            members: vec![],
        });
    }
    snapshot.items[indexes[0]].favorite = true;
    snapshot.items[indexes[2]].favorite = true;
    (
        indexes[0],
        indexes[1],
        indexes[2],
        indexes[indexes.len() - 1],
    )
}

/// Spec 112 R1/R4: each Discover category, in Cards and List, leads with an
/// installed favorite, an installed mapset, a favorite-only mapset and then the
/// rest, chronologically inside each group, rendered from real catalog recipes.
#[test]
fn discover_installed_favorites_first_in_both_layouts() {
    if isolated(
        module_path!(),
        "discover_installed_favorites_first_in_both_layouts",
    ) {
        return;
    }
    use crate::runtime::Category;
    use crate::services::ViewFormat;
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    let (installed_favorite, installed, favorite, _) = mark_discover_groups(&mut r);
    r.screen = Screen::Discover;
    r.choose_category(Category::Tag("megawad".into()));
    for (format, name) in [
        (ViewFormat::Cards, "112-discover-groups-cards.png"),
        (ViewFormat::List, "112-discover-groups-list.png"),
    ] {
        r.view_format = format;
        r.focus = 10;
        r.sync(&ui);
        advance_time(20);
        let shown = drawn_ids(&ui, format == ViewFormat::List);
        assert_eq!(
            shown,
            expected_discover(&r.snapshot.as_ref().unwrap().items, "megawad"),
            "{format:?} follows the four groups"
        );
        // The first three slots are the three leading groups; the grid shows
        // them side by side on its first row.
        assert_eq!(
            &shown[..3],
            &[
                10 + installed_favorite as i32,
                10 + installed as i32,
                10 + favorite as i32,
            ],
            "{format:?} leads with installed favorite, installed and favorite"
        );
        let frame = render(&window);
        let path = write_png(name, &frame);
        assert!(path.is_file());
        // Four groups on screen: the grid/rows are not truncated to one group.
        assert!(shown.len() > 3);
    }
}

/// Spec 112 R2: toggling X moves the row into its favorite group in both
/// layouts without losing or duplicating rows, keeps focus on the same mapset,
/// and returning from Mapset detail keeps that selection.
#[test]
fn discover_favorite_toggle_moves_the_row_and_keeps_focus() {
    if isolated(
        module_path!(),
        "discover_favorite_toggle_moves_the_row_and_keeps_focus",
    ) {
        return;
    }
    use crate::{input::Action, runtime::Category, services::ViewFormat};
    let (_window, ui) = setup();
    ui.show().unwrap();
    for (format, list) in [(ViewFormat::Cards, false), (ViewFormat::List, true)] {
        let temp = tempfile::tempdir().unwrap();
        let mut r = owner_runtime(temp.path(), &ui);
        let (_, _, _, rest) = mark_discover_groups(&mut r);
        r.screen = Screen::Discover;
        r.choose_category(Category::Tag("megawad".into()));
        r.view_format = format;
        let target = 10 + rest as i32;
        r.focus = target;
        r.sync(&ui);
        let before = drawn_ids(&ui, list);
        assert_eq!(
            before,
            expected_discover(&r.snapshot.as_ref().unwrap().items, "megawad"),
            "{format:?} starts in the four groups"
        );
        r.action(&ui, Action::X);
        assert_eq!(r.focus, target, "{format:?} keeps focus on the same card");
        assert!(
            r.snapshot.as_ref().unwrap().items[rest].favorite,
            "{format:?} marks the favorite optimistically"
        );
        let after = drawn_ids(&ui, list);
        assert_eq!(
            after,
            expected_discover(&r.snapshot.as_ref().unwrap().items, "megawad"),
            "{format:?} re-sorts with the new favorite group"
        );
        assert_eq!(after.len(), before.len(), "{format:?} loses no row");
        assert_eq!(
            after.iter().filter(|&&id| id == target).count(),
            1,
            "{format:?} duplicates no row"
        );
        assert!(
            after.iter().position(|&id| id == target) < before.iter().position(|&id| id == target),
            "{format:?} moves the favorite up"
        );
        // Settle the saved favorite: the fresh snapshot keeps recipe identity.
        for _ in 0..1000 {
            pump_events();
            r.drain(&ui);
            if !r.busy {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!r.busy && r.problem.is_none());
        assert!(
            r.snapshot.as_ref().unwrap().items[rest].favorite,
            "{format:?} persists the favorite"
        );
        assert_eq!(r.focus, target, "{format:?} keeps focus after the save");
        // Return from Mapset detail keeps the same selected mapset.
        r.action(&ui, Action::Confirm);
        assert_eq!(r.screen, Screen::Detail);
        assert_eq!(r.selected, rest);
        r.action(&ui, Action::Back);
        assert_eq!(r.screen, Screen::Discover);
        assert_eq!(r.focus, target, "{format:?} return keeps the selection");
    }
}

/// Spec 112 R2/R4: a finished production install moves the mapset into the
/// installed group in Cards and List, leaving Play setup keeps it focused, and a
/// finished production uninstall moves it back out with its row identity intact.
///
/// The post-uninstall navigation stays as spec 022 defined it (Library, card
/// focus, cleared history); the coordinator confirmed spec 112 R2 applies to the
/// Discover ordering and row identity for uninstalls, not to that flow.
#[test]
fn discover_install_and_uninstall_move_the_item_and_keep_focus() {
    if isolated(
        module_path!(),
        "discover_install_and_uninstall_move_the_item_and_keep_focus",
    ) {
        return;
    }
    use crate::input::Action;
    use crate::runtime::Category;
    use crate::services::ViewFormat;
    let scene = super::production::scene();
    let ui = &scene.ui;
    let state = scene.state.clone();
    super::production::idle(&state);
    let (sigil, auger) = {
        let mut r = state.borrow_mut();
        r.screen = Screen::Discover;
        r.choose_category(Category::Tag("cacowards".into()));
        r.view_format = ViewFormat::Cards;
        let items = &r.snapshot.as_ref().unwrap().items;
        let index = |id: &str| items.iter().position(|i| i.recipe.id == id).unwrap();
        (
            10 + index("sigil") as i32,
            10 + index("auger-zenith") as i32,
        )
    };
    // A favorite that is not installed leads the category.
    {
        let mut r = state.borrow_mut();
        r.focus = auger;
        r.sync(ui);
        r.action(ui, Action::X);
    }
    super::production::idle(&state);
    assert_eq!(state.borrow().focus, auger, "the toggle keeps the mapset");
    assert_eq!(drawn_ids(ui, false)[0], auger, "the favorite leads Cards");
    // A real install of SIGIL moves it ahead of that favorite.
    {
        let mut r = state.borrow_mut();
        r.focus = sigil;
        r.sync(ui);
        r.start(ui, Command::Install("sigil".into()));
    }
    super::production::wait(&state, |r| r.progress.is_some() || !r.busy);
    scene.package_gate.release();
    super::production::idle(&state);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert!(
        state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .find(|i| i.recipe.id == "sigil")
            .unwrap()
            .manifest
            .is_some(),
        "the install finished"
    );
    // Back leaves Play setup for the same Discover mapset, now installed.
    state.borrow_mut().action(ui, Action::Back);
    assert_eq!(state.borrow().screen, Screen::Discover);
    assert_eq!(state.borrow().focus, sigil, "focus keeps the mapset");
    for (format, list) in [(ViewFormat::Cards, false), (ViewFormat::List, true)] {
        let mut r = state.borrow_mut();
        r.view_format = format;
        r.sync(ui);
        let shown = drawn_ids(ui, list);
        assert_eq!(
            shown,
            expected_discover(&r.snapshot.as_ref().unwrap().items, "cacowards"),
            "{format:?} keeps the four groups"
        );
        assert_eq!(shown[0], sigil, "{format:?}: installed leads favorite-only");
        assert_eq!(shown[1], auger, "{format:?}: the favorite follows");
    }
    // Favorite the installed mapset: installed favorite leads both layouts.
    {
        let mut r = state.borrow_mut();
        r.focus = sigil;
        r.sync(ui);
        r.action(ui, Action::X);
    }
    super::production::idle(&state);
    assert_eq!(state.borrow().focus, sigil, "the toggle keeps the mapset");
    let before_uninstall = {
        let mut r = state.borrow_mut();
        let mut shown = Vec::new();
        for (format, list) in [(ViewFormat::Cards, false), (ViewFormat::List, true)] {
            r.view_format = format;
            r.sync(ui);
            let drawn = drawn_ids(ui, list);
            assert_eq!(
                drawn,
                expected_discover(&r.snapshot.as_ref().unwrap().items, "cacowards"),
                "{format:?} leads with the installed favorite"
            );
            assert_eq!(drawn[0], sigil, "{format:?}: installed favorite leads");
            shown.push(drawn);
        }
        shown
    };
    // A real uninstall moves it out of the installed group. Spec 022 keeps the
    // Library destination; the row and the selection keep their identity.
    {
        let mut r = state.borrow_mut();
        r.view_format = ViewFormat::Cards;
        r.selected = r
            .snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .position(|i| i.recipe.id == "sigil")
            .unwrap();
        // The production path opens Mapset detail before its uninstall action.
        r.screen = Screen::Detail;
        r.focus = 1;
        r.sync(ui);
        r.start(ui, Command::Uninstall("sigil".into(), false));
    }
    super::production::idle(&state);
    assert_eq!(state.borrow().screen, Screen::Library);
    let selected = state.borrow().selected;
    let sigil_index = {
        let r = state.borrow();
        let items = &r.snapshot.as_ref().unwrap().items;
        let index = items.iter().position(|i| i.recipe.id == "sigil").unwrap();
        assert!(
            items[index].manifest.is_none(),
            "the uninstall finished and keeps the row"
        );
        assert!(items[index].favorite, "the favorite survives the uninstall");
        assert_eq!(
            r.snapshot.as_ref().unwrap().items[selected].recipe.id,
            "sigil",
            "the selection keeps the mapset"
        );
        index
    };
    // Opening Discover with that mapset focused shows it in its new group, once.
    for (format, list) in [(ViewFormat::Cards, false), (ViewFormat::List, true)] {
        let mut r = state.borrow_mut();
        r.screen = Screen::Discover;
        r.view_format = format;
        r.focus = 10 + sigil_index as i32;
        r.sync(ui);
        let drawn = drawn_ids(ui, list);
        assert_eq!(
            drawn,
            expected_discover(&r.snapshot.as_ref().unwrap().items, "cacowards"),
            "{format:?} re-sorts after the uninstall"
        );
        assert_eq!(
            drawn.iter().filter(|&&id| id == r.focus).count(),
            1,
            "{format:?} draws the uninstalled mapset once"
        );
        assert_eq!(
            drawn.len(),
            before_uninstall[if list { 1 } else { 0 }].len(),
            "{format:?} loses no row"
        );
    }
}

#[test]
fn library_owner_scene_renders() {
    if isolated(module_path!(), "library_owner_scene_renders") {
        return;
    }
    use slint::{Model, ModelRc, VecModel};
    let (window, ui) = setup();
    ui.show().unwrap();
    let capture = || {
        advance_time(20);
        render(&window)
    };
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    let offset = ui.get_scroll_offset();
    let rest = capture();
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
        .filter(|n| n.actionable && n.kind == 1 && n.y == 322.)
        .collect();
    assert_eq!(cards.len(), 3, "three full-width columns");
    assert_eq!(
        cards.iter().map(|n| n.x).collect::<Vec<_>>(),
        [12., 420., 828.]
    );
    ui.set_nodes(ModelRc::new(VecModel::from(Vec::<crate::ViewNode>::new())));
    let background = capture();
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
    r.focus = 6300; // First base game, at the absolute top of the Library.
    r.sync(&ui);
    assert_eq!(ui.get_scroll_offset(), 0.);
    write_png("061-library-top.png", &capture());
    // Spec 090 R2: the grid is in release order, so pick the first card of the
    // second and third displayed rows by position, not by a fixed snapshot id.
    let displayed: Vec<i32> = ui
        .get_nodes()
        .iter()
        .filter(|n| n.kind == 1 && n.actionable && n.w >= 380.)
        .map(|n| n.id)
        .collect();
    for (name, index) in [("row-two", 3usize), ("row-three", 6usize)] {
        let focus = displayed[index];
        r.focus = focus;
        r.sync(&ui);
        let node = ui.get_nodes().iter().find(|n| n.id == focus).unwrap();
        assert_eq!(
            ui.get_scroll_offset(),
            node.y - 8.,
            "later rows align whole cards"
        );
        write_png(&format!("061-library-{name}.png"), &capture());
        advance_time(20);
        let offset = ui.get_scroll_offset();
        assert_eq!(
            offset,
            node.y - 8.,
            "Flickable preserves the whole-row landing after rendering"
        );
        ui.invoke_scroll(0., 0.);
        assert_eq!(
            ui.get_scroll_offset(),
            offset,
            "neutral poll preserves the whole-row landing"
        );
    }
    let focus = r.focus;
    let before = ui.get_scroll_offset();
    ui.invoke_scroll(0., -160.);
    assert_eq!(ui.get_scroll_offset(), before - 160.);
    r.sync(&ui);
    assert_eq!(r.focus, focus);
    assert_eq!(
        ui.get_scroll_offset(),
        before - 160.,
        "service refresh preserves manual scroll"
    );
    let scrolled = capture();
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
    r.focus = 6101;
    r.sync(&ui);
    write_png("061-library-custom-setup.png", &capture());
    for (screen, name) in [(Screen::Home, "home"), (Screen::Discover, "discover")] {
        r.screen = screen;
        r.focus = 1;
        r.sync(&ui);
        assert_eq!(
            ui.get_active_tab(),
            if screen == Screen::Home { 0 } else { 2 }
        );
        write_png(&format!("061-{name}-rest.png"), &capture());
    }
}

#[test]
fn neutral_stick_tick_preserves_each_plane_offset() {
    if isolated(
        module_path!(),
        "neutral_stick_tick_preserves_each_plane_offset",
    ) {
        return;
    }
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let _runtime = owner_runtime(temp.path(), &ui);
    let _ = render(&window);
    advance_time(20);
    let offset = ui.get_scroll_offset() + 20.;
    ui.set_scroll_offset(offset);
    // A real event-loop tick evaluates the prior change handler. The next
    // neutral poll must not read the previous content-y before the new handler.
    ui.invoke_scroll(0., 0.);
    assert_eq!(
        ui.get_scroll_offset(),
        offset,
        "main offset survives a neutral tick"
    );
    ui.invoke_scroll(0., 12.);
    assert_eq!(
        ui.get_scroll_offset(),
        offset + 12.,
        "manual tick starts at the new focus offset"
    );
    ui.set_setup_content_height(2000.);
    ui.set_sheet_height(1400.);
    ui.set_setup_screen(true);
    ui.set_setup_right_focused(true);
    ui.set_setup_scroll_offset(240.);
    ui.invoke_scroll(0., 0.);
    assert_eq!(
        ui.get_setup_scroll_offset(),
        240.,
        "summary offset survives a neutral tick"
    );
    ui.set_sheet_open(true);
    let _ = render(&window);
    ui.set_sheet_scroll_offset(280.);
    ui.invoke_scroll(0., 0.);
    assert_eq!(
        ui.get_sheet_scroll_offset(),
        280.,
        "sheet offset survives a neutral tick"
    );
}

#[test]
fn library_base_game_focus_survives_refresh_and_dpad_navigation() {
    if isolated(
        module_path!(),
        "library_base_game_focus_survives_refresh_and_dpad_navigation",
    ) {
        return;
    }
    use crate::input::Action;
    let (_window, ui) = setup();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    r.focus = 6300;
    r.sync(&ui);
    assert_eq!(
        r.focus, 6300,
        "base-game focus survives a full node rebuild"
    );
    // Replace the service snapshot just as a refresh does, retaining the
    // installed mapsets, four base games and custom setups in this fixture.
    let mut refreshed = r.snapshot.as_ref().unwrap().clone();
    refreshed.items[0].profile.name.push_str(" refreshed");
    r.snapshot = Some(refreshed);
    r.sync(&ui);
    assert_eq!(r.focus, 6300, "base-game focus survives snapshot refresh");
    assert_eq!(ui.get_focused(), 6300);
    r.focus = 6000;
    r.sync(&ui);
    r.action(&ui, Action::Down);
    assert!(
        (6300..=6303).contains(&r.focus),
        "Down enters the base-game row"
    );
    for _ in 0..3 {
        if r.focus == 6300 {
            break;
        }
        r.action(&ui, Action::Left);
    }
    assert_eq!(r.focus, 6300, "Left reaches the first base game");
    for id in 6301..=6303 {
        r.action(&ui, Action::Right);
        assert_eq!(r.focus, id, "Right reaches every base game");
        r.sync(&ui);
        assert_eq!(r.focus, id, "focus remains on the selected base game");
    }
    for id in (6300..=6302).rev() {
        r.action(&ui, Action::Left);
        assert_eq!(r.focus, id, "Left traverses the base-game row");
    }
    r.action(&ui, Action::Down);
    // Spec 090 R2: the grid is in release order, so the first displayed card is
    // the oldest installed mapset, whose id is CARD + its *snapshot* index (not
    // necessarily 10). Down from the leftmost base game must land on that card.
    let oldest = {
        let snapshot = r.snapshot.as_ref().unwrap();
        snapshot
            .items
            .iter()
            .filter(|item| item.manifest.is_some())
            .min_by(|a, b| ddl_core::catalog::compare_browsing_order(&a.recipe, &b.recipe))
            .map(|item| item.recipe.id.clone())
            .expect("the fixture installs mapsets")
    };
    let oldest_card = {
        let snapshot = r.snapshot.as_ref().unwrap();
        10 + snapshot
            .items
            .iter()
            .position(|item| item.recipe.id == oldest)
            .expect("the oldest installed mapset is in the snapshot") as i32
    };
    assert_eq!(
        r.focus, oldest_card,
        "Down reaches the oldest installed mapset, the first displayed card"
    );
    let top_left_card = ui
        .get_nodes()
        .iter()
        .filter(|node| node.kind == 1 && node.actionable && (10..=199).contains(&node.id))
        .min_by(|a, b| {
            (a.y, a.x)
                .partial_cmp(&(b.y, b.x))
                .expect("finite card geometry")
        })
        .map(|node| node.id)
        .expect("the installed grid is laid out");
    assert_eq!(
        r.focus, top_left_card,
        "the landed card is the top-left card under the base-game row"
    );
    r.action(&ui, Action::Up);
    assert!((6300..=6303).contains(&r.focus), "Up returns to base games");
    r.action(&ui, Action::Up);
    assert_eq!(r.focus, 6000, "Up returns to New custom setup");
}

/// Production layouts and keyboard input, with no native backend or desktop capture.
#[test]
fn compact_view_scenes_and_identity_journey() {
    if isolated(module_path!(), "compact_view_scenes_and_identity_journey") {
        return;
    }
    use crate::{input::Action, runtime::Category, services::ViewFormat};
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let runtime = owner_runtime(temp.path(), &ui);
    let state = Rc::new(std::cell::RefCell::new(runtime));
    let callback = state.clone();
    let weak = ui.as_weak();
    ui.on_action(move |code| {
        if let Some(action) = Action::from_code(code) {
            callback
                .borrow_mut()
                .action(&weak.upgrade().unwrap(), action);
        }
    });
    let capture = |name: &str| {
        advance_time(20);
        write_png(name, &render(&window));
    };
    {
        let mut r = state.borrow_mut();
        for item in &mut r.snapshot.as_mut().unwrap().items {
            item.favorite = true;
        }
        r.snapshot.as_mut().unwrap().items[1].recipe.title = "The extraordinarily long mapset title across the forgotten cathedral at the edge of the universe".into();
        r.choose_category(Category::Favorites);
        r.screen = Screen::Discover;
        r.focus = 11;
        r.sync(&ui);
    }
    capture("095-discover-cards.png");
    window.dispatch_event(WindowEvent::KeyPressed { text: "v".into() });
    assert_eq!(state.borrow().view_format, ViewFormat::List);
    window.dispatch_event(WindowEvent::KeyPressed { text: "v".into() });
    assert_eq!(
        state.borrow().view_format,
        ViewFormat::List,
        "held keyboard V cannot repeat"
    );
    window.dispatch_event(WindowEvent::KeyReleased { text: "v".into() });
    assert_eq!(state.borrow().focus, 11);
    let first_row = ui
        .get_nodes()
        .iter()
        .filter(|n| n.kind == 24)
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap()
        .id;
    state.borrow_mut().focus = first_row;
    state.borrow_mut().sync(&ui);
    capture("095-discover-list-category.png");
    let offset = ui.get_scroll_offset();
    let first = ui.get_nodes().iter().find(|n| n.id == first_row).unwrap();
    assert!(first.y >= offset && first.y + first.h + 16. <= offset + 502.);
    assert!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 24 && n.y + 8. >= offset && n.y + n.h + 8. - offset <= 502.)
            .count()
            >= 5
    );
    state.borrow_mut().focus = 11;
    state.borrow_mut().sync(&ui);
    capture("095-discover-list-long-title.png");
    let offset = ui.get_scroll_offset();
    assert!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 24 && n.y - offset >= 0. && n.y + n.h - offset <= 502.)
            .count()
            >= 5
    );
    {
        let mut r = state.borrow_mut();
        r.focus = 10 + (r.snapshot.as_ref().unwrap().items.len() - 1) as i32;
        r.sync(&ui);
    }
    assert!(ui.get_scroll_offset() > 0.);
    capture("095-discover-list-scrolled.png");
    let focused = state.borrow().focus;
    for _ in 0..2 {
        state.borrow_mut().action(&ui, Action::ToggleView);
        assert_eq!(state.borrow().focus, focused);
    }
    state.borrow_mut().action(&ui, Action::Confirm);
    assert_eq!(state.borrow().screen, Screen::Detail);
    assert!(!ui.get_view_hint());
    state.borrow_mut().action(&ui, Action::ToggleView);
    assert_eq!(state.borrow().view_format, ViewFormat::List);
    state.borrow_mut().action(&ui, Action::Back);
    assert_eq!(state.borrow().focus, focused);
    state.borrow_mut().action(&ui, Action::Tab(1));
    assert_eq!(state.borrow().screen, Screen::Library);
    assert!(ui.get_list_view());
    state.borrow_mut().focus = 6300;
    state.borrow_mut().sync(&ui);
    capture("095-library-list-mixed.png");
    state.borrow_mut().focus = 6303;
    state.borrow_mut().sync(&ui);
    ui.set_scroll_offset(260.);
    capture("095-library-list-section-transition.png");
    state.borrow_mut().focus = 6300;
    state.borrow_mut().sync(&ui);
    state.borrow_mut().action(&ui, Action::ToggleView);
    assert_eq!(state.borrow().focus, 6300);
    capture("095-library-cards-mixed.png");
    state.borrow_mut().action(&ui, Action::ToggleView);
    state.borrow_mut().focus = 6101;
    state.borrow_mut().sync(&ui);
    capture("095-library-list-local-scrolled.png");
    state.borrow_mut().action(&ui, Action::ToggleView);
    assert_eq!(state.borrow().focus, 6101);
    state.borrow_mut().action(&ui, Action::ToggleView);
    state.borrow_mut().action(&ui, Action::Menu);
    assert!(!ui.get_view_hint());
    let format = state.borrow().view_format;
    state.borrow_mut().action(&ui, Action::ToggleView);
    assert_eq!(state.borrow().view_format, format);
    state.borrow_mut().action(&ui, Action::Back);
    state.borrow_mut().focus = -2;
    state.borrow_mut().sync(&ui);
    state.borrow_mut().action(&ui, Action::ToggleView);
    assert_eq!(state.borrow().focus, -2, "tab focus survives");
    state.borrow_mut().focus = 6000;
    state.borrow_mut().sync(&ui);
    state.borrow_mut().action(&ui, Action::ToggleView);
    assert_eq!(state.borrow().focus, 6000, "setup control survives");
    state.borrow_mut().action(&ui, Action::Confirm);
    assert!(!ui.get_view_hint(), "local setup excludes toggle");
    state.borrow_mut().finish_preferences();
    let env = environment(&temp.path().join("home"));
    assert_eq!(
        crate::services::load_view_format(&env).unwrap(),
        state.borrow().view_format
    );
}

#[test]
fn compact_thousand_items_repeat_without_reload_or_retained_models() {
    if isolated(
        module_path!(),
        "compact_thousand_items_repeat_without_reload_or_retained_models",
    ) {
        return;
    }
    use crate::{input::Action, runtime::Category, services::ViewFormat};
    let (_window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    let prototype = r.snapshot.as_ref().unwrap().items[0].clone();
    r.snapshot.as_mut().unwrap().items = (0..1000)
        .map(|index| {
            let mut item = prototype.clone();
            item.recipe.id = format!("synthetic-{index:04}");
            item.recipe.title = format!("Mapset {index:04}");
            item.favorite = true;
            item
        })
        .collect();
    r.choose_category(Category::Favorites);
    r.screen = Screen::Discover;
    r.focus = 1_000_777;
    r.sync(&ui);
    let items = r.snapshot.as_ref().unwrap().items.as_ptr();
    let mut models = vec![];
    for _ in 0..24 {
        let model: Rc<dyn slint::Model<Data = crate::ViewNode>> =
            ui.get_nodes().try_into().unwrap();
        models.push(Rc::downgrade(&model));
        drop(model);
        r.action(&ui, Action::ToggleView);
        assert_eq!(r.focus, 1_000_777);
        assert_eq!(
            r.snapshot.as_ref().unwrap().items.as_ptr(),
            items,
            "reuse the same snapshot data"
        );
        if r.view_format == ViewFormat::List {
            assert!(
                ui.get_nodes().iter().all(|n| n.screenshot.is_empty()),
                "compact rows never request screenshots"
            );
        }
        assert!(
            r.pending.is_none() && !r.busy,
            "layout never starts a catalog/library command"
        );
        assert!(ui.get_scroll_offset() > 0.);
        let focus = ui.get_nodes().iter().find(|n| n.id == r.focus).unwrap();
        assert!(focus.y - ui.get_scroll_offset() >= 0.);
        assert!(focus.y + focus.h - ui.get_scroll_offset() <= 502.);
        assert!(ui.get_nodes().row_count() < 11020);
        advance_time(1);
        assert!(
            models.iter().all(|old| old.upgrade().is_none()),
            "replaced row models are released"
        );
    }
    r.action(&ui, Action::ToggleView);
    assert_eq!(r.view_format, ViewFormat::List);
    r.action(&ui, Action::Down);
    assert_ne!(r.focus, 1_000_777);
    r.action(&ui, Action::Up);
    assert_eq!(r.focus, 1_000_777);
    r.action(&ui, Action::Confirm);
    assert_eq!(r.selected, 777);
    assert_eq!(r.screen, Screen::Detail);
    r.action(&ui, Action::Back);
    assert_eq!(r.focus, 1_000_777);
    r.action(&ui, Action::Tab(1));
    assert_eq!(r.view_format, ViewFormat::List);
}

#[test]
fn compact_categories_favorites_and_shared_root_journey() {
    if isolated(
        module_path!(),
        "compact_categories_favorites_and_shared_root_journey",
    ) {
        return;
    }
    use crate::{input::Action, runtime::Category, services::ViewFormat};
    let (_window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    r.screen = Screen::Discover;
    r.choose_category(Category::Tag("doomer-boards".into()));
    r.sync(&ui);
    let cards: Vec<_> = ui
        .get_nodes()
        .iter()
        .filter(|n| n.actionable && n.kind == 1)
        .collect();
    assert!(cards.len() > 5);
    r.focus = cards[4].id;
    r.sync(&ui);
    let id = r.snapshot.as_ref().unwrap().items[(r.focus - 10) as usize]
        .recipe
        .id
        .clone();
    let order = |ui: &AppWindow| {
        ui.get_nodes()
            .iter()
            .filter(|n| n.actionable && (n.kind == 1 || n.kind == 24))
            .map(|n| n.id)
            .collect::<Vec<_>>()
    };
    let before = order(&ui);
    let category = ui
        .get_nodes()
        .iter()
        .find(|n| n.kind == 14 && n.primary)
        .unwrap()
        .label;
    r.action(&ui, Action::ToggleView);
    assert_eq!(order(&ui), before);
    assert_eq!(
        ui.get_nodes()
            .iter()
            .find(|n| n.kind == 14 && n.primary)
            .unwrap()
            .label,
        category
    );
    r.action(&ui, Action::X);
    for _ in 0..1000 {
        pump_events();
        r.drain(&ui);
        if !r.busy {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!r.busy && r.problem.is_none());
    assert_eq!(
        r.snapshot.as_ref().unwrap().items[(r.focus - 10) as usize]
            .recipe
            .id,
        id
    );
    assert!(
        r.snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .find(|item| item.recipe.id == id)
            .unwrap()
            .favorite
    );
    assert_eq!(r.view_format, ViewFormat::List);
    r.action(&ui, Action::Confirm);
    assert_eq!(r.screen, Screen::Detail);
    assert_eq!(r.snapshot.as_ref().unwrap().items[r.selected].recipe.id, id);
    r.action(&ui, Action::Back);
    r.action(&ui, Action::ToggleView);
    assert_eq!(
        r.snapshot.as_ref().unwrap().items[(r.focus - 10) as usize]
            .recipe
            .id,
        id
    );
    r.action(&ui, Action::NextCategory);
    assert_ne!(
        ui.get_nodes()
            .iter()
            .find(|n| n.kind == 14 && n.primary)
            .unwrap()
            .label,
        category
    );
    r.action(&ui, Action::Up);
    assert!((200..230).contains(&r.focus));
    let chip = r.focus;
    r.action(&ui, Action::ToggleView);
    assert_eq!(r.focus, chip);
    r.action(&ui, Action::Tab(1));
    assert_eq!(r.view_format, ViewFormat::List);
    r.focus = 6300;
    r.sync(&ui);
    r.action(&ui, Action::Down);
    assert_eq!(
        r.focus, 6100,
        "the registered base game leads to local content"
    );
    r.action(&ui, Action::Up);
    assert_eq!(r.focus, 6300);
    r.finish_preferences();
}

/// Spec 102: the actual Slint scroll plane, retargeted while in flight, with
/// timed production frames rather than desktop capture.
#[test]
fn continuous_lists_retarget_preserve_manual_offsets_and_render_motion() {
    if isolated(
        module_path!(),
        "continuous_lists_retarget_preserve_manual_offsets_and_render_motion",
    ) {
        return;
    }
    // Frame rendering/PNG encoding must not consume the animation's clock.
    MOCK_TIME_ONLY.store(true, std::sync::atomic::Ordering::SeqCst);
    use crate::{input::Action, runtime::Category, services::ViewFormat};
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    let prototype = r.snapshot.as_ref().unwrap().items[0].clone();
    r.snapshot.as_mut().unwrap().items = (0..30)
        .map(|index| {
            let mut item = prototype.clone();
            item.recipe.id = format!("motion-{index:02}");
            item.recipe.title = format!("Mapset {index:02}");
            item.favorite = true;
            item
        })
        .collect();
    r.view_format = ViewFormat::List;
    r.choose_category(Category::Favorites);
    for (screen, name, viewport) in [
        (Screen::Discover, "discover", 422.),
        (Screen::Library, "library", 522.),
    ] {
        r.screen = screen;
        r.focus = 10;
        r.sync(&ui);
        advance_time(150);
        let _ = render(&window);
        // Find the last fully visible mapset, then move exactly one row.
        let rows: Vec<_> = ui
            .get_nodes()
            .iter()
            .filter(|n| n.kind == 24 && (10..40).contains(&n.id))
            .map(|mut n| {
                n.y -= ui.get_pinned_height();
                n
            })
            .collect();
        let visible = rows
            .iter()
            .rposition(|n| n.y + n.h + 12. <= ui.get_scroll_offset() + viewport)
            .unwrap();
        r.focus = rows[visible].id;
        r.sync(&ui);
        advance_time(150);
        write_png(&format!("102-{name}-before.png"), &render(&window));
        let before = ui.get_presented_scroll_offset();
        r.action(&ui, Action::Down);
        assert_eq!(r.focus, rows[visible + 1].id);
        let destination = ui.get_scroll_offset();
        assert!(destination > before && destination - before <= 82.);
        let initial = ui.get_presented_scroll_offset();
        assert!(
            (initial - before).abs() < 4.,
            "no instantaneous jump: {initial} vs {before}"
        );
        advance_time(40);
        let middle = ui.get_presented_scroll_offset();
        assert!(
            middle > before && middle < destination,
            "interpolation: {before} < {middle} < {destination}"
        );
        write_png(&format!("102-{name}-during.png"), &render(&window));
        advance_time(160);
        assert!((ui.get_presented_scroll_offset() - destination).abs() < 0.1);
        write_png(&format!("102-{name}-after.png"), &render(&window));
        let focus = r.focus;
        let offset = ui.get_scroll_offset();
        r.action(&ui, Action::Up);
        assert_eq!(
            ui.get_scroll_offset(),
            offset,
            "visible preceding row retains neighbors"
        );
        r.action(&ui, Action::Down);
        assert_eq!(r.focus, focus);
        // More than two viewports of held/repeated down input, including rapid
        // reversals before the 120 ms interpolation finishes.
        for _ in 0..22 {
            r.action(&ui, Action::Down);
            advance_time(16);
            let _ = ui.get_presented_scroll_offset();
        }
        r.action(&ui, Action::Up);
        let reversed = r.focus;
        r.action(&ui, Action::Down);
        r.action(&ui, Action::Up);
        assert_eq!(r.focus, reversed);
        advance_time(160);
        let offset = ui.get_presented_scroll_offset();
        let mut node = ui.get_nodes().iter().find(|n| n.id == r.focus).unwrap();
        node.y -= ui.get_pinned_height();
        assert!(node.y + 4. >= offset && node.y + node.h + 12. <= offset + viewport);
        assert!(offset > 2. * viewport);
        ui.invoke_scroll(0., -137.);
        let manual = ui.get_scroll_offset();
        assert!((manual - (offset - 137.)).abs() < 0.1);
        for _ in 0..3 {
            ui.invoke_scroll(0., 0.);
            r.sync(&ui);
            advance_time(160);
            let _ = render(&window);
            assert_eq!(
                ui.get_scroll_offset(),
                manual,
                "neutral refresh retains manual offset"
            );
            assert!((ui.get_presented_scroll_offset() - manual).abs() < 0.1);
        }
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: slint::LogicalPosition::new(600., 500.),
            delta_x: 0.,
            delta_y: -43.,
        });
        advance_time(20);
        let wheel = ui.get_scroll_offset();
        assert!(
            (wheel - manual - 43.).abs() < 0.1,
            "native wheel starts at the retained position"
        );
        r.sync(&ui);
        ui.invoke_scroll(0., 0.);
        advance_time(160);
        assert_eq!(
            ui.get_scroll_offset(),
            wheel,
            "wheel offset survives neutral refresh"
        );
        // Manual input takes over from an animation's current presentation.
        r.action(&ui, Action::Down);
        advance_time(30);
        let moving = ui.get_presented_scroll_offset();
        ui.invoke_scroll(0., -25.);
        advance_time(160);
        assert!((ui.get_presented_scroll_offset() - (moving - 25.)).abs() < 0.1);
        r.action(&ui, Action::Down);
        advance_time(30);
        let moving = ui.get_presented_scroll_offset();
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: slint::LogicalPosition::new(600., 500.),
            delta_x: 0.,
            delta_y: 19.,
        });
        advance_time(160);
        assert!(
            (ui.get_presented_scroll_offset() - (moving - 19.)).abs() < 0.1,
            "native wheel interrupts focus interpolation"
        );
        ui.invoke_scroll(0., 100000.);
        let limit = (ui.get_content_height() - viewport).max(0.);
        assert!(
            (ui.get_scroll_offset() - limit).abs() < 0.1,
            "manual bottom bound"
        );
        ui.invoke_scroll(0., -100000.);
        assert_eq!(ui.get_scroll_offset(), 0., "manual top bound");
        r.focus = 10;
        r.sync(&ui);
        advance_time(160);
        r.action(&ui, Action::Confirm);
        assert_eq!(r.selected, 0, "actions see current identity immediately");
        r.action(&ui, Action::Back);
        assert_eq!(r.focus, 10);
        let mut node = ui.get_nodes().iter().find(|n| n.id == r.focus).unwrap();
        node.y -= ui.get_pinned_height();
        assert!(node.y + 4. >= ui.get_scroll_offset());
        assert!(node.y + node.h + 12. <= ui.get_scroll_offset() + viewport);
    }
}

#[test]
fn library_favorites_group_in_both_layouts_and_toggle_keeps_identity() {
    if isolated(
        module_path!(),
        "library_favorites_group_in_both_layouts_and_toggle_keeps_identity",
    ) {
        return;
    }
    use crate::{input::Action, services::ViewFormat};
    let (_window, ui) = setup();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    // Equal dates exercise the existing case-insensitive title tie-break;
    // unknown dates still follow known dates within each favorite group.
    for (i, item) in r
        .snapshot
        .as_mut()
        .unwrap()
        .items
        .iter_mut()
        .take(INSTALLED)
        .enumerate()
    {
        item.favorite = i % 2 == 1;
        item.recipe.original_release = (i < 5).then_some(ddl_core::catalog::ReleaseDate {
            year: if matches!(i, 1 | 4) { 2001 } else { 2000 },
            month: None,
            day: None,
        });
        item.recipe.title = format!("Mapset {}", INSTALLED - i);
    }
    let order = |ui: &AppWindow| {
        ui.get_nodes()
            .iter()
            .filter(|n| {
                n.actionable
                    && matches!(n.kind, 1 | 24)
                    && (10..10 + INSTALLED as i32).contains(&n.id)
            })
            .map(|n| n.id)
            .collect::<Vec<_>>()
    };
    for format in [ViewFormat::Cards, ViewFormat::List] {
        r.view_format = format;
        r.focus = 16;
        r.sync(&ui);
        assert_eq!(order(&ui), [13, 11, 15, 12, 10, 14, 16]);
        assert!(ui.get_nodes().iter().any(|n| n.id == 6300));
        assert!(ui.get_nodes().iter().any(|n| n.id == 6101));
    }
    r.action(&ui, Action::Down);
    assert_eq!(
        r.focus, 6100,
        "Down reaches local setups after reordered mapsets"
    );
    r.action(&ui, Action::Up);
    assert_eq!(r.focus, 16);
    let identity = r.snapshot.as_ref().unwrap().items[6].recipe.id.clone();
    r.action(&ui, Action::X);
    // Inspect the optimistic reorder before the service result replaces the
    // synthetic installed snapshot; persistence has its existing service tests.
    assert_eq!(r.focus, 16);
    assert_eq!(r.snapshot.as_ref().unwrap().items[6].recipe.id, identity);
    assert_eq!(order(&ui), [13, 11, 16, 15, 12, 10, 14]);
    r.action(&ui, Action::ToggleView);
    assert_eq!(r.focus, 16);
    assert_eq!(order(&ui), [13, 11, 16, 15, 12, 10, 14]);
    r.action(&ui, Action::ToggleView);
    assert_eq!(order(&ui).len(), INSTALLED);
    let selected = ui.get_nodes().iter().find(|n| n.id == r.focus).unwrap();
    assert!(selected.y + 4. >= ui.get_scroll_offset());
    assert!(selected.y + selected.h + 12. <= ui.get_scroll_offset() + 522.);
    // Settle the real favorite write, retaining the fixture's installed state
    // only for the independent Discover ordering check below.
    let snapshot = r.snapshot.clone();
    for _ in 0..1000 {
        pump_events();
        r.drain(&ui);
        if !r.busy {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!r.busy && r.problem.is_none());
    assert!(
        r.snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .find(|item| item.recipe.id == identity)
            .unwrap()
            .favorite
    );
    r.snapshot = snapshot;
    r.screen = Screen::Discover;
    r.choose_category(crate::runtime::Category::Favorites);
    r.sync(&ui);
    let items = &r.snapshot.as_ref().unwrap().items;
    let mut favorites: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.favorite)
        .collect();
    favorites.sort_by(|(_, a), (_, b)| {
        ddl_core::catalog::compare_discover_order(
            &a.recipe,
            a.manifest.is_some(),
            a.favorite,
            &b.recipe,
            b.manifest.is_some(),
            b.favorite,
        )
    });
    let actual: Vec<_> = ui
        .get_nodes()
        .iter()
        .filter(|n| n.kind == 24)
        .map(|n| n.id)
        .collect();
    assert_eq!(
        actual,
        favorites
            .iter()
            .map(|(i, _)| 10 + *i as i32)
            .collect::<Vec<_>>()
    );
}

/// Spec 111: production geometry, painting, input clipping, and both layouts.
#[test]
fn pinned_discover_categories() {
    if isolated(module_path!(), "pinned_discover_categories") {
        return;
    }
    MOCK_TIME_ONLY.store(true, std::sync::atomic::Ordering::SeqCst);
    use crate::{input::Action, runtime::Category, services::ViewFormat};
    use std::{cell::RefCell, rc::Rc};
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    let prototype = r.snapshot.as_ref().unwrap().items[0].clone();
    r.snapshot.as_mut().unwrap().items = (0..30)
        .map(|index| {
            let mut item = prototype.clone();
            item.recipe.id = format!("pinned-{index:02}");
            item.recipe.title = format!("Mapset {index:02}");
            item.favorite = true;
            item
        })
        .collect();
    r.screen = Screen::Discover;
    r.choose_category(Category::Favorites);
    let clicks = Rc::new(RefCell::new(Vec::new()));
    let clicked = clicks.clone();
    ui.on_clicked(move |id| clicked.borrow_mut().push(id));
    let click = |x, y| {
        let position = slint::LogicalPosition::new(x, y);
        window.dispatch_event(WindowEvent::PointerPressed {
            position,
            button: slint::platform::PointerEventButton::Left,
        });
        window.dispatch_event(WindowEvent::PointerReleased {
            position,
            button: slint::platform::PointerEventButton::Left,
        });
    };
    let strip_pixels = |frame: &slint::SharedPixelBuffer<slint::Rgb8Pixel>| {
        (230..310)
            .flat_map(|y| {
                frame.as_slice()[y * WIDTH as usize + 20..y * WIDTH as usize + 1260]
                    .iter()
                    .copied()
            })
            .collect::<Vec<_>>()
    };
    for (layout, name) in [(ViewFormat::Cards, "cards"), (ViewFormat::List, "list")] {
        r.view_format = layout;
        r.focus = 10;
        r.sync(&ui);
        advance_time(160);
        let chip = ui
            .get_nodes()
            .iter()
            .find(|n| n.pinned && n.actionable && n.primary)
            .unwrap();
        // Enter and leave the selected chip through real navigation.
        let top = ui.get_scroll_offset();
        r.action(&ui, Action::Up);
        assert_eq!(r.focus, chip.id);
        assert_eq!(
            ui.get_scroll_offset(),
            top,
            "Up into pinned strip never jumps"
        );
        advance_time(160);
        let baseline = render(&window);
        write_png(&format!("111-{name}-top-chip.png"), &baseline);
        r.action(&ui, Action::Down);
        assert_eq!(r.focus, 10);
        advance_time(160);
        write_png(&format!("111-{name}-top-item.png"), &render(&window));
        for _ in 0..20 {
            r.action(&ui, Action::Down);
            advance_time(16);
        }
        r.action(&ui, Action::Up);
        let reversed = r.focus;
        r.action(&ui, Action::Down);
        r.action(&ui, Action::Up);
        assert_eq!(
            r.focus, reversed,
            "direction reversal keeps the right identity"
        );
        advance_time(160);
        // Every item, first/last included, and a complete reversal.
        for id in (10..40).chain((10..40).rev()) {
            r.focus = id;
            r.sync(&ui);
            advance_time(160);
            let n = ui.get_nodes().iter().find(|n| n.id == id).unwrap();
            let ring = if layout == ViewFormat::Cards { 16. } else { 8. };
            let top = 230. + n.y + 8. - ring - ui.get_presented_scroll_offset();
            let bottom = top + n.h + 2. * ring;
            assert!(top >= 310. - 0.1, "{name} {id} ring top {top}");
            assert!(bottom <= 732. + 0.1, "{name} {id} ring bottom {bottom}");
        }
        r.focus = 39;
        r.sync(&ui);
        advance_time(160);
        assert!(ui.get_scroll_offset() > 2. * 422.);
        write_png(&format!("111-{name}-deep-item.png"), &render(&window));
        r.focus = chip.id;
        r.sync(&ui);
        advance_time(160);
        let deep = render(&window);
        write_png(&format!("111-{name}-deep-chip.png"), &deep);
        assert_eq!(
            strip_pixels(&baseline),
            strip_pixels(&deep),
            "strip pixels fixed at deep offset"
        );
        // Stick, wheel and Flickable drag all paint the same stationary strip.
        for offset in [37., 137., 900.] {
            ui.invoke_follow_scroll(offset, false);
            advance_time(160);
            assert_eq!(strip_pixels(&baseline), strip_pixels(&render(&window)));
        }
        window.dispatch_event(WindowEvent::PointerScrolled {
            position: slint::LogicalPosition::new(600., 500.),
            delta_x: 0.,
            delta_y: -43.,
        });
        advance_time(160);
        assert_eq!(strip_pixels(&baseline), strip_pixels(&render(&window)));
        ui.invoke_scroll(0., 73.);
        advance_time(160);
        assert_eq!(strip_pixels(&baseline), strip_pixels(&render(&window)));
        let before_drag = ui.get_scroll_offset();
        window.dispatch_event(WindowEvent::PointerPressed {
            position: slint::LogicalPosition::new(600., 500.),
            button: slint::platform::PointerEventButton::Left,
        });
        advance_time(50);
        window.dispatch_event(WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(600., 440.),
        });
        advance_time(50);
        window.dispatch_event(WindowEvent::PointerReleased {
            position: slint::LogicalPosition::new(600., 440.),
            button: slint::platform::PointerEventButton::Left,
        });
        advance_time(160);
        assert!(
            ui.get_scroll_offset() > before_drag,
            "Flickable drag scrolls content"
        );
        assert_eq!(strip_pixels(&baseline), strip_pixels(&render(&window)));
        // Position the first item across the strip's lower clipping boundary.
        ui.invoke_follow_scroll(70., false);
        advance_time(160);
        render(&window);
        clicks.borrow_mut().clear();
        click(60., 305.); // blank lower strip margin, with a hidden mapset behind
        assert!(
            clicks.borrow().is_empty(),
            "hidden content cannot receive clicks"
        );
        click(60., 315.); // visible part of that same item
        assert_eq!(clicks.borrow().last(), Some(&10));
        clicks.borrow_mut().clear();
        click(chip.x + 28. + chip.w / 2., 270.);
        assert_eq!(
            clicks.borrow().as_slice(),
            &[chip.id],
            "chip routes independently of scroll"
        );
        // Category changes reset content even when a pinned chip had focus.
        r.action(&ui, Action::NextCategory);
        assert_eq!(ui.get_scroll_offset(), 0.);
        r.choose_category(Category::Favorites);
        r.focus = 39;
        r.sync(&ui);
        advance_time(160);
        assert!(ui.get_scroll_offset() > 2. * 422.);
        r.action(&ui, Action::PreviousCategory);
        assert_eq!(
            ui.get_scroll_offset(),
            0.,
            "category switch from an item starts at top"
        );
        r.choose_category(Category::Favorites);
        r.focus = 10;
        r.sync(&ui);
        advance_time(160);
    }
    r.finish_preferences();
}

/// Owner-selected production density, with real catalog titles/artwork/tags.
#[test]
fn compact_discover_catalog_scene() {
    if isolated(module_path!(), "compact_discover_catalog_scene") {
        return;
    }
    use crate::{runtime::Category, services::ViewFormat};
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut r = owner_runtime(temp.path(), &ui);
    r.screen = Screen::Discover;
    for item in &mut r.snapshot.as_mut().unwrap().items {
        item.favorite = true;
    }
    r.choose_category(Category::Favorites);
    r.view_format = ViewFormat::Cards;
    r.sync(&ui);
    let all_nodes: Vec<_> = ui.get_nodes().iter().collect();
    let all_cards: Vec<_> = all_nodes
        .iter()
        .filter(|n| n.kind == 1 && n.actionable)
        .collect();
    assert_eq!(all_cards.len(), r.snapshot.as_ref().unwrap().items.len());
    // Exercise actual production chips for every recipe's tag combination.
    for card in all_cards {
        let tags: Vec<_> = all_nodes
            .iter()
            .filter(|n| {
                n.kind == 13
                    && n.x >= card.x
                    && n.x < card.x + card.w
                    && n.y >= card.y
                    && n.y < card.y + card.h
            })
            .collect();
        assert!((1..=2).contains(&tags.len()));
        for tag in tags {
            assert_eq!((tag.y, tag.h, tag.size), (card.y + 282., 26., 14.));
            assert!(tag.x >= card.x + 28. && tag.x + tag.w <= card.x + 356.);
        }
    }
    r.choose_category(Category::Tag("community-projects".into()));
    for (layout, name, kind) in [
        (ViewFormat::Cards, "cards", 1),
        (ViewFormat::List, "list", 24),
    ] {
        r.view_format = layout;
        r.focus = -3;
        r.sync(&ui);
        advance_time(160);
        let items: Vec<_> = ui
            .get_nodes()
            .iter()
            .filter(|n| n.actionable && n.kind == kind)
            .collect();
        assert!(items.len() > 9, "real catalog has several card rows");
        if layout == ViewFormat::Cards {
            let nodes: Vec<_> = ui.get_nodes().iter().collect();
            for card in &items {
                assert_eq!((card.w, card.h), (384., 340.));
                let tags: Vec<_> = nodes
                    .iter()
                    .filter(|n| {
                        n.kind == 13
                            && n.x >= card.x
                            && n.x < card.x + card.w
                            && n.y >= card.y
                            && n.y < card.y + card.h
                    })
                    .collect();
                assert!((1..=2).contains(&tags.len()));
                assert_eq!(tags[0].label, "Community Projects", "open category leads");
                for tag in tags {
                    assert_eq!((tag.y, tag.h, tag.size), (card.y + 282., 26., 14.));
                    assert!(tag.x >= card.x + 28. && tag.x + tag.w <= card.x + 356.);
                }
                let art = nodes
                    .iter()
                    .find(|n| n.kind == 4 && n.x == card.x + 28. && n.y == card.y + 28.)
                    .unwrap();
                assert_eq!((art.w, art.h), (328., 96.));
            }
        } else {
            assert!(items.iter().all(|n| n.h == 74.), "List geometry unchanged");
        }
        let two_tag_items: Vec<_> = items
            .iter()
            .filter(|n| {
                r.snapshot.as_ref().unwrap().items[(n.id - 10) as usize]
                    .recipe
                    .tags
                    .len()
                    >= 2
            })
            .collect();
        for (item, position) in [
            (*two_tag_items.first().unwrap(), "top"),
            (*two_tag_items.last().unwrap(), "deep"),
        ] {
            r.focus = item.id;
            r.sync(&ui);
            advance_time(160);
            if position == "top" {
                assert_eq!(ui.get_scroll_offset(), 0.);
            }
            if position == "deep" {
                assert!(
                    ui.get_scroll_offset() > 0.,
                    "last real catalog item is scrolled into view"
                );
            }
            write_png(
                &format!("111-catalog-{name}-{position}.png"),
                &render(&window),
            );
        }
    }
    // Real Doomer Boards recipes expose the two long labels together.
    r.choose_category(Category::Tag("doomer-boards".into()));
    r.view_format = ViewFormat::Cards;
    r.focus = -3;
    r.sync(&ui);
    advance_time(160);
    let nodes: Vec<_> = ui.get_nodes().iter().collect();
    let cards: Vec<_> = nodes
        .iter()
        .filter(|n| n.kind == 1 && n.actionable)
        .collect();
    for (card, position) in [(cards[0], "top"), (*cards.last().unwrap(), "deep")] {
        let tags: Vec<_> = nodes
            .iter()
            .filter(|n| {
                n.kind == 13
                    && n.x >= card.x
                    && n.x < card.x + card.w
                    && n.y >= card.y
                    && n.y < card.y + card.h
            })
            .map(|n| n.label.as_str())
            .collect();
        assert_eq!(tags, ["Doomer Boards", "Community Projects"]);
        r.focus = card.id;
        r.sync(&ui);
        advance_time(160);
        if position == "top" {
            assert_eq!(ui.get_scroll_offset(), 0.);
        } else {
            assert!(ui.get_scroll_offset() > 2. * 422.);
        }
        write_png(
            &format!("111-catalog-long-tags-{position}.png"),
            &render(&window),
        );
    }
    r.finish_preferences();
}
