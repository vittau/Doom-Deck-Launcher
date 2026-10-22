//! Production presentation state. Workers own filesystem/network/engine operations.
use crate::{
    AppWindow, ViewNode,
    input::Action,
    services::{self, Command, Directory, Environment, Event, Item, Problem, Services, Snapshot},
    wiring_copy::{Text as T, text, value},
};
#[path = "local_ui.rs"]
mod local_ui;
#[path = "osk.rs"]
pub(crate) mod osk;
use local_ui::LocalUi;

use ddl_core::{
    catalog::{compare_browsing_order, shows_in_category},
    downloads::{CancellationToken, InstallProgress, InstallStage},
    platform::{legacy_import::LegacyInstall, sandbox::Sandbox},
    profiles::Profile,
    resolver::Blocker,
    soundfonts::{Choice, engine::Current},
};
use slint::{ComponentHandle, Model as _, ModelRc, SharedString, VecModel};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{self, Receiver},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Loading,
    FirstRun,
    Recovery,
    Home,
    Discover,
    Library,
    Detail,
    Setup,
    Progress,
    Review,
    Browser,
    Settings,
}
/// A Discover category (spec 015, R4; spec 072). The strip lists the owner's
/// requested specific categories that some recipe uses, then `Favorites`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Category {
    Tag(String),
    Favorites,
}
/// The owner's Discover order (spec 072 R1). Vocabulary entries outside this
/// list keep their catalog metadata but are not navigation categories.
const CATEGORY_ORDER: [&str; 8] = [
    "cacowards",
    "megawad",
    "community-projects",
    "doomer-boards",
    "classics",
    "episode",
    "challenging",
    "gimmick",
];
/// First node id of a Discover/Library card; card `k` has id `CARD + k`.
pub(crate) const CARD: i32 = 10;
/// Last card id. Kept below `CHIP` so cards and category chips never collide
/// (spec 072 R4: every mapset in the 90+ recipe catalog stays reachable).
pub(crate) const CARD_LAST: i32 = 199;
/// Overflow IDs avoid every category, base-game and local-control range.
fn card_id(index: usize) -> i32 {
    if index < 190 {
        CARD + index as i32
    } else {
        1_000_000 + index as i32
    }
}
fn card_index(id: i32) -> Option<usize> {
    if (CARD..=CARD_LAST).contains(&id) {
        Some((id - CARD) as usize)
    } else if id >= 1_000_190 {
        Some((id - 1_000_000) as usize)
    } else {
        None
    }
}
/// First node id of the category strip; chip `k` has id `CHIP + k`.
const CHIP: i32 = 200;
const CHIPS_MAX: usize = 30;
/// "Browse mapsets" action of the empty Favorites state.
const BROWSE: i32 = 230;
/// Horizontal room for the strip, between the card grid's edges.
const STRIP_WIDTH: f32 = 1200.;
/// Vertical room the strip takes above the Discover grid.
const STRIP_HEIGHT: f32 = 72.;
/// The requested categories that at least one recipe uses, in the owner's order.
fn requested_categories(used: impl Fn(&str) -> bool) -> Vec<Category> {
    CATEGORY_ORDER
        .iter()
        .filter(|id| used(id))
        .take(CHIPS_MAX - 1)
        .map(|id| Category::Tag((*id).to_owned()))
        .collect()
}
/// Spec 072 R2: the first available category, or Favorites when none exist.
fn first_available(available: &[Category]) -> Category {
    available.first().cloned().unwrap_or(Category::Favorites)
}
#[derive(Clone)]
enum Sheet {
    Quit,
    Cancel,
    Reset,
    StartFresh,
    Remove(PathBuf),
    Component(String),
    /// An imported user-supplied component: toggle it or import a replacement.
    UserComponent(String),
    Profiles,
    ProfileNames,
    DeleteProfile(String, String),
    /// Spec 022: the uninstall confirm carries the focused recipe id.
    Uninstall(String),
    /// Spec 059 R5: offer to import a tarball-era setup into the Flatpak.
    ImportOffer,
}
pub struct Runtime {
    env: Environment,
    pub(crate) view_format: services::ViewFormat,
    view_loaded: bool,
    view_writer: Option<services::ViewWriter>,
    local: LocalUi,
    pub screen: Screen,
    pub focus: i32,
    pub snapshot: Option<Snapshot>,
    pub selected: usize,
    history: Vec<(Screen, i32)>,
    remembered: [i32; 4],
    pub busy: bool,
    pub playing: bool,
    neutral: bool,
    cancel: CancellationToken,
    rx: Option<Receiver<Event>>,
    pub pending: Option<Command>,
    retry: Option<Command>,
    return_to: (Screen, i32),
    nodes: Vec<ViewNode>,
    sheet: Option<Sheet>,
    invoking: i32,
    pub progress: Option<InstallProgress>,
    pub problem: Option<Problem>,
    pub directory: Option<Directory>,
    import_component: Option<String>,
    review_return: (Screen, i32),
    unreadable: Vec<services::UnreadableFile>,
    recovery_attempts: usize,
    recovery_reset: bool,
    settings_section: usize,
    advanced: bool,
    /// Spec 022 R2: the uninstall checkbox is unchecked every time the sheet opens.
    uninstall_delete_profiles: bool,
    /// Discover category, remembered for the session (spec 015, R4).
    category: Category,
    /// False until the first snapshot picks the initial specific category.
    category_chosen: bool,
    /// First strip chip drawn when the strip is wider than the grid.
    strip_start: usize,
    scroll_anchor: Option<(Screen, Option<local_ui::View>, i32, f32, f32)>,
    updates: Vec<String>,
    updates_checked: bool,
    pub last_exit: Option<ddl_core::engines::dsda::LaunchOutcome>,
    /// Spec 059: the sandbox DDL runs in.
    sandbox: Sandbox,
    /// Spec 059 R5: the tarball-era install to offer importing, until the
    /// player answers or the import succeeds.
    import_offer: Option<LegacyInstall>,
    /// The next `start` imports `import_offer` before opening the services.
    import_now: bool,
    /// Spec 075, R2: a light Pistol start write is in flight.
    pistol_in_flight: bool,
    /// Spec 077, R1: the per-startup catalog check runs on its own channel so it
    /// never sets `busy`.
    catalog_rx: Option<Receiver<services::StartupCatalog>>,
    /// Spec 077, R1: at most one automatic check per application start.
    catalog_checked: bool,
    /// Spec 077, R1: a newer catalog is cached and the fresh view is pending.
    catalog_reload_pending: bool,
    /// Spec 075, R2: the latest requested value per profile id.
    pistol_intent: std::collections::BTreeMap<String, bool>,
    /// Spec 075, R2: the last value a successful save confirmed per profile id.
    pistol_baseline: std::collections::BTreeMap<String, bool>,
    /// Spec 075, R2: the queued catalog request, `(recipe_id, profile_id)`.
    pistol_queue: Option<(String, String)>,
    /// Spec 075, R2: the queued base-game request.
    base_pistol_queue: Option<String>,
}

impl Runtime {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
            view_format: services::ViewFormat::Cards,
            view_loaded: false,
            view_writer: None,
            local: LocalUi::default(),
            screen: Screen::Loading,
            focus: 1,
            snapshot: None,
            selected: 0,
            history: vec![],
            remembered: [1, 10, 10, 1],
            busy: false,
            playing: false,
            neutral: false,
            cancel: CancellationToken::new(),
            rx: None,
            pending: None,
            retry: None,
            return_to: (Screen::Home, 1),
            nodes: vec![],
            sheet: None,
            invoking: 1,
            progress: None,
            problem: None,
            directory: None,
            import_component: None,
            review_return: (Screen::FirstRun, 2),
            unreadable: vec![],
            recovery_attempts: 0,
            recovery_reset: false,
            settings_section: 0,
            advanced: false,
            uninstall_delete_profiles: false,
            category: Category::Favorites,
            category_chosen: false,
            strip_start: 0,
            scroll_anchor: None,
            updates: vec![],
            updates_checked: false,
            last_exit: None,
            sandbox: Sandbox::none(),
            import_offer: None,
            import_now: false,
            pistol_in_flight: false,
            pistol_intent: std::collections::BTreeMap::new(),
            pistol_baseline: std::collections::BTreeMap::new(),
            catalog_rx: None,
            catalog_checked: false,
            catalog_reload_pending: false,
            pistol_queue: None,
            base_pistol_queue: None,
        }
    }
    /// Runs inside `sandbox`: decides, before the services create any file of
    /// their own, whether a tarball-era install should be offered (spec 059 R5).
    #[must_use]
    pub fn with_sandbox(mut self, sandbox: Sandbox) -> Self {
        self.import_offer =
            ddl_core::platform::legacy_import::find(&sandbox, &self.env.home, &self.env.paths);
        self.sandbox = sandbox;
        self
    }
    fn unreadable_text(&self) -> String {
        self.unreadable
            .iter()
            .map(|f| {
                format!(
                    "{}\n{}",
                    f.path.display(),
                    f.reason.replace(['\n', '\r'], " ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    fn diagnostics_text(&self) -> String {
        format!(
            "DoomDeckLauncher {}\n\nLog: {}\n\nConfig: {}\n\nData: {}\n\n{}\n\n{}",
            crate::cli::VERSION,
            self.env.paths.state.join("logs/ddl.log").display(),
            self.env.paths.config.display(),
            self.env.paths.data.display(),
            sandbox_line(&self.sandbox),
            self.unreadable_text()
        )
    }
    fn item(&self) -> Option<&Item> {
        self.snapshot.as_ref()?.items.get(self.selected)
    }
    /// Adopts a fresh snapshot while keeping the selected mapset and card focus
    /// by recipe identity (spec 090 R5). A catalog insertion, reorder or update
    /// must not silently move the user to a different mapset; when the selected
    /// recipe disappeared, the selection clamps to a valid neighbour.
    fn replace_snapshot(&mut self, snapshot: Snapshot) {
        let focused_index = card_index(self.focus);
        let focused_id = focused_index
            .and_then(|index| self.snapshot.as_ref()?.items.get(index))
            .map(|item| item.recipe.id.clone());
        let selected_id = self
            .snapshot
            .as_ref()
            .and_then(|old| old.items.get(self.selected))
            .map(|item| item.recipe.id.clone());
        let card_focus = matches!(self.screen, Screen::Discover | Screen::Library)
            && card_index(self.focus).is_some();
        self.snapshot = Some(snapshot);
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        if snapshot.items.is_empty() {
            self.selected = 0;
            return;
        }
        let Some(id) = selected_id else {
            return;
        };
        self.selected = snapshot
            .items
            .iter()
            .position(|item| item.recipe.id == id)
            .unwrap_or_else(|| self.selected.min(snapshot.items.len() - 1));
        if card_focus {
            let index = focused_id
                .and_then(|id| snapshot.items.iter().position(|item| item.recipe.id == id))
                .unwrap_or_else(|| {
                    focused_index
                        .unwrap_or(self.selected)
                        .min(snapshot.items.len() - 1)
                });
            self.focus = card_id(index);
        }
    }
    fn root(&self) -> bool {
        matches!(
            self.screen,
            Screen::Home | Screen::Discover | Screen::Library | Screen::Settings
        ) && self.sheet.is_none()
            && self.local.dialog.is_none()
            && self.local.view.is_none()
    }
    fn enter(&mut self, screen: Screen, focus: i32) {
        self.history.push((self.screen, self.focus));
        self.screen = screen;
        self.focus = focus;
    }
    fn tab(&mut self, ui: &AppWindow, tab: usize) {
        if self.root() {
            if self.tab_index() != tab.min(3) {
                ui.invoke_show_toast(SharedString::default());
            }
            if self.focus >= 0 {
                self.remembered[self.tab_index()] = self.focus;
            }
            if !self.unreadable.is_empty() && tab != 3 {
                self.screen = Screen::Recovery;
                self.focus = 800;
                self.sync(ui);
                return;
            }
            self.screen = [
                Screen::Home,
                Screen::Library,
                Screen::Discover,
                Screen::Settings,
            ][tab.min(3)];
            self.focus = self.remembered[tab.min(3)];
            self.history.clear();
        }
    }
    fn show_sheet(&mut self, sheet: Sheet) {
        self.invoking = self.focus;
        // R2: the checkbox always starts unchecked.
        self.uninstall_delete_profiles = false;
        self.focus = if matches!(sheet, Sheet::Profiles) {
            1000 + self
                .item()
                .and_then(|i| i.profiles.iter().position(|p| p.id == i.profile.id))
                .unwrap_or(0) as i32
        } else if matches!(sheet, Sheet::UserComponent(_)) {
            901
        } else {
            900
        };
        self.sheet = Some(sheet);
    }
    pub fn start(&mut self, ui: &AppWindow, command: Command) {
        if self.busy {
            return;
        }
        self.busy = true;
        if matches!(
            command,
            Command::Play(_) | Command::Local(services::LocalCommand::Play(_))
        ) {
            self.playing = true;
            self.neutral = true;
        }
        self.problem = None;
        ui.invoke_show_toast(SharedString::default());
        self.progress = None;
        self.cancel = CancellationToken::new();
        if self.screen != Screen::Progress {
            self.return_to = (self.screen, self.focus);
        }
        self.pending = Some(command.clone());
        self.retry = Some(command.clone());
        if matches!(
            command,
            Command::InstallEngine(_)
                | Command::Install(_)
                | Command::UpdateMapset(_)
                | Command::InstallSoundfont
        ) && self.screen != Screen::Progress
        {
            self.enter(Screen::Progress, 1);
        }
        let fresh = std::mem::take(&mut self.recovery_reset);
        let import = if std::mem::take(&mut self.import_now) {
            self.import_offer.clone()
        } else {
            None
        };
        let sandbox = self.sandbox.clone();
        self.env.screen = drawable_screen(ui);
        let env = self.env.clone();
        let cancel = self.cancel.clone();
        let weak = ui.as_weak();
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let mut emit = |event| {
                if tx.send(event).is_ok() {
                    let _ = weak.upgrade_in_event_loop(|ui| ui.invoke_services_ready());
                }
            };
            let prepared = if fresh {
                services::start_fresh(&env).map(|_| ())
            } else if let Some(legacy) = &import {
                services::import_legacy(&env, legacy, &cancel).map(|report| {
                    emit(Event::Imported {
                        engines_to_reinstall: report.skipped_engines.len(),
                    });
                })
            } else {
                Ok(())
            };
            let result = prepared
                .and_then(|_| Services::open_with(env, sandbox))
                .and_then(|mut services| {
                    let refresh = services.execute(command, &cancel, &mut emit)?;
                    match refresh {
                        // Spec 075, R1: the light path reports its one value and
                        // skips the snapshot, so nothing re-hashes the library.
                        services::Refresh::Profile(profile) => {
                            emit(Event::ProfileSaved(*profile));
                            Ok(None)
                        }
                        services::Refresh::Full => services.snapshot().map(|s| Some(Box::new(s))),
                    }
                });
            emit(Event::Finished(result));
        });
    }
    /// Same completion path is drained by the Slint event-loop callback and headless tests.
    pub fn drain(&mut self, ui: &AppWindow) {
        if let Some(Err(problem)) = self
            .view_writer
            .as_ref()
            .and_then(|writer| writer.take_result())
        {
            ui.invoke_show_toast(problem_text(ui, &problem));
        }
        let events: Vec<_> = self
            .rx
            .as_ref()
            .map(|r| r.try_iter().collect())
            .unwrap_or_default();
        // Spec 077, R1: the startup catalog check has its own channel so it can
        // finish while the user is already navigating or saving.
        let catalog_events: Vec<_> = self
            .catalog_rx
            .as_ref()
            .map(|r| r.try_iter().collect())
            .unwrap_or_default();
        let mut auto_check = false;
        for result in catalog_events {
            // Spec 077, R1/R3: adopt a newer catalog without touching the busy
            // gate, the pending command or the current screen.
            if let Some(serial) = result.updated {
                let date =
                    ddl_core::catalog::serial_date(serial).unwrap_or_else(|| "unknown".into());
                ui.invoke_show_toast(value(ui, T::CatalogUpdated, date));
                // Adopt through a fresh services instance once no command runs,
                // never from this background instance's older state.
                self.catalog_reload_pending = true;
            }
            self.catalog_rx = None;
        }
        for event in events {
            match event {
                Event::Working => {}
                Event::ViewFormat(format) => {
                    if !self.view_loaded {
                        self.view_format = format;
                        self.view_loaded = true;
                    }
                }
                Event::Progress(p) => self.progress = Some(p),
                Event::Playing => {
                    self.playing = true;
                    self.neutral = true;
                }
                Event::Directory(d) => {
                    self.directory = Some(d);
                    if self.local.view.is_none() {
                        self.screen = Screen::Browser;
                    }
                    self.focus = 1;
                }
                Event::Updates(u) => {
                    self.updates = u;
                    self.updates_checked = true;
                }
                Event::Imported {
                    engines_to_reinstall,
                } => {
                    self.import_offer = None;
                    ui.invoke_show_toast(text(
                        ui,
                        if engines_to_reinstall == 0 {
                            T::ImportDone
                        } else {
                            T::ImportEngineAgain
                        },
                    ));
                }
                Event::Catalog(outcome) => {
                    // Spec 019 R5: the manual action reports its result.
                    match &outcome {
                        services::CatalogOutcome::Updated { serial, .. } => {
                            let date = ddl_core::catalog::serial_date(*serial)
                                .unwrap_or_else(|| "unknown".into());
                            ui.invoke_show_toast(value(ui, T::CatalogUpdated, date));
                        }
                        services::CatalogOutcome::Current { .. } => {
                            ui.invoke_show_toast(text(ui, T::CatalogCurrent));
                        }
                    }
                }
                Event::EngineExit(outcome) => {
                    if !outcome.exit_status.is_clean() {
                        ui.invoke_show_toast(value(
                            ui,
                            T::EngineFailed,
                            outcome.log_path.display(),
                        ));
                    }
                    self.last_exit = Some(outcome);
                }
                Event::ProfileSaved(profile) => {
                    self.apply_saved_profile(&profile);
                }
                Event::Finished(result) => {
                    self.busy = false;
                    self.playing = false;
                    let command = self.pending.take();
                    match result {
                        Ok(snapshot) => {
                            if let Some(snapshot) = snapshot {
                                self.replace_snapshot(*snapshot);
                                self.unreadable.clear();
                                self.recovery_attempts = 0;
                                self.local_finished(ui, &command);
                                self.problem = None;
                            }
                            self.pistol_in_flight = false;
                            if let Some(Command::UpdateMapset(id)) = &command
                                && let Some(item) = self
                                    .snapshot
                                    .as_ref()
                                    .and_then(|s| s.items.iter().find(|i| &i.recipe.id == id))
                                && !item.update_notice.is_empty()
                            {
                                ui.invoke_show_toast(value(
                                    ui,
                                    T::RecipeNotice,
                                    item.update_notice.join(" · "),
                                ));
                            }
                            match command {
                                Some(Command::NewProfile(..)) => self.edit_profile_name(ui),
                                Some(Command::Startup) => {
                                    self.screen = if self
                                        .snapshot
                                        .as_ref()
                                        .is_some_and(|s| s.engine.is_none())
                                    {
                                        Screen::FirstRun
                                    } else {
                                        Screen::Home
                                    };
                                    self.focus = 1;
                                    // Spec 019 R5: check in the background once
                                    // the UI is up, at most every 24 h.
                                    auto_check = self.should_auto_check();
                                    // Spec 059 R5: a tarball-era install is
                                    // offered once the UI is up.
                                    if self.import_offer.is_some() {
                                        self.show_sheet(Sheet::ImportOffer);
                                    }
                                }
                                Some(Command::InstallEngine(_)) => {
                                    self.screen = self.return_to.0;
                                    self.focus = 2;
                                    self.history.pop();
                                }
                                // Spec 067 R2/R6: return to Audio with a valid
                                // focus: Arachno once its install is published,
                                // otherwise the Default row.
                                Some(Command::InstallSoundfont) => {
                                    self.screen = Screen::Settings;
                                    self.settings_section = 1;
                                    self.focus = if self
                                        .snapshot
                                        .as_ref()
                                        .is_some_and(|s| s.audio.installed)
                                    {
                                        71
                                    } else {
                                        70
                                    };
                                }
                                Some(Command::Scan(_) | Command::Choose(..)) => {
                                    self.screen = Screen::Review;
                                    self.focus = if self.snapshot.as_ref().is_some_and(|s| {
                                        s.iwads.iter().any(|e| !e.candidates.is_empty())
                                    }) {
                                        10
                                    } else {
                                        1
                                    };
                                    if matches!(command, Some(Command::Scan(_))) {
                                        self.review_return = if self.return_to.0 == Screen::Browser
                                        {
                                            self.history.pop().unwrap_or((Screen::FirstRun, 2))
                                        } else {
                                            self.return_to
                                        };
                                    }
                                }
                                Some(Command::Install(_) | Command::UpdateMapset(_)) => {
                                    self.screen = Screen::Setup;
                                    self.focus = 50;
                                    if self.return_to.0 == Screen::Setup {
                                        self.history.pop();
                                    }
                                }
                                Some(Command::Play(_)) => {
                                    // Direct Play returns to the screen that started
                                    // it; a run begun from Play setup returns there.
                                    self.screen = self.return_to.0;
                                    self.focus = self.return_to.1;
                                }
                                Some(Command::Import(id, component, _)) => {
                                    self.screen = Screen::Setup;
                                    self.focus = 50;
                                    self.history.pop();
                                    // Spec 094 R4: an unrecognized import is
                                    // usable but must be reported honestly.
                                    let unrecognized = self
                                        .snapshot
                                        .as_ref()
                                        .and_then(|snapshot| {
                                            snapshot.items.iter().find(|item| item.recipe.id == id)
                                        })
                                        .is_some_and(|item| {
                                            let known = item
                                                .recipe
                                                .components
                                                .iter()
                                                .find(|candidate| candidate.id == component)
                                                .map(|candidate| candidate.known_sha256.as_slice())
                                                .unwrap_or(&[]);
                                            item.manifest
                                                .as_ref()
                                                .and_then(|manifest| {
                                                    manifest.find_imported(&component)
                                                })
                                                .is_some_and(|member| {
                                                    !known.iter().any(|hash| hash == &member.sha256)
                                                })
                                        });
                                    if unrecognized {
                                        ui.invoke_show_toast(text(ui, T::UnknownFile));
                                    }
                                }
                                Some(Command::Uninstall(id, _)) => {
                                    ui.invoke_show_toast(value(ui, T::Uninstalled, &id));
                                    self.screen = Screen::Library;
                                    self.focus = CARD;
                                    self.history.clear();
                                }
                                _ => {}
                            }
                        }
                        Err(Problem::Unreadable(files)) => {
                            self.unreadable = files;
                            self.recovery_attempts += 1;
                            self.snapshot = None;
                            self.problem = None;
                            self.screen = Screen::Recovery;
                            self.focus = 800;
                        }
                        Err(Problem::LocalFile(path, changed)) => {
                            self.local_show_issue((path, changed))
                        }
                        Err(Problem::Local(message)) => self.local_error(message),
                        Err(problem) => {
                            // Spec 075, R2: a failed write must not leave the
                            // optimistic value looking saved.
                            match &command {
                                Some(Command::SetPistolStart { profile_id, .. }) => {
                                    self.revert_pistol_start(profile_id);
                                }
                                Some(Command::Local(services::LocalCommand::BasePistolStart {
                                    profile_id,
                                    ..
                                })) => {
                                    self.revert_pistol_start(profile_id);
                                }
                                _ => {}
                            }
                            self.problem = Some(problem);
                            self.focus = 800;
                            if self.screen == Screen::Loading {
                                self.screen = Screen::FirstRun;
                            }
                            if matches!(command, Some(Command::Play(_))) {
                                self.screen = Screen::Setup;
                                self.focus = 50;
                            }
                        }
                    }
                }
            }
        }
        // Spec 077, R1: dispatched after the startup snapshot so the first frame
        // never waits on the network, outside the busy gate.
        if auto_check {
            self.spawn_catalog_check(ui);
        }
        // Spec 077, R1: adopt a newer catalog through a fresh instance first;
        // the flag is cleared only when the reload is actually dispatched, so a
        // busy moment can never drop it.
        if !self.busy && self.catalog_reload_pending {
            self.catalog_reload_pending = false;
            self.start(ui, Command::RefreshCatalog);
        } else {
            // Spec 075, R2: a queued press, or a save result overtaken by a
            // newer press, is applied once the worker is free.
            self.settle_pistol(ui);
        }
        self.sync(ui);
    }
    /// Spec 077, R1: the automatic check runs once per application start, in the
    /// background after the first usable snapshot, whenever the opt-out is off.
    /// There is no time-based throttle: every startup checks once.
    fn should_auto_check(&self) -> bool {
        auto_check_due(
            self.env.is_production(),
            self.catalog_checked,
            self.snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.catalog.auto_update),
        )
    }
    /// Spec 077, R1: fetch and adopt a newer catalog without setting `busy`, so
    /// navigation and actions keep working while the network is out.
    fn spawn_catalog_check(&mut self, ui: &AppWindow) {
        self.catalog_checked = true;
        let (tx, rx) = mpsc::channel();
        self.catalog_rx = Some(rx);
        let env = self.env.clone();
        let sandbox = self.sandbox.clone();
        let weak = ui.as_weak();
        std::thread::spawn(move || {
            let result = Services::open_with(env, sandbox)
                .and_then(|mut services| services.startup_catalog_check(&CancellationToken::new()))
                .unwrap_or_default();
            if tx.send(result).is_ok() {
                let _ = weak.upgrade_in_event_loop(|ui| ui.invoke_services_ready());
            }
        });
    }
    pub fn hotplug(&self, ui: &AppWindow, connected: bool) {
        if !self.playing {
            ui.invoke_show_toast(text(
                ui,
                if connected {
                    T::Connected
                } else {
                    T::Disconnected
                },
            ));
        }
    }
    /// Normal teardown runs this after Slint stops, never in an interactive callback.
    pub fn finish_preferences(&mut self) {
        if let Some(writer) = self.view_writer.take() {
            writer.finish();
        }
    }
    fn can_toggle_view(&self) -> bool {
        matches!(self.screen, Screen::Discover | Screen::Library)
            && self.local.view.is_none()
            && self.local.dialog.is_none()
            && self.sheet.is_none()
            && !self.playing
            && !self.neutral
    }
    fn write_view_format(&mut self, ui: &AppWindow, format: services::ViewFormat) {
        if self.view_writer.is_none() {
            let weak = ui.as_weak();
            self.view_writer = Some(services::ViewWriter::new(self.env.clone(), move || {
                let _ = weak.upgrade_in_event_loop(|ui| ui.invoke_services_ready());
            }));
        }
        self.view_writer.as_ref().unwrap().set(format);
    }
    pub fn action(&mut self, ui: &AppWindow, action: Action) {
        if action == Action::ToggleView {
            if self.can_toggle_view() {
                self.view_format = self.view_format.toggled();
                self.view_loaded = true;
                self.scroll_anchor = None;
                self.write_view_format(ui, self.view_format);
                self.sync(ui);
            }
            return;
        }
        if self.playing {
            return;
        }
        if self.neutral {
            return;
        }
        if self.local_action(ui, action) {
            self.sync(ui);
            return;
        }
        if matches!(self.sheet, Some(Sheet::Profiles | Sheet::ProfileNames)) {
            match action {
                Action::Back => {
                    self.sheet = None;
                    self.focus = self.invoking;
                }
                Action::Up | Action::Down
                    if matches!(self.sheet, Some(Sheet::Profiles)) && self.focus >= 1000 =>
                {
                    let count = self.item().map_or(0, |i| i.profiles.len()) as i32;
                    if action == Action::Up && self.focus > 1000 {
                        self.focus -= 1;
                    } else if action == Action::Down && self.focus < 1000 + count - 1 {
                        self.focus += 1;
                    } else if action == Action::Down {
                        self.focus = 912;
                    }
                }
                Action::Up
                    if matches!(self.sheet, Some(Sheet::Profiles))
                        && matches!(self.focus, 912 | 913) =>
                {
                    self.focus = 1000 + self.item().map_or(1, |i| i.profiles.len()) as i32 - 1;
                }
                Action::Up | Action::Down | Action::Left | Action::Right => {
                    let saved =
                        std::mem::replace(&mut self.nodes, ui.get_sheet_nodes().iter().collect());
                    self.move_focus(ui, action);
                    self.nodes = saved;
                }
                Action::PreviousTab
                    if matches!(self.sheet, Some(Sheet::Profiles)) && !self.busy =>
                {
                    self.switch_profile(ui, false)
                }
                Action::NextTab if matches!(self.sheet, Some(Sheet::Profiles)) && !self.busy => {
                    self.switch_profile(ui, true)
                }
                Action::Confirm if !self.busy => self.profile_action(ui),
                _ => {}
            }
            self.sync(ui);
            return;
        }
        if self.screen == Screen::Setup && self.focus == 60 && !self.busy {
            match action {
                Action::PreviousTab | Action::NextTab => {
                    self.switch_profile(ui, action == Action::NextTab);
                    self.sync(ui);
                    return;
                }
                _ => {}
            }
        }
        if self.sheet.is_some() {
            match action {
                Action::Up | Action::Down | Action::Left | Action::Right => {
                    let saved =
                        std::mem::replace(&mut self.nodes, ui.get_sheet_nodes().iter().collect());
                    self.move_focus(ui, action);
                    self.nodes = saved;
                }
                Action::Back => {
                    self.sheet = None;
                    self.focus = self.invoking;
                }
                // R2: the uninstall checkbox is toggled with the controller.
                Action::Confirm if self.focus == 902 => {
                    self.uninstall_delete_profiles = !self.uninstall_delete_profiles;
                }
                Action::Confirm => {
                    let sheet = self.sheet.take().unwrap();
                    // Quit and the import offer put their confirming action on
                    // the left, focused first.
                    let confirm = self.focus
                        == if matches!(sheet, Sheet::Quit | Sheet::ImportOffer) {
                            900
                        } else {
                            901
                        };
                    let delete_profiles = self.uninstall_delete_profiles;
                    self.focus = self.invoking;
                    if matches!(sheet, Sheet::ImportOffer) && !confirm {
                        // "No, don't ask again" is remembered; B only closes
                        // the offer until the next start.
                        services::decline_legacy(&self.env);
                        self.import_offer = None;
                    }
                    if confirm {
                        match sheet {
                            Sheet::Quit => ui.invoke_quit_requested(),
                            Sheet::Cancel => self.cancel.cancel(),
                            Sheet::Reset => self.start(ui, Command::Reset),
                            Sheet::StartFresh => {
                                self.recovery_reset = true;
                                self.start(ui, Command::Startup);
                            }
                            Sheet::Remove(path) => self.start(ui, Command::RemoveFolder(path)),
                            Sheet::Component(id) => self.toggle(ui, &id),
                            Sheet::UserComponent(id) => {
                                // Spec 094 R4: a required player-supplied main has
                                // no off switch, so confirming the sheet only closes
                                // it (after an import or replace).
                                let required = self.item().is_some_and(|item| {
                                    item.recipe.components.iter().any(|component| {
                                        component.id.as_str() == id.as_str() && !component.optional
                                    })
                                });
                                if !required {
                                    self.toggle(ui, &id);
                                }
                            }
                            Sheet::DeleteProfile(id, _) => {
                                self.start(ui, Command::DeleteProfile(id))
                            }
                            Sheet::Uninstall(id) => {
                                self.start(ui, Command::Uninstall(id, delete_profiles));
                            }
                            Sheet::ImportOffer => {
                                self.import_now = true;
                                self.start(ui, Command::Startup);
                            }
                            Sheet::Profiles | Sheet::ProfileNames => {}
                        }
                    } else if let Sheet::UserComponent(id) = sheet {
                        // The left button imports a replacement even though the
                        // row already has an imported file. It must not toggle.
                        self.browse(ui, Some(id));
                    }
                }
                _ => {}
            }
            self.sync(ui);
            return;
        }
        match action {
            Action::Confirm => self.activate(ui),
            Action::Back => {
                self.problem = None;
                if self.screen == Screen::Recovery {
                    self.show_sheet(Sheet::Quit);
                } else if self.screen == Screen::Progress {
                    (self.screen, self.focus) = self.return_to;
                } else if self.screen == Screen::Browser {
                    self.import_component = None;
                    (self.screen, self.focus) = self.history.pop().unwrap_or(self.return_to);
                } else if self.screen == Screen::Settings && self.focus >= 70 {
                    self.focus = 1 + self.settings_section as i32;
                } else if self.root() && self.history.is_empty() {
                    self.show_sheet(Sheet::Quit);
                } else if let Some(place) = self.history.pop() {
                    (self.screen, self.focus) = place;
                } else if self.screen == Screen::Review {
                    self.screen = Screen::FirstRun;
                    self.focus = 2;
                }
            }
            Action::Left | Action::Right
                if self.screen == Screen::Discover
                    && (CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus) =>
            {
                self.step_category(if action == Action::Right { 1 } else { -1 });
            }
            // The strip is entered at the selected chip and left for the first card, so
            // the way in and out never depends on column geometry.
            Action::Up if self.screen == Screen::Discover && card_index(self.focus).is_some() => {
                self.move_focus(ui, action);
                if (CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus) {
                    self.focus = CHIP + self.category_index() as i32;
                }
            }
            Action::Down
                if self.screen == Screen::Discover
                    && (CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus)
                    && !self.discover_items().is_empty() =>
            {
                self.focus = card_id(self.discover_items()[0]);
            }
            Action::Up | Action::Down | Action::Left | Action::Right => self.move_focus(ui, action),
            Action::PreviousCategory | Action::NextCategory if self.screen == Screen::Discover => {
                self.step_category(if action == Action::NextCategory {
                    1
                } else {
                    -1
                });
            }
            Action::X => self.toggle_favorite(ui),
            Action::Tab(tab) => self.tab(ui, tab),
            Action::PreviousTab | Action::NextTab => {
                let tab = self.tab_index();
                self.tab(
                    ui,
                    (tab + if action == Action::NextTab { 1 } else { 3 }) % 4,
                );
            }
            Action::Menu => {
                if self.busy {
                    if self.screen != Screen::Progress {
                        self.enter(Screen::Progress, 1);
                    }
                } else {
                    self.show_sheet(Sheet::Quit);
                }
            }
            _ => {}
        }
        self.sync(ui);
    }
    /// Require a neutral poll after the child exits before accepting gamepad input.
    pub fn neutral_poll(&mut self, neutral: bool) {
        if !self.playing && neutral {
            self.neutral = false;
        }
    }
    pub fn click(&mut self, ui: &AppWindow, id: i32) {
        if self.playing {
            return;
        }
        if self.sheet.is_some() || self.local.dialog.is_some() {
            if ui
                .get_sheet_nodes()
                .iter()
                .any(|n| n.id == id && n.actionable)
            {
                self.focus = id;
                self.action(ui, Action::Confirm);
            }
        } else if self.nodes.iter().any(|n| n.id == id && n.actionable) {
            self.neutral = false;
            self.focus = id;
            self.action(ui, Action::Confirm);
        }
    }
    fn browse(&mut self, ui: &AppWindow, component: Option<String>) {
        self.import_component = component;
        self.enter(Screen::Browser, 1);
        self.start(
            ui,
            Command::Browse(self.env.home.clone(), self.import_component.is_some()),
        );
    }
    fn toggle(&mut self, ui: &AppWindow, id: &str) {
        if let Some(item) = self.item() {
            let mut p = item.profile.clone();
            services::select_component(&item.recipe, &mut p, id);
            self.start(ui, Command::Save(p));
        }
    }
    fn activate(&mut self, ui: &AppWindow) {
        if self.focus < 0 && self.root() {
            self.tab(ui, (-self.focus - 1) as usize);
            return;
        }
        if self.screen == Screen::Recovery && self.focus == 802 && !self.busy {
            self.show_sheet(Sheet::StartFresh);
            return;
        }
        if self.focus == 800 {
            if self.screen == Screen::Recovery {
                self.start(ui, Command::Startup);
            } else if let Some(command) = self.retry.clone() {
                self.start(ui, command);
            }
            return;
        }
        if self.focus == 801 {
            self.enter(Screen::Settings, 5);
            self.settings_section = 4;
            return;
        }
        if self.busy {
            // Spec 075, R2: the Pistol row keeps a repeated press while a
            // command runs; `set_pistol_start` queues the last intent.
            if self.screen == Screen::Setup && self.focus == 41 {
                if let Some(item) = self.item() {
                    let recipe_id = item.recipe.id.clone();
                    let profile_id = item.profile.id.clone();
                    let value = !item.profile.pistol_start;
                    self.set_pistol_start(ui, recipe_id, profile_id, value);
                }
                self.sync(ui);
                return;
            }
            if self.screen == Screen::Progress
                && self.focus == 1
                && self
                    .progress
                    .is_none_or(|p| p.stage != InstallStage::Commit)
            {
                self.show_sheet(Sheet::Cancel);
            }
            return;
        }
        match (self.screen, self.focus) {
            (Screen::FirstRun, 1) => self.start(ui, Command::InstallEngine("0.30.1".into())),
            (Screen::FirstRun, 2) | (Screen::Settings, 70)
                if self.screen == Screen::FirstRun || self.settings_section == 2 =>
            {
                self.browse(ui, None)
            }
            (Screen::FirstRun, 3) => self.start(ui, Command::Scan(None)),
            (Screen::FirstRun, 4) => {
                self.screen = Screen::Discover;
                self.focus = self.discover_entry_focus();
            }
            (Screen::FirstRun, 5) => {
                self.screen = Screen::Home;
                self.focus = 1;
            }
            (Screen::Home, 1) => {
                let latest = self.snapshot.as_ref().and_then(|s| {
                    s.items
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| item.last_played.is_some())
                        .max_by_key(|(_, item)| item.last_played)
                        .map(|(index, item)| {
                            (
                                index,
                                item.recipe.id.clone(),
                                item.launch.is_ok() && s.config_usable && s.engine.is_some(),
                            )
                        })
                });
                if let Some((index, id, ready)) = latest {
                    self.selected = index;
                    if ready {
                        // Spec 064 R1: Continue is the remembered game/profile,
                        // launched through the same validation as Play setup.
                        self.start(ui, Command::Play(id));
                    } else {
                        self.enter(Screen::Setup, 50);
                    }
                } else {
                    self.screen = Screen::Discover;
                    self.focus = self.discover_entry_focus();
                }
            }
            (Screen::Home, 2) => {
                self.screen = Screen::Discover;
                self.focus = self.discover_entry_focus();
            }
            (Screen::Discover, id) if (CHIP..CHIP + CHIPS_MAX as i32).contains(&id) => {
                if let Some(category) = self.categories().get((id - CHIP) as usize) {
                    self.category = category.clone();
                }
                if let Some(&first) = self.discover_items().first() {
                    self.focus = card_id(first);
                }
            }
            (Screen::Discover, BROWSE) => {
                if let Some(first) = self
                    .categories()
                    .into_iter()
                    .find(|c| matches!(c, Category::Tag(_)))
                {
                    self.category = first;
                    self.focus = self.discover_entry_focus();
                }
            }
            (Screen::Discover | Screen::Library, id) if card_index(id).is_some() => {
                if self.screen == Screen::Library
                    && self
                        .snapshot
                        .as_ref()
                        .is_some_and(|s| s.items.iter().all(|i| i.manifest.is_none()))
                {
                    self.screen = Screen::Discover;
                    self.focus = self.discover_entry_focus();
                } else {
                    self.selected = card_index(self.focus).unwrap();
                    self.enter(Screen::Detail, 1);
                }
            }
            (Screen::Detail, 1) => {
                let action = self.item().map(|item| {
                    (
                        item.recipe.id.clone(),
                        item.manifest.is_some(),
                        item.launch.is_ok(),
                    )
                });
                if let Some((id, installed, launchable)) = action {
                    if !installed {
                        self.start(ui, Command::Install(id));
                    } else {
                        let ready = launchable
                            && self
                                .snapshot
                                .as_ref()
                                .is_some_and(|s| s.config_usable && s.engine.is_some());
                        if ready {
                            // Spec 064 R1: the main action plays directly; Edit
                            // still opens Play setup.
                            self.start(ui, Command::Play(id));
                        } else {
                            self.enter(Screen::Setup, 50);
                        }
                    }
                }
            }
            (Screen::Detail, 6) => self.enter(Screen::Setup, 60),
            (Screen::Setup, 60) => {
                if let Some(item) = self.item() {
                    // Materialise an implicit Default before any management action.
                    self.start(ui, Command::Save(item.profile.clone()));
                    self.show_sheet(Sheet::Profiles);
                }
            }
            (Screen::Detail, 5) => {
                if let Some(item) = self.item()
                    && item.manifest.is_some()
                {
                    self.show_sheet(Sheet::Uninstall(item.recipe.id.clone()));
                }
            }
            (Screen::Detail, 2) => {
                if let Some(item) = self.item() {
                    if matches!(
                        item.recipe_status,
                        ddl_core::library::RecipeStatus::Update(_)
                    ) {
                        self.start(ui, Command::UpdateMapset(item.recipe.id.clone()));
                    } else {
                        self.enter(Screen::Setup, 50);
                    }
                }
            }
            (Screen::Detail, 3) => {
                if let Some(item) = self.item() {
                    if matches!(
                        item.recipe_status,
                        ddl_core::library::RecipeStatus::Update(_)
                    ) {
                        self.start(ui, Command::KeepCurrent(item.recipe.id.clone()));
                    } else {
                        self.toggle_favorite(ui);
                    }
                }
            }
            (Screen::Setup, 1) => {
                if self.snapshot.as_ref().is_some_and(|s| !s.config_usable) {
                    self.enter(Screen::Settings, 70);
                    self.settings_section = 3;
                } else if self.snapshot.as_ref().is_some_and(|s| s.engine.is_none()) {
                    self.start(ui, Command::InstallEngine("0.30.1".into()));
                } else if let Some(Blocker::ComponentNotProvided { component, .. }) =
                    self.item().and_then(|i| i.launch.as_ref().err()).cloned()
                {
                    self.browse(ui, Some(component));
                } else {
                    self.browse(ui, None);
                }
            }
            (Screen::Setup, 2) => {
                if let Some(item) = self.item() {
                    self.start(ui, Command::Install(item.recipe.id.clone()));
                }
            }
            (Screen::Setup, 40) => {
                if let Some(item) = self.item() {
                    let mut profile = item.profile.clone();
                    profile.skill = profile.skill % 5 + 1;
                    self.start(ui, Command::Save(profile));
                }
            }
            (Screen::Setup, 41) => {
                if let Some(item) = self.item() {
                    let recipe_id = item.recipe.id.clone();
                    let profile_id = item.profile.id.clone();
                    let value = !item.profile.pistol_start;
                    self.set_pistol_start(ui, recipe_id, profile_id, value);
                }
            }
            (Screen::Setup, 42) => self.advanced = !self.advanced,
            (Screen::Setup, 43) => ui.invoke_show_toast(text(ui, T::OrderHelp)),
            (Screen::Setup, 100..=199) => {
                if let Some(c) = self
                    .item()
                    .and_then(|i| i.recipe.components.get((self.focus - 100) as usize))
                    .cloned()
                {
                    if let Some(group) = c
                        .exclusive_group
                        .as_ref()
                        .filter(|_| c.optional && c.replaces.is_none() && !c.user_supplied)
                    {
                        if let Some(item) = self.item() {
                            if item.profile.components.contains(&c.id) {
                                return;
                            }
                            let mut profile = item.profile.clone();
                            services::choose_exclusive_component(
                                &item.recipe,
                                &mut profile,
                                group,
                                Some(&c.id),
                            );
                            self.start(ui, Command::Save(profile));
                        }
                    } else if c.user_supplied {
                        let imported = self
                            .item()
                            .and_then(|item| item.manifest.as_ref())
                            .is_some_and(|manifest| manifest.find_imported(&c.id).is_some());
                        if imported {
                            self.show_sheet(Sheet::UserComponent(c.id));
                        } else {
                            self.browse(ui, Some(c.id));
                        }
                    } else if c.replaces.is_some()
                        && self
                            .item()
                            .is_some_and(|i| !i.profile.components.contains(&c.id))
                    {
                        self.show_sheet(Sheet::Component(c.id));
                    } else {
                        self.toggle(ui, &c.id);
                    }
                }
            }
            (Screen::Setup, 300..=399) => {
                if let Some(item) = self.item()
                    && let Some(group) = item
                        .recipe
                        .components
                        .get((self.focus - 300) as usize)
                        .and_then(|c| c.exclusive_group.as_deref())
                {
                    let mut profile = item.profile.clone();
                    services::choose_exclusive_component(&item.recipe, &mut profile, group, None);
                    self.start(ui, Command::Save(profile));
                }
            }
            (Screen::Setup, 50) => {
                if let Some(item) = self.item()
                    && item.launch.is_ok()
                    && self.snapshot.as_ref().is_some_and(|s| s.config_usable)
                {
                    self.start(ui, Command::Play(item.recipe.id.clone()));
                }
            }
            (Screen::Review, 1) => {
                (self.screen, self.focus) = self.review_return;
            }
            (Screen::Review, 10..=199) => {
                if let Some(entry) = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.iwads.iter().find(|e| !e.candidates.is_empty()))
                    && let Some(variant) = entry.candidates.get((self.focus - 10) as usize)
                {
                    self.start(ui, Command::Choose(entry.path.clone(), variant.clone()));
                }
            }
            (Screen::Browser, 1) => {
                if self.import_component.is_none()
                    && let Some(d) = &self.directory
                {
                    self.start(ui, Command::Scan(Some(d.path.clone())));
                }
            }
            (Screen::Browser, 2) => {
                if let Some(p) = self
                    .directory
                    .as_ref()
                    .and_then(|d| d.path.parent())
                    .map(PathBuf::from)
                {
                    self.start(ui, Command::Browse(p, self.import_component.is_some()));
                }
            }
            (Screen::Browser, 10..=99) => {
                if let Some(p) = self
                    .directory
                    .as_ref()
                    .and_then(|d| d.shortcuts.get((self.focus - 10) as usize))
                    .cloned()
                {
                    self.start(ui, Command::Browse(p, self.import_component.is_some()));
                }
            }
            (Screen::Browser, 100..) => {
                if let Some((p, dir)) = self
                    .directory
                    .as_ref()
                    .and_then(|d| d.entries.get((self.focus - 100) as usize))
                    .cloned()
                {
                    if dir {
                        self.start(ui, Command::Browse(p, self.import_component.is_some()));
                    } else if let (Some(component), Some(item)) =
                        (self.import_component.clone(), self.item())
                    {
                        self.start(ui, Command::Import(item.recipe.id.clone(), component, p));
                    }
                }
            }
            (Screen::Progress, 1) => self.show_sheet(Sheet::Cancel),
            (Screen::Settings, 1..=5) => {
                self.settings_section = (self.focus - 1) as usize;
                self.focus = 70;
            }
            (Screen::Settings, 70) => match self.settings_section {
                0 => {
                    if self.snapshot.as_ref().is_some_and(|s| s.engine.is_none()) {
                        self.start(ui, Command::InstallEngine("0.30.1".into()));
                    } else {
                        self.start(ui, Command::Updates)
                    }
                }
                1 => self.start(ui, Command::SelectSoundfont(Choice::Default)),
                3 => self.show_sheet(Sheet::Reset),
                _ => {}
            },
            // Spec 067 R2/R4: Arachno is selectable only after a reviewed install;
            // the action beside it downloads, reinstalls or retries without
            // selecting automatically.
            (Screen::Settings, 71) if self.settings_section == 1 => {
                if self.snapshot.as_ref().is_some_and(|s| s.audio.installed) {
                    self.start(ui, Command::SelectSoundfont(Choice::Arachno));
                }
            }
            (Screen::Settings, 72) if self.settings_section == 1 => {
                self.start(ui, Command::InstallSoundfont);
            }
            (Screen::Settings, 73) if self.settings_section == 1 => {
                if osk::open_url(&osk::SystemLauncher, ddl_core::soundfonts::TERMS).is_err() {
                    ui.invoke_show_toast(text(ui, T::AudioLinkFailed));
                }
            }
            (Screen::Settings, 71) if self.settings_section == 0 => {
                self.start(ui, Command::Rollback)
            }
            (Screen::Settings, 71) if self.settings_section == 2 => {
                self.start(ui, Command::Scan(None))
            }
            (Screen::Settings, 72) if self.settings_section == 0 => {
                if let Some(version) = self.updates.first() {
                    self.start(ui, Command::InstallEngine(version.clone()));
                }
            }
            (Screen::Settings, 72) if self.settings_section == 2 => {
                if let Some(s) = &self.snapshot {
                    self.start(
                        ui,
                        Command::SetSearchStandardFolders(!s.search_standard_folders),
                    );
                }
            }
            // Spec 019 R5: manual catalog check and the automatic toggle.
            (Screen::Settings, 73) if self.settings_section == 0 => {
                self.start(ui, Command::CatalogCheck)
            }
            (Screen::Settings, 74) if self.settings_section == 0 => {
                if let Some(s) = &self.snapshot {
                    self.start(ui, Command::SetCatalogAutoUpdate(!s.catalog.auto_update));
                }
            }
            (Screen::Settings, id) if self.settings_section == 2 && id >= 1000 => {
                if let Some(folder) = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.automatic_folders.get((id - 1000) as usize))
                {
                    ui.invoke_show_toast(
                        format!("{} · {}", folder.source.label(), folder.path.display()).into(),
                    );
                }
            }
            (Screen::Settings, 100..) => {
                if self.settings_section == 2
                    && let Some(path) = self
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.folders.get((self.focus - 100) as usize))
                        .cloned()
                {
                    self.show_sheet(Sheet::Remove(path));
                }
            }
            _ => {}
        }
    }
    /// The strip (spec 072 R1): requested specific categories some recipe uses,
    /// in the owner's order, then `Favorites`. Never empty.
    fn categories(&self) -> Vec<Category> {
        let mut list = self.snapshot.as_ref().map_or_else(Vec::new, |s| {
            // Spec 090 R1: a category with no visible recipe is not offered, so
            // Community Projects disappears only if nothing but DBPs used it.
            requested_categories(|id| s.items.iter().any(|i| shows_in_category(&i.recipe, id)))
        });
        list.push(Category::Favorites);
        list
    }
    fn category_label(&self, ui: &AppWindow, category: &Category) -> String {
        match category {
            Category::Favorites => text(ui, T::FavoritesCategory).into(),
            Category::Tag(id) => self
                .snapshot
                .as_ref()
                .and_then(|s| s.tags.get(id))
                .map_or_else(|| id.clone(), |tag| tag.label.clone()),
        }
    }
    fn category_index(&self) -> usize {
        self.categories()
            .iter()
            .position(|c| *c == self.category)
            .unwrap_or(0)
    }
    /// Where Discover starts: the first shown card, else the selected chip.
    fn discover_entry_focus(&self) -> i32 {
        self.discover_items()
            .first()
            .map_or(CHIP + self.category_index() as i32, |&i| card_id(i))
    }
    /// Snapshot indexes shown on Discover in the current category, oldest first
    /// (spec 090, R1/R2). Community Projects hides `doomer-boards` recipes, and a
    /// favorite never moves a card: chronology decides every position.
    fn discover_items(&self) -> Vec<usize> {
        let Some(s) = &self.snapshot else {
            return vec![];
        };
        let mut shown: Vec<usize> = (0..s.items.len())
            .filter(|&i| match &self.category {
                Category::Tag(id) => shows_in_category(&s.items[i].recipe, id),
                Category::Favorites => s.items[i].favorite,
            })
            .collect();
        shown.sort_by(|&a, &b| compare_browsing_order(&s.items[a].recipe, &s.items[b].recipe));
        shown
    }
    /// Installed favorites lead Library in both layouts (spec 102), retaining
    /// the chronological/title order within each group.
    fn library_items(&self) -> Vec<usize> {
        let Some(s) = &self.snapshot else {
            return vec![];
        };
        let mut shown: Vec<usize> = (0..s.items.len())
            .filter(|&i| s.items[i].manifest.is_some())
            .collect();
        shown.sort_by(|&a, &b| {
            s.items[b]
                .favorite
                .cmp(&s.items[a].favorite)
                .then_with(|| compare_browsing_order(&s.items[a].recipe, &s.items[b].recipe))
        });
        shown
    }
    /// Moves the category by `delta` with wrapping; the strip chip keeps focus
    /// when it had it, otherwise focus enters the new grid (or the chip when empty).
    fn step_category(&mut self, delta: isize) {
        let list = self.categories();
        if list.is_empty() {
            return;
        }
        let next = (self.category_index() as isize + delta).rem_euclid(list.len() as isize);
        self.category = list[next as usize].clone();
        if !(CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus) {
            self.focus = self
                .discover_items()
                .first()
                .map_or(CHIP + next as i32, |&i| card_id(i));
        } else {
            self.focus = CHIP + next as i32;
        }
    }
    /// Spec 075, R1/R2: toggle Pistol start through the light profile path.
    ///
    /// The row and the launch summary flip immediately; the latest requested
    /// value is kept per profile in `pistol_intent`, separate from the last
    /// value a save confirmed (`pistol_baseline`), so an older save result can
    /// never invert a newer press. A press while a command runs is queued and
    /// reapplied after the result.
    fn set_pistol_start(
        &mut self,
        ui: &AppWindow,
        recipe_id: String,
        profile_id: String,
        value: bool,
    ) {
        let Some(previous) = self.patch_pistol_value(&profile_id, value) else {
            return;
        };
        // The row's value before this press is the last persisted one until a
        // save says otherwise; it is the fallback a failure restores.
        self.pistol_baseline
            .entry(profile_id.clone())
            .or_insert(previous);
        self.pistol_intent.insert(profile_id.clone(), value);
        if self.busy {
            self.pistol_queue = Some((recipe_id, profile_id));
            return;
        }
        self.dispatch_pistol_start(ui, recipe_id, profile_id);
    }
    /// Spec 075, R2: dispatch the latest intent for one catalog-mapset profile.
    fn dispatch_pistol_start(&mut self, ui: &AppWindow, recipe_id: String, profile_id: String) {
        let Some(value) = self.pistol_intent.get(&profile_id).copied() else {
            return;
        };
        self.pistol_in_flight = true;
        self.start(
            ui,
            Command::SetPistolStart {
                recipe_id,
                profile_id,
                value,
            },
        );
    }
    /// Spec 075, R1/R2: the base-game Pistol start toggle (local UI), the second
    /// production control, through the same light path.
    fn set_base_pistol_start(&mut self, ui: &AppWindow, profile_id: String, value: bool) {
        let Some(previous) = self.patch_pistol_value(&profile_id, value) else {
            return;
        };
        self.pistol_baseline
            .entry(profile_id.clone())
            .or_insert(previous);
        self.pistol_intent.insert(profile_id.clone(), value);
        if self.busy {
            self.base_pistol_queue = Some(profile_id);
            return;
        }
        self.dispatch_base_pistol_start(ui, profile_id);
    }
    /// Spec 075, R2: dispatch the latest intent for one base-game profile.
    fn dispatch_base_pistol_start(&mut self, ui: &AppWindow, profile_id: String) {
        let Some(value) = self.pistol_intent.get(&profile_id).copied() else {
            return;
        };
        self.pistol_in_flight = true;
        self.start(
            ui,
            Command::Local(services::LocalCommand::BasePistolStart { profile_id, value }),
        );
    }
    /// Spec 075, R2: applies a save result. The saved value becomes the
    /// baseline, the row takes the saved profile, and the latest intent is
    /// re-applied on top, so an older result can never invert a newer press.
    /// The selected-profile mirrors follow both.
    fn apply_saved_profile(&mut self, profile: &Profile) {
        let saved = profile.pistol_start;
        self.pistol_baseline.insert(profile.id.clone(), saved);
        let intent = self.pistol_intent.get(&profile.id).copied();
        if let Some(snapshot) = self.snapshot.as_mut() {
            if let Some(item) = snapshot
                .items
                .iter_mut()
                .find(|i| i.profile.id == profile.id)
            {
                item.profile = profile.clone();
                for held in &mut item.profiles {
                    if held.id == profile.id {
                        held.pistol_start = saved;
                    }
                }
                if let Ok(summary) = &mut item.launch {
                    summary.pistol_start = saved;
                }
            } else if let Some(base) = snapshot
                .base_games
                .iter_mut()
                .find(|b| b.profile.id == profile.id)
            {
                base.profile = profile.clone();
                for held in &mut base.profiles {
                    if held.id == profile.id {
                        held.pistol_start = saved;
                    }
                }
            }
        }
        if let Some(intent) = intent
            && intent != saved
        {
            self.patch_pistol_value(&profile.id, intent);
        }
        self.pistol_in_flight = false;
    }

    /// Spec 075, R2: writes the visible value into the snapshot row (catalog
    /// mapset or base game) and its launch summary; returns the value it
    /// replaced, or `None` when no row owns the profile.
    fn patch_pistol_value(&mut self, profile_id: &str, value: bool) -> Option<bool> {
        let snapshot = self.snapshot.as_mut()?;
        if let Some(item) = snapshot
            .items
            .iter_mut()
            .find(|i| i.profile.id == profile_id)
        {
            let previous = item.profile.pistol_start;
            item.profile.pistol_start = value;
            for held in &mut item.profiles {
                if held.id == item.profile.id {
                    held.pistol_start = value;
                }
            }
            if let Ok(summary) = &mut item.launch {
                summary.pistol_start = value;
            }
            return Some(previous);
        }
        let base = snapshot
            .base_games
            .iter_mut()
            .find(|b| b.profile.id == profile_id)?;
        let previous = base.profile.pistol_start;
        base.profile.pistol_start = value;
        for held in &mut base.profiles {
            if held.id == base.profile.id {
                held.pistol_start = value;
            }
        }
        Some(previous)
    }
    /// Spec 075, R2: a failed write restores the **last value a save confirmed**
    /// (not the value of whichever press happened to be first), drops only that
    /// profile's intent and queue entry, and keeps other profiles' intents.
    fn revert_pistol_start(&mut self, profile_id: &str) {
        self.pistol_in_flight = false;
        let previous = self
            .pistol_baseline
            .get(profile_id)
            .copied()
            .unwrap_or(false);
        self.pistol_intent.remove(profile_id);
        self.pistol_queue = self
            .pistol_queue
            .take()
            .filter(|(_, queued)| queued != profile_id);
        self.base_pistol_queue = self
            .base_pistol_queue
            .take()
            .filter(|queued| queued != profile_id);
        self.patch_pistol_value(profile_id, previous);
    }
    /// Spec 075, R2: dispatches at most one pending pistol intent; called when
    /// the worker is free so a queued press (or a save result overtaken by a
    /// newer press) is applied in order.
    fn settle_pistol(&mut self, ui: &AppWindow) {
        if self.busy || self.pistol_in_flight {
            return;
        }
        if let Some((recipe_id, profile_id)) = self.pistol_queue.clone() {
            let value = self.pistol_intent.get(&profile_id).copied();
            if value.is_none() || value == self.pistol_baseline.get(&profile_id).copied() {
                self.pistol_queue = None;
            } else {
                self.pistol_queue = None;
                self.dispatch_pistol_start(ui, recipe_id, profile_id);
                return;
            }
        }
        if let Some(profile_id) = self.base_pistol_queue.clone() {
            let value = self.pistol_intent.get(&profile_id).copied();
            if value.is_none() || value == self.pistol_baseline.get(&profile_id).copied() {
                self.base_pistol_queue = None;
            } else {
                self.base_pistol_queue = None;
                self.dispatch_base_pistol_start(ui, profile_id);
            }
        }
    }
    /// X: toggles the favorite mark of the focused card, or of the open detail.
    fn toggle_favorite(&mut self, ui: &AppWindow) {
        if self.busy {
            return;
        }
        let target = match self.screen {
            Screen::Discover | Screen::Library if card_index(self.focus).is_some() => {
                Some(card_index(self.focus).unwrap())
            }
            Screen::Detail => Some(self.selected),
            _ => None,
        };
        let Some(index) = target else {
            return;
        };
        let Some(item) = self.snapshot.as_mut().and_then(|s| s.items.get_mut(index)) else {
            return;
        };
        // Optimistic: the saved state replaces the snapshot when the worker finishes.
        item.favorite = !item.favorite;
        let id = item.recipe.id.clone();
        self.start(ui, Command::Favorite(id));
    }
    fn tab_index(&self) -> usize {
        match self.screen {
            Screen::Discover => 2,
            Screen::Library => 1,
            Screen::Settings => 3,
            _ => 0,
        }
    }
    fn profile_action(&mut self, ui: &AppWindow) {
        let Some((target, profile, profiles)) = self.base_profile_context().or_else(|| {
            self.item().map(|item| {
                (
                    item.recipe.id.clone(),
                    item.profile.clone(),
                    item.profiles.clone(),
                )
            })
        }) else {
            return;
        };
        if matches!(self.sheet, Some(Sheet::ProfileNames)) {
            if self.focus == 917 {
                match ddl_core::profiles::validate_name(
                    &ui.get_profile_name(),
                    &target,
                    Some(&profile.id),
                    &profiles,
                ) {
                    Ok(name) => {
                        self.sheet = None;
                        self.focus = self.invoking;
                        self.start(ui, Command::RenameProfile(profile.id, name));
                    }
                    Err(reason) => ui.set_profile_name_error(reason.into()),
                }
            } else if self.focus == 916 {
                self.sheet = None;
                self.focus = self.invoking;
            }
            return;
        }
        match self.focus {
            id if id >= 1000 && (id - 1000) < profiles.len() as i32 => {
                self.start(
                    ui,
                    Command::SelectProfile(target, profiles[(id - 1000) as usize].id.clone()),
                );
            }
            912 | 913 => {
                let copy = self.focus == 912;
                self.sheet = None;
                self.focus = self.invoking;
                self.start(ui, Command::NewProfile(target, copy));
            }
            914 => {
                self.edit_profile_name(ui);
            }
            915 if profiles.len() > 1 => {
                self.sheet = Some(Sheet::DeleteProfile(profile.id, profile.name));
                self.focus = 900;
            }
            916 => {
                self.sheet = None;
                self.focus = self.invoking;
            }
            _ => {}
        }
    }
    fn edit_profile_name(&mut self, ui: &AppWindow) {
        let profile = self
            .base_profile_context()
            .map(|(_, profile, _)| profile)
            .or_else(|| self.item().map(|item| item.profile.clone()));
        if let Some(profile) = profile {
            ui.set_profile_name(profile.name.into());
            ui.set_profile_name_error("".into());
            self.sheet = Some(Sheet::ProfileNames);
            self.focus = 918;
        }
    }
    fn switch_profile(&mut self, ui: &AppWindow, next: bool) {
        let Some(item) = self.item() else { return };
        let count = item.profiles.len();
        if count == 0 {
            return;
        }
        let index = item
            .profiles
            .iter()
            .position(|p| p.id == item.profile.id)
            .unwrap_or(0);
        let index = if next {
            (index + 1) % count
        } else {
            (index + count - 1) % count
        };
        let command =
            Command::SelectProfile(item.recipe.id.clone(), item.profiles[index].id.clone());
        if matches!(self.sheet, Some(Sheet::Profiles)) {
            self.focus = 1000 + index as i32;
        }
        self.start(ui, command);
    }
    fn profile_sheet(&self, ui: &AppWindow) {
        let mut b = Nodes::new(ui);
        if matches!(self.sheet, Some(Sheet::ProfileNames)) {
            ui.set_sheet_title(text(ui, T::RenameProfile));
            // Invisible navigation anchor forwards controller focus to TextInput.
            b.button(918, 32., 140., 656., "".into(), false);
            b.nodes.last_mut().unwrap().kind = 0;
            b.button(917, 32., 360., 304., text(ui, T::SaveProfileName), false);
            b.button(916, 368., 360., 304., text(ui, T::Cancel), false);
        } else {
            let Some(item) = self.item() else { return };
            ui.set_sheet_title(text(ui, T::Profiles));
            let active = item
                .profiles
                .iter()
                .position(|p| p.id == item.profile.id)
                .unwrap_or(0);
            let selected = if self.focus >= 1000 {
                (self.focus - 1000) as usize
            } else {
                active
            };
            let start = selected
                .saturating_sub(2)
                .min(item.profiles.len().saturating_sub(3));
            for (index, profile) in item.profiles.iter().enumerate().skip(start).take(3) {
                b.button(
                    1000 + index as i32,
                    32.,
                    96. + (index - start) as f32 * 62.,
                    656.,
                    format!(
                        "{}{}",
                        if profile.id == item.profile.id {
                            "●  "
                        } else {
                            "    "
                        },
                        profile.name
                    )
                    .into(),
                    false,
                );
            }
            let actions_y = 116. + item.profiles.len().min(3) as f32 * 62.;
            b.button(912, 32., actions_y, 304., text(ui, T::CopyProfile), false);
            b.button(
                913,
                368.,
                actions_y,
                304.,
                text(ui, T::DefaultProfile),
                false,
            );
            b.button(
                914,
                32.,
                actions_y + 68.,
                304.,
                text(ui, T::RenameProfile),
                false,
            );
            b.button(
                915,
                368.,
                actions_y + 68.,
                304.,
                text(
                    ui,
                    if item.profiles.len() > 1 {
                        T::DeleteProfile
                    } else {
                        T::LastProfile
                    },
                ),
                false,
            );
            b.nodes.last_mut().unwrap().actionable = item.profiles.len() > 1;
            b.button(916, 32., actions_y + 136., 656., text(ui, T::Done), false);
        }
        ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
    }
    /// Spec 022 R2: the uninstall confirm in the existing Ember sheet, with a
    /// controller-toggleable checkbox, installed size and profile/import facts.
    fn uninstall_sheet(&self, ui: &AppWindow) {
        let Some(item) = self.item() else {
            return;
        };
        ui.set_sheet_title(value(ui, T::UninstallConfirm, display_title(&item.recipe)));
        let bytes = item
            .manifest
            .as_ref()
            .map_or(0, ddl_core::library::InstallManifest::installed_bytes);
        let imported = item
            .manifest
            .as_ref()
            .is_some_and(ddl_core::library::InstallManifest::has_imports);
        let mut b = Nodes::new(ui);
        b.label(
            32.,
            104.,
            656.,
            32.,
            value(ui, T::UninstallSize, human_bytes(bytes)),
            22.,
        );
        b.checkbox(
            902,
            32.,
            144.,
            656.,
            if item.profiles.len() == 1 {
                text(ui, T::UninstallProfilesOne)
            } else {
                value(ui, T::UninstallProfiles, item.profiles.len())
            },
            self.uninstall_delete_profiles,
        );
        if imported {
            b.label(32., 212., 656., 32., text(ui, T::UninstallImported), 20.);
        }
        b.label(
            32.,
            252.,
            656.,
            34.,
            text(
                ui,
                if self.uninstall_delete_profiles {
                    T::UninstallDeletes
                } else {
                    T::UninstallKeeps
                },
            ),
            20.,
        );
        b.button(900, 32., 312., 304., text(ui, T::Cancel), false);
        b.button(901, 368., 312., 304., text(ui, T::Uninstall), false);
        ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
    }
    fn move_focus(&mut self, ui: &AppWindow, direction: Action) {
        if self.focus < 0 && self.root() {
            match direction {
                Action::Left if self.focus < -1 => self.focus += 1,
                Action::Right if self.focus > -4 => self.focus -= 1,
                Action::Down => {
                    let tab = (-self.focus - 1) as usize;
                    self.tab(ui, tab);
                    self.focus = self.remembered[tab];
                }
                _ => {}
            }
            return;
        }
        let Some(current) = self
            .nodes
            .iter()
            .find(|n| n.id == self.focus && n.actionable)
        else {
            return;
        };
        let next = if matches!(direction, Action::Up | Action::Down) {
            crate::focus::vertical_target(&self.nodes, current, direction)
        } else {
            let (cx, cy) = (current.x + current.w / 2., current.y + current.h / 2.);
            self.nodes
                .iter()
                .filter(|n| n.actionable && n.id != self.focus)
                .filter_map(|n| {
                    let (dx, dy) = (n.x + n.w / 2. - cx, n.y + n.h / 2. - cy);
                    let (along, across) = match direction {
                        Action::Left => (-dx, dy.abs()),
                        Action::Right => (dx, dy.abs()),
                        _ => return None,
                    };
                    (along > 1.).then_some((n.id, along + across * 3.))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|n| n.0)
        };
        if let Some(next) = next {
            self.focus = next;
        } else if direction == Action::Up && self.root() && self.sheet.is_none() {
            self.remembered[self.tab_index()] = self.focus;
            self.focus = -(self.tab_index() as i32) - 1;
        }
    }
    /// Render-only seam: audit sheets without saving profiles or deleting files.
    #[cfg(test)]
    pub fn title_fit_sheet(&mut self, ui: &AppWindow, uninstall: bool) {
        let sheet = if uninstall {
            Sheet::Uninstall(self.item().unwrap().recipe.id.clone())
        } else {
            Sheet::Profiles
        };
        self.show_sheet(sheet);
        self.sync(ui);
    }
    #[cfg(test)]
    pub fn title_fit_advanced(&mut self) {
        self.advanced = true;
    }
    /// Render-only seam: simulate a remembered category after a catalog refresh.
    #[cfg(test)]
    pub fn choose_category(&mut self, category: Category) {
        self.category = category;
        self.category_chosen = true;
    }
    #[cfg(test)]
    pub fn chosen_category(&self) -> &Category {
        &self.category
    }
    pub fn sync(&mut self, ui: &AppWindow) {
        // Spec 072 R2: a new session starts on the first requested category a
        // recipe uses; a category that vanishes after a refresh falls back the
        // same way. `categories` always ends with Favorites, so this never indexes
        // an empty list, even with no catalog.
        let available = self.categories();
        let choosing_initial = !self.category_chosen && self.snapshot.is_some();
        if choosing_initial || !available.contains(&self.category) {
            self.category = first_available(&available);
            self.category_chosen = true;
            if self.screen == Screen::Discover
                && (card_index(self.focus).is_some()
                    || (CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus))
            {
                self.focus = self.discover_entry_focus();
            }
        }
        if self.screen == Screen::Settings && (1..=5).contains(&self.focus) {
            self.settings_section = (self.focus - 1) as usize;
        }
        let heading = if self.playing {
            T::Playing
        } else {
            match self.screen {
                Screen::Loading => T::Loading,
                Screen::FirstRun => T::FirstRun,
                Screen::Home => T::Home,
                Screen::Discover => T::Discover,
                Screen::Library => T::Library,
                Screen::Settings => T::Settings,
                Screen::Recovery => T::Recovery,
                Screen::Detail => T::Details,
                Screen::Setup => T::Setup,
                Screen::Progress => T::InstallTitle,
                Screen::Review => T::Review,
                Screen::Browser => {
                    if self.import_component.is_some() {
                        T::ChooseFile
                    } else {
                        T::ChooseFolder
                    }
                }
            }
        };
        let named = matches!(self.screen, Screen::Detail | Screen::Setup)
            .then(|| self.item())
            .flatten();
        ui.set_heading(if let Some(item) = named {
            ui.global::<crate::WiringStrings>()
                .set_value(text(ui, heading));
            ui.global::<crate::WiringStrings>()
                .set_other(display_title(&item.recipe).into());
            text(ui, T::ContextTitle)
        } else {
            text(ui, heading)
        });
        ui.set_art_band(
            matches!(
                self.screen,
                Screen::Home
                    | Screen::Discover
                    | Screen::Library
                    | Screen::Detail
                    | Screen::Setup
                    | Screen::Progress
            ) && !self.playing,
        );
        let band_item = if self.screen == Screen::Home {
            self.snapshot
                .as_ref()
                .and_then(|s| s.items.iter().max_by_key(|i| i.last_played))
        } else {
            self.item()
        };
        ui.set_band_art(band_item.map_or(4, |i| art(&i.recipe.id)));
        ui.set_band_screenshot(band_item.map_or_else(Default::default, |i| {
            crate::screenshots::presentation(&i.recipe)
        }));
        // A header is its title alone; the status line only reports a state,
        // a requirement or the next step (spec 033).
        let status = if self.playing {
            Some(T::PlayingSub)
        } else if self.screen == Screen::FirstRun {
            None
        } else if self.busy {
            Some(T::Working)
        } else {
            match self.screen {
                Screen::Review | Screen::Browser => Some(T::ScanHelp),
                Screen::Loading => Some(T::Loading),
                _ => None,
            }
        };
        if self.screen == Screen::Discover {
            // Spec 077 R3: the subtitle is the effective catalog's source
            // serial, never the last check time; an unknown serial is honest.
            ui.set_status(catalog_subtitle(
                self.snapshot.as_ref().map_or(0, |s| s.catalog.serial),
            ));
        } else {
            ui.set_status(status.map_or_else(Default::default, |key| text(ui, key)));
        }
        let mut b = Nodes::new(ui);
        if self.playing {
            b.panel(12., 12., 1200., 478.);
            b.label(40., 60., 1100., 100., text(ui, T::Playing), 40.);
            b.label(40., 186., 1100., 90., text(ui, T::PlayingHelp), 22.);
        } else {
            match self.screen {
                Screen::Loading => {
                    b.panel(12., 12., 1200., 240.);
                    b.logo(412., 24., 400., 210., true);
                }
                Screen::Recovery => {
                    let files = self.unreadable_text();
                    let rows = files
                        .lines()
                        .map(|line| line.chars().count().div_ceil(84).max(1))
                        .sum::<usize>();
                    let height = 100. + rows.max(2) as f32 * 32.;
                    b.panel(12., 12., 1200., height + 140.);
                    b.label(
                        36.,
                        32.,
                        1152.,
                        64.,
                        text(
                            ui,
                            if self.recovery_attempts > 1 {
                                T::StillUnreadable
                            } else {
                                T::RecoveryHelp
                            },
                        ),
                        24.,
                    );
                    b.label(36., 112., 1152., height - 100., files.into(), 22.);
                    b.button(800, 36., height + 32., 348., text(ui, T::Retry), false);
                    b.button(
                        802,
                        408.,
                        height + 32.,
                        384.,
                        text(ui, T::StartFresh),
                        false,
                    );
                    b.button(
                        801,
                        816.,
                        height + 32.,
                        372.,
                        text(ui, T::Diagnostics),
                        false,
                    );
                }
                Screen::FirstRun => {
                    b.panel(12., 12., 584., 478.);
                    b.panel(628., 12., 584., 478.);
                    b.label(36., 36., 536., 44., text(ui, T::Engine), 30.);
                    let ready = self.snapshot.as_ref().is_some_and(|s| s.engine.is_some());
                    b.label(
                        36.,
                        96.,
                        536.,
                        84.,
                        text(ui, if ready { T::EngineReady } else { T::FirstHelp }),
                        22.,
                    );
                    if !ready {
                        b.button(1, 36., 220., 536., text(ui, T::InstallEngine), false);
                    }
                    b.label(652., 36., 536., 44., text(ui, T::BaseGames), 30.);
                    b.label(652., 96., 536., 84., text(ui, T::ScanHelp), 22.);
                    b.button(2, 652., 208., 536., text(ui, T::AddFolder), false);
                    b.button(3, 652., 284., 536., text(ui, T::FindIwads), false);
                    b.button(4, 36., 402., 536., text(ui, T::BrowseWithout), false);
                    b.button(5, 652., 402., 536., text(ui, T::Finish), false);
                }
                Screen::Home => {
                    let latest = self.snapshot.as_ref().and_then(|s| {
                        s.items
                            .iter()
                            .filter(|i| i.last_played.is_some())
                            .max_by_key(|i| i.last_played)
                    });
                    b.panel(12., 12., 784., 478.);
                    b.panel(828., 12., 384., 478.);
                    if let Some(item) = latest {
                        b.recipe_art(36., 36., 240., 300., &item.recipe);
                        b.title(
                            308.,
                            56.,
                            448.,
                            68.,
                            display_title(&item.recipe).into(),
                            36.,
                        );
                        b.author(
                            308.,
                            132.,
                            448.,
                            36.,
                            item.recipe.authors.join(", ").into(),
                            22.,
                        );
                        b.label(
                            308.,
                            176.,
                            448.,
                            64.,
                            format!(
                                "{} · {}",
                                game(ui, item.recipe.base_game.game),
                                item.profile.name
                            )
                            .into(),
                            22.,
                        );
                        b.button(1, 308., 252., 448., text(ui, T::Continue), true);
                    } else {
                        b.label(36., 42., 720., 100., text(ui, T::NoHistory), 36.);
                        b.button(1, 36., 252., 720., text(ui, T::DiscoverAction), false);
                    }
                    if let Some(s) = &self.snapshot {
                        b.label(852., 36., 336., 48., text(ui, T::ReadyChecks), 30.);
                        b.check(
                            852.,
                            100.,
                            s.engine.as_ref().map_or_else(
                                || text(ui, T::NoEngine),
                                |e| value(ui, T::EngineVersion, &e.version),
                            ),
                            if s.engine.is_some() { 2 } else { 4 },
                        );
                        for (n, g) in [
                            ddl_core::catalog::Game::Doom,
                            ddl_core::catalog::Game::Doom2,
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            let found = s
                                .items
                                .iter()
                                .any(|i| i.recipe.base_game.game == g && base_ready(s, &i.recipe));
                            b.check(
                                852.,
                                172. + n as f32 * 72.,
                                value(
                                    ui,
                                    if found { T::FoundGame } else { T::MissingGame },
                                    game(ui, g),
                                ),
                                if found { 2 } else { 4 },
                            );
                        }
                    }
                    b.button(2, 852., 330., 336., text(ui, T::DiscoverAction), false);
                }
                Screen::Discover | Screen::Library => {
                    let discover = self.screen == Screen::Discover;
                    let shown = if discover {
                        self.discover_items()
                    } else {
                        self.library_items()
                    };
                    let top = if discover { STRIP_HEIGHT } else { 0. };
                    if let Some(s) = &self.snapshot {
                        for (visible, &i) in shown.iter().enumerate() {
                            if !discover && self.view_format == services::ViewFormat::List {
                                break;
                            }
                            let item = &s.items[i];
                            if self.view_format == services::ViewFormat::List {
                                let y = 12. + top + 82. * visible as f32;
                                let status = item_availability(ui, s, item);
                                b.compact(
                                    card_id(i),
                                    12.,
                                    y,
                                    display_title(&item.recipe).into(),
                                    format!(
                                        "{} · {status} · {}",
                                        game(ui, item.recipe.base_game.game),
                                        item.recipe.authors.join(", ")
                                    )
                                    .into(),
                                    text(
                                        ui,
                                        if item.manifest.is_some() {
                                            T::Installed
                                        } else {
                                            T::NotInstalled
                                        },
                                    ),
                                    item.favorite,
                                );
                                continue;
                            }
                            let x = 12. + 408. * (visible % 3) as f32;
                            let y = 12. + top + 438. * (visible / 3) as f32;
                            b.card(card_id(i), x, y, 384., 410.);
                            // Spec 074: a 28 px inset on every side keeps the art,
                            // text and chips clear of the 20 px ornamental frame and
                            // leaves room for two chip rows above the bottom edge.
                            b.recipe_art(x + 28., y + 28., 328., 128., &item.recipe);
                            if item.favorite {
                                b.favorite(x + 316., y + 36.);
                            }
                            b.title(
                                x + 28.,
                                y + 164.,
                                328.,
                                60.,
                                display_title(&item.recipe).into(),
                                26.,
                            );
                            b.author(
                                x + 28.,
                                y + 230.,
                                328.,
                                24.,
                                item.recipe.authors.join(", ").into(),
                                20.,
                            );
                            let status = text(
                                ui,
                                match item.recipe_status {
                                    ddl_core::library::RecipeStatus::Removed => T::RecipeRemoved,
                                    ddl_core::library::RecipeStatus::Update(_)
                                        if !item.update_dismissed =>
                                    {
                                        T::RecipeUpdate
                                    }
                                    _ if base_ready(s, &item.recipe) => T::BaseReady,
                                    _ => T::BaseMissingShort,
                                },
                            );
                            b.label(
                                x + 28.,
                                y + 258.,
                                328.,
                                24.,
                                format!("{} · {status}", game(ui, item.recipe.base_game.game))
                                    .into(),
                                18.,
                            );
                            b.label(
                                x + 28.,
                                y + 286.,
                                328.,
                                24.,
                                if discover {
                                    text(
                                        ui,
                                        if item.manifest.is_some() {
                                            T::Installed
                                        } else {
                                            T::NotInstalled
                                        },
                                    )
                                } else {
                                    profile_count(ui, item.profiles.len())
                                },
                                20.,
                            );
                            let lead = match &self.category {
                                Category::Tag(id) if discover => Some(id.as_str()),
                                _ => None,
                            };
                            b.chips(
                                &tag_labels(s, &item.recipe, 2, lead),
                                x + 28.,
                                y + 314.,
                                328.,
                            );
                        }
                        if discover && self.category == Category::Favorites && shown.is_empty() {
                            b.panel(12., 12. + top, 1200., 280.);
                            b.label(36., 36. + top, 1100., 70., text(ui, T::NoFavorites), 30.);
                            b.label(
                                36.,
                                112. + top,
                                1100.,
                                70.,
                                text(ui, T::NoFavoritesHelp),
                                22.,
                            );
                            if self
                                .categories()
                                .iter()
                                .any(|c| matches!(c, Category::Tag(_)))
                            {
                                b.button(
                                    BROWSE,
                                    36.,
                                    204. + top,
                                    400.,
                                    text(ui, T::BrowseMapsets),
                                    false,
                                );
                            }
                        } else if shown.is_empty() {
                            b.panel(12., 12., 1200., 300.);
                            b.label(36., 60., 1100., 70., text(ui, T::NoInstalled), 30.);
                            b.button(10, 36., 180., 400., text(ui, T::DiscoverAction), false);
                        }
                        if discover {
                            // After the cards, so the first actionable node stays a card.
                            let chips: Vec<(String, bool)> = self
                                .categories()
                                .iter()
                                .map(|c| (self.category_label(ui, c), *c == self.category))
                                .collect();
                            let selected = self.category_index();
                            self.strip_start = b.strip(
                                &chips,
                                selected,
                                self.strip_start,
                                if self.focus >= CHIP && self.focus < CHIP + CHIPS_MAX as i32 {
                                    Some((self.focus - CHIP) as usize)
                                } else {
                                    None
                                },
                            );
                        }
                    }
                }
                Screen::Detail => {
                    if let Some(item) = self.item() {
                        b.recipe_art(12., 12., 360., 262., &item.recipe);
                        if item.favorite {
                            b.favorite(316., 22.);
                        }
                        // Actions are laid out from the visible ones so the
                        // column never leaves an empty slot when the conditional
                        // Update action is absent (spec 022 review fix 2).
                        let updating = matches!(
                            item.recipe_status,
                            ddl_core::library::RecipeStatus::Update(_)
                        );
                        let installed = item.manifest.is_some();
                        let mut action_y = 298.;
                        b.button(
                            1,
                            12.,
                            action_y,
                            360.,
                            // Spec 064 R1: installed mapsets play directly; the
                            // Edit action opens Play setup.
                            text(ui, if installed { T::Play } else { T::Install }),
                            installed,
                        );
                        action_y += 68.;
                        // The second action is only the spec 020 update; the
                        // primary is never repeated (review fix 1).
                        if updating {
                            b.button(
                                2,
                                12.,
                                action_y,
                                360.,
                                text(ui, T::RecipeUpdateAction),
                                false,
                            );
                            action_y += 68.;
                        }
                        b.button(
                            3,
                            12.,
                            action_y,
                            360.,
                            text(
                                ui,
                                if updating {
                                    T::RecipeKeep
                                } else if item.favorite {
                                    T::FavoriteRemoveButton
                                } else {
                                    T::FavoriteAddButton
                                },
                            ),
                            false,
                        );
                        action_y += 68.;
                        // R1: Uninstall is offered only for installed mapsets,
                        // including ones no longer in the catalog (spec 020 R5).
                        if installed {
                            b.button(5, 12., action_y, 360., text(ui, T::Uninstall), false);
                        }
                        let detail_panel = b.nodes.len();
                        b.panel(404., 12., 808., 478.);
                        b.title(428., 24., 760., 96., item.recipe.title.clone().into(), 44.);
                        b.nodes.last_mut().unwrap().kind = 22;
                        b.author(
                            428.,
                            126.,
                            760.,
                            32.,
                            item.recipe.authors.join(", ").into(),
                            22.,
                        );
                        b.label(
                            428.,
                            166.,
                            760.,
                            68.,
                            match &item.recipe_status {
                                ddl_core::library::RecipeStatus::Update(_) => {
                                    value(ui, T::RecipeSummary, &item.change_summary)
                                }
                                ddl_core::library::RecipeStatus::Removed => {
                                    text(ui, T::RecipeRemoved)
                                }
                                _ if !item.update_notice.is_empty() => {
                                    value(ui, T::RecipeNotice, item.update_notice.join(" · "))
                                }
                                _ => item.recipe.description.clone().into(),
                            },
                            22.,
                        );
                        // The title's second line leaves a shorter description slot.
                        // Elide its last complete line rather than cutting a glyph.
                        b.nodes.last_mut().unwrap().kind = 23;
                        let mut y = 242.;
                        if let Some(s) = &self.snapshot {
                            let labels = tag_labels(s, &item.recipe, usize::MAX, None);
                            if !labels.is_empty() {
                                y += 38. * b.chips(&labels, 428., y, 760.) as f32 + 6.;
                            }
                        }
                        b.label(
                            428.,
                            y,
                            200.,
                            36.,
                            game(ui, item.recipe.base_game.game),
                            26.,
                        );
                        b.label(
                            650.,
                            y,
                            538.,
                            44.,
                            text(
                                ui,
                                if self
                                    .snapshot
                                    .as_ref()
                                    .is_some_and(|s| base_ready(s, &item.recipe))
                                {
                                    T::BaseReady
                                } else {
                                    T::BaseMissing
                                },
                            ),
                            22.,
                        );
                        b.label(
                            428.,
                            y + 52.,
                            600.,
                            32.,
                            value(ui, T::ActiveProfile, &item.profile.name),
                            22.,
                        );
                        b.label(
                            428.,
                            y + 84.,
                            600.,
                            56.,
                            text(ui, T::ProfileSummary)
                                .as_str()
                                .replace("{skill}", &item.profile.skill.to_string())
                                .replace(
                                    "{pistol}",
                                    text(
                                        ui,
                                        if item.profile.pistol_start {
                                            T::On
                                        } else {
                                            T::Off
                                        },
                                    )
                                    .as_str(),
                                )
                                .replace(
                                    "{addons}",
                                    &if item.profile.components.is_empty() {
                                        text(ui, T::None).to_string()
                                    } else {
                                        item.profile
                                            .components
                                            .iter()
                                            .map(|id| {
                                                item.recipe
                                                    .components
                                                    .iter()
                                                    .find(|c| &c.id == id)
                                                    .map_or(id.as_str(), |c| c.label.as_str())
                                            })
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    },
                                )
                                .replace("{extras}", &item.profile.extras.len().to_string())
                                .into(),
                            20.,
                        );
                        b.button(6, 1052., y + 52., 136., text(ui, T::EditProfile), false);
                        b.label(
                            428.,
                            y + 142.,
                            760.,
                            28.,
                            item.recipe.page.clone().into(),
                            20.,
                        );
                        let mut notes_y = y + 170.;
                        let credits = crate::screenshots::recipe_credits(&item.recipe);
                        if !credits.is_empty() {
                            b.label(
                                428.,
                                notes_y,
                                760.,
                                28.,
                                value(ui, T::ScreenshotsCredit, credits.join(", ")),
                                20.,
                            );
                            notes_y += 28.;
                        }
                        if matches!(&item.recipe_status, ddl_core::library::RecipeStatus::Update(c) if c.content)
                        {
                            b.label(428., notes_y, 760., 30., text(ui, T::RecipeSaves), 20.);
                        }
                        b.nodes[detail_panel].h = (notes_y + 34.).max(478.);
                    }
                }
                Screen::Setup => {
                    if let Some(item) = self.item() {
                        let options_panel = b.nodes.len();
                        b.panel(12., 12., 680., 478.);
                        b.panel(716., 12., 496., 478.);
                        b.row(
                            60,
                            36.,
                            36.,
                            632.,
                            value(ui, T::ActiveProfile, &item.profile.name),
                            profile_count(ui, item.profiles.len()),
                            true,
                        );
                        let mut y = 128.;
                        for (index, c) in item.recipe.components.iter().enumerate() {
                            if c.replaces.is_some() && !c.user_supplied && !self.advanced {
                                continue;
                            }
                            if let Some(group) = c
                                .exclusive_group
                                .as_ref()
                                .filter(|_| c.optional && c.replaces.is_none() && !c.user_supplied)
                            {
                                if item.recipe.components[..index]
                                    .iter()
                                    .any(|prior| prior.exclusive_group.as_ref() == Some(group))
                                {
                                    continue;
                                }
                                let members: Vec<_> = item
                                    .recipe
                                    .components
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, c)| {
                                        c.exclusive_group.as_ref() == Some(group)
                                            && c.optional
                                            && c.replaces.is_none()
                                            && !c.user_supplied
                                    })
                                    .collect();
                                let rows = (members.len() + 1).div_ceil(3);
                                b.push(
                                    -999,
                                    36.,
                                    y,
                                    632.,
                                    36. + rows as f32 * 60.,
                                    9,
                                    text(ui, T::AddonChoice),
                                    Default::default(),
                                    false,
                                    false,
                                );
                                let none = !members
                                    .iter()
                                    .any(|(_, c)| item.profile.components.contains(&c.id));
                                b.choice(
                                    300 + index as i32,
                                    44.,
                                    y + 36.,
                                    96.,
                                    text(ui, T::None),
                                    none,
                                );
                                for (slot, (member_index, member)) in members.iter().enumerate() {
                                    let cell = slot + 1;
                                    let row = cell / 3;
                                    let col = cell % 3;
                                    let (x, width) = match col {
                                        0 => (44., 96.),
                                        1 => (148., 248.),
                                        _ => (404., 256.),
                                    };
                                    b.choice(
                                        100 + *member_index as i32,
                                        x,
                                        y + 36. + row as f32 * 60.,
                                        width,
                                        member.label.clone().into(),
                                        item.profile.components.contains(&member.id),
                                    );
                                }
                                y += 36. + rows as f32 * 60. + 12.;
                                continue;
                            }
                            let imported = c.user_supplied
                                && item.manifest.as_ref().is_some_and(|manifest| {
                                    manifest.find_imported(&c.id).is_some()
                                });
                            // Spec 094 R4: a required player-supplied main stays
                            // required (no off switch) but its row must be
                            // focusable so the controller can import or replace
                            // the file; every other required row stays locked.
                            let actionable = c.optional || c.user_supplied;
                            let detail = if c.user_supplied {
                                if !imported {
                                    text(ui, T::NotProvided)
                                } else if !c.optional {
                                    // A required imported main always reports its
                                    // imported state; it has no off switch.
                                    text(ui, T::On)
                                } else {
                                    // An optional import follows the profile: a
                                    // disabled file shows Off, not unconditional On.
                                    text(
                                        ui,
                                        if item.profile.components.contains(&c.id) {
                                            T::On
                                        } else {
                                            T::Off
                                        },
                                    )
                                }
                            } else if !c.optional {
                                text(ui, T::Locked)
                            } else {
                                text(
                                    ui,
                                    if item.profile.components.contains(&c.id) {
                                        T::On
                                    } else {
                                        T::Off
                                    },
                                )
                            };
                            b.row(
                                100 + index as i32,
                                36.,
                                y,
                                632.,
                                c.label.clone().into(),
                                detail,
                                actionable,
                            );
                            b.nodes.last_mut().unwrap().fitted_label = true;
                            if !actionable {
                                // Spec 060 R3: focus skips it, so it reads as disabled.
                                b.nodes.last_mut().unwrap().kind = 16;
                            }
                            y += b.nodes.last().unwrap().h + 16.;
                        }
                        b.row(
                            42,
                            36.,
                            y,
                            632.,
                            text(ui, T::Advanced),
                            text(ui, T::AdvancedHelp),
                            true,
                        );
                        b.nodes.last_mut().unwrap().h = 64.;
                        y += b.nodes.last().unwrap().h + 16.;
                        b.button(
                            40,
                            36.,
                            y,
                            304.,
                            value(ui, T::Skill, item.profile.skill),
                            false,
                        );
                        b.button(
                            41,
                            356.,
                            y,
                            312.,
                            text(
                                ui,
                                if item.profile.pistol_start {
                                    T::PistolOn
                                } else {
                                    T::PistolOff
                                },
                            ),
                            false,
                        );
                        y += 80.;
                        b.row(
                            43,
                            36.,
                            y,
                            632.,
                            text(ui, T::LoadOrder),
                            text(ui, T::OrderHelp),
                            true,
                        );
                        let last_row = b.nodes.last().unwrap();
                        b.nodes[options_panel].h = (last_row.y + last_row.h + 24. - 12.).max(478.);
                        b.label(740., 36., 448., 40., text(ui, T::WillLaunch), 26.);
                        b.label(
                            740.,
                            82.,
                            448.,
                            36.,
                            if self
                                .snapshot
                                .as_ref()
                                .is_some_and(|s| base_ready(s, &item.recipe))
                            {
                                value(ui, T::FoundGame, game(ui, item.recipe.base_game.game))
                            } else {
                                value(ui, T::MissingGame, game(ui, item.recipe.base_game.game))
                            },
                            22.,
                        );
                        let usable = self.snapshot.as_ref().is_some_and(|s| s.config_usable);
                        if let Ok(summary) = &item.launch {
                            b.label(
                                740.,
                                130.,
                                448.,
                                28.,
                                if summary.content.len() == 1 {
                                    text(ui, T::ContentSingle)
                                } else {
                                    value(ui, T::ContentCount, summary.content.len())
                                },
                                22.,
                            );
                            for (i, entry) in summary.content.iter().enumerate() {
                                b.label(
                                    740.,
                                    170. + i as f32 * 42.,
                                    448.,
                                    38.,
                                    format!(
                                        "{}  {}",
                                        entry.position,
                                        entry
                                            .path
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                    )
                                    .into(),
                                    20.,
                                );
                            }
                            b.label(
                                740.,
                                186. + summary.content.len() as f32 * 38.,
                                448.,
                                56.,
                                value(ui, T::Complevel, summary.complevel.label()),
                                20.,
                            );
                            b.label(
                                740.,
                                250. + summary.content.len() as f32 * 38.,
                                448.,
                                30.,
                                value(ui, T::EngineVersion, &summary.engine_version),
                                20.,
                            );
                            b.label(
                                740.,
                                288. + summary.content.len() as f32 * 38.,
                                448.,
                                32.,
                                format!(
                                    "{} · {}",
                                    value(ui, T::Skill, summary.skill),
                                    text(
                                        ui,
                                        if summary.pistol_start {
                                            T::PistolOn
                                        } else {
                                            T::PistolOff
                                        }
                                    )
                                )
                                .into(),
                                20.,
                            );
                            b.button(
                                50,
                                740.,
                                402.,
                                448.,
                                text(ui, T::Play),
                                usable && !self.busy,
                            );
                            if !usable {
                                b.nodes.last_mut().unwrap().actionable = false;
                                b.button(1, 740., 402., 448., text(ui, T::Reset), false);
                                if self.focus == 50 {
                                    self.focus = 1;
                                }
                                ui.set_status(text(ui, T::ErrorConfig));
                            }
                        } else if let Err(blocker) = &item.launch {
                            let (reason, action) = blocker_copy(blocker);
                            b.label(740., 138., 448., 118., text(ui, reason), 22.);
                            let engine_missing =
                                self.snapshot.as_ref().is_some_and(|s| s.engine.is_none());
                            if matches!(
                                blocker,
                                Blocker::NotInstalled | Blocker::IntegrityFailed { .. }
                            ) {
                                b.button(2, 740., 282., 448., text(ui, T::InstallMore), false);
                            } else {
                                b.button(
                                    1,
                                    740.,
                                    282.,
                                    448.,
                                    text(
                                        ui,
                                        if engine_missing {
                                            T::InstallEngine
                                        } else {
                                            action
                                        },
                                    ),
                                    false,
                                );
                            }
                            b.button(50, 740., 402., 448., text(ui, T::Unavailable), false);
                            b.nodes.last_mut().unwrap().actionable = false;
                            b.nodes.last_mut().unwrap().tone = 1;
                            if self.focus == 50 {
                                self.focus = if matches!(
                                    blocker,
                                    Blocker::NotInstalled | Blocker::IntegrityFailed { .. }
                                ) {
                                    2
                                } else {
                                    1
                                };
                            }
                        }
                    }
                }
                Screen::Progress => {
                    b.panel(12., 12., 1200., 478.);
                    let subject = match self.pending.as_ref().or(self.retry.as_ref()) {
                        Some(Command::InstallEngine(version)) => {
                            value(ui, T::EngineVersion, version)
                        }
                        Some(Command::Install(id) | Command::UpdateMapset(id)) => self
                            .snapshot
                            .as_ref()
                            .and_then(|s| s.items.iter().find(|i| i.recipe.id == *id))
                            .map_or_else(
                                || text(ui, T::InstallTitle),
                                |i| display_title(&i.recipe).into(),
                            ),
                        _ => text(ui, T::InstallTitle),
                    };
                    ui.global::<crate::WiringStrings>()
                        .set_value(text(ui, T::InstallTitle));
                    ui.global::<crate::WiringStrings>()
                        .set_other(subject.clone());
                    ui.set_heading(text(ui, T::ContextTitle));
                    b.title(36., 36., 1100., 68., subject, 36.);
                    let download = download_row(self.progress);
                    let stage = if download.resuming {
                        // Spec 040 R6: the range request is in flight.
                        T::Resuming
                    } else {
                        self.progress.map_or(T::Working, |p| match p.stage {
                            InstallStage::Download => T::Download,
                            InstallStage::Verify => T::Verify,
                            InstallStage::Extract => T::Extract,
                            InstallStage::Commit => T::Commit,
                        })
                    };
                    let stage_help = match self.progress.map(|p| p.stage) {
                        Some(InstallStage::Download) => T::DownloadHelp,
                        Some(InstallStage::Verify) => T::VerifyHelp,
                        Some(InstallStage::Extract) => T::ExtractHelp,
                        Some(InstallStage::Commit) => T::CommitHelp,
                        None => T::Working,
                    };
                    ui.set_status(if self.problem.is_some() {
                        Default::default()
                    } else {
                        text(
                            ui,
                            if self.cancel.is_cancelled() {
                                T::CancelPending
                            } else {
                                stage_help
                            },
                        )
                    });
                    b.label(
                        36.,
                        112.,
                        1100.,
                        36.,
                        text(
                            ui,
                            if matches!(
                                self.pending.as_ref().or(self.retry.as_ref()),
                                Some(Command::InstallEngine(_))
                            ) {
                                T::EngineStageSequence
                            } else {
                                T::StageSequence
                            },
                        ),
                        22.,
                    );
                    b.label(
                        36.,
                        160.,
                        1100.,
                        60.,
                        text(
                            ui,
                            if self.problem.is_some() {
                                T::InstallStopped
                            } else if self.cancel.is_cancelled() {
                                T::CancelPending
                            } else {
                                stage
                            },
                        ),
                        30.,
                    );
                    b.push(
                        -999,
                        36.,
                        206.,
                        1152.,
                        24.,
                        6,
                        SharedString::default(),
                        SharedString::default(),
                        false,
                        false,
                    );
                    b.nodes.last_mut().unwrap().art = download.fraction.unwrap_or(-1);
                    if let Some(p) = self.progress.and_then(|p| p.transfer) {
                        let label = if let Some(total) = p.total {
                            ui.global::<crate::WiringStrings>()
                                .set_value(human_bytes(p.downloaded).into());
                            ui.global::<crate::WiringStrings>()
                                .set_other(human_bytes(total).into());
                            text(ui, T::Bytes)
                        } else {
                            value(ui, T::BytesUnknown, human_bytes(p.downloaded))
                        };
                        b.label(36., 252., 1100., 42., label, 22.);
                    }
                    if self.busy {
                        b.button(1, 36., 320., 400., text(ui, T::Cancel), false);
                        if self
                            .progress
                            .is_some_and(|p| p.stage == InstallStage::Commit)
                        {
                            b.nodes.last_mut().unwrap().actionable = false;
                        }
                    } else if self.problem.is_none() {
                        b.label(36., 320., 1100., 60., text(ui, T::NoTasks), 26.);
                    }
                }
                Screen::Review => {
                    b.panel(12., 12., 1200., 478.);
                    if let Some(s) = &self.snapshot {
                        if let Some(entry) = s.iwads.iter().find(|e| !e.candidates.is_empty()) {
                            b.label(36., 36., 1152., 54., text(ui, T::Ambiguous), 30.);
                            b.label(
                                36.,
                                88.,
                                1152.,
                                60.,
                                entry.path.display().to_string().into(),
                                22.,
                            );
                            for (i, id) in entry.candidates.iter().enumerate() {
                                b.button(
                                    10 + i as i32,
                                    36.,
                                    172. + i as f32 * 80.,
                                    760.,
                                    id.clone().into(),
                                    false,
                                );
                            }
                        } else {
                            b.label(
                                36.,
                                36.,
                                1152.,
                                54.,
                                text(
                                    ui,
                                    if s.iwads.is_empty() {
                                        T::NoGames
                                    } else {
                                        T::Recognized
                                    },
                                ),
                                30.,
                            );
                            for (i, e) in s.iwads.iter().enumerate() {
                                // R3: name the source ("GOG via Heroic",
                                // "Steam") next to the identified file.
                                b.row(
                                    -1,
                                    36.,
                                    100. + i as f32 * 112.,
                                    1152.,
                                    e.source.label().into(),
                                    format!(
                                        "{} · {}",
                                        e.variant.clone().unwrap_or_else(|| "unresolved".into()),
                                        e.path.display()
                                    )
                                    .into(),
                                    false,
                                );
                            }
                        }
                        // Spec 059 R4: an empty or short list may only be the
                        // sandbox not seeing the player's drive.
                        let notes_y = if s.iwads.is_empty() {
                            100.
                        } else {
                            100. + s.iwads.len() as f32 * 112.
                        };
                        for (i, path) in s.hidden_paths.iter().take(3).enumerate() {
                            b.label(
                                36.,
                                notes_y + i as f32 * 48.,
                                1152.,
                                40.,
                                value(ui, T::NotVisible, path.display()),
                                20.,
                            );
                        }
                        b.button(1, 836., 390., 352., text(ui, T::UseGame), false);
                    }
                }
                Screen::Browser => {
                    if let Some(d) = &self.directory {
                        ui.set_status(elide_path_start(&d.path.display().to_string(), 88).into());
                        b.panel(12., 12., 360., 478.);
                        b.label(36., 36., 312., 38., text(ui, T::Shortcuts), 26.);
                        for (i, p) in d.shortcuts.iter().enumerate() {
                            b.row(
                                10 + i as i32,
                                36.,
                                84. + i as f32 * 92.,
                                312.,
                                p.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned()
                                    .into(),
                                SharedString::default(),
                                true,
                            );
                        }
                        b.button(
                            1,
                            404.,
                            12.,
                            496.,
                            text(ui, T::UseFolder),
                            self.import_component.is_none(),
                        );
                        b.button(2, 924., 12., 288., text(ui, T::Parent), false);
                        for (i, (p, _)) in d.entries.iter().enumerate() {
                            b.row(
                                100 + i as i32,
                                404.,
                                96. + i as f32 * 92.,
                                808.,
                                p.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned()
                                    .into(),
                                SharedString::default(),
                                true,
                            );
                            if services::hidden_path(p) {
                                b.nodes.last_mut().unwrap().tone = 1;
                            }
                        }
                    }
                }
                Screen::Settings => {
                    b.panel(12., 12., 312., 424.);
                    // Spec 064 R5: About keeps author, repository and asset
                    // licences, not the long screenshot-credit list.
                    // Spec 059: the engine version, the sandbox line and, under
                    // Flatpak, the update-source note push later lines down.
                    let about_extra: f32 = 88. + if self.sandbox.is_flatpak() { 64. } else { 0. };
                    let hidden = self.snapshot.as_ref().map_or(0, |s| s.hidden_paths.len());
                    let height: f32 = match self.settings_section {
                        0 => 700.,
                        1 => 520.,
                        2 => self.snapshot.as_ref().map_or(316., |s| {
                            (316. + (s.folders.len() + s.automatic_folders.len()) as f32 * 112.)
                                .max(316.)
                                + if hidden > 0 {
                                    hidden as f32 * 64. + 20.
                                } else {
                                    0.
                                }
                        }),
                        3 => 280.,
                        4 if self.snapshot.is_none() => {
                            72. + self
                                .diagnostics_text()
                                .lines()
                                .map(|line| line.chars().count().div_ceil(58).max(1))
                                .sum::<usize>() as f32
                                * 30.
                        }
                        // Tall enough for the spec 017 R6 widescreen status-bar
                        // credit and the author/repository lines (spec 064 R5).
                        _ => (384. + about_extra + 80.).max(478.0_f32),
                    };
                    b.panel(348., 12., 864., height);
                    for (i, label) in [T::Engine, T::Audio, T::Folders, T::Config, T::Diagnostics]
                        .iter()
                        .enumerate()
                    {
                        b.button(
                            1 + i as i32,
                            28.,
                            36. + i as f32 * 80.,
                            280.,
                            text(ui, *label),
                            false,
                        );
                    }
                    if self.snapshot.is_none() {
                        if self.settings_section == 4 {
                            let diagnostics = self.diagnostics_text();
                            b.label(372., 36., 816., height - 48., diagnostics.into(), 22.);
                        } else {
                            b.label(372., 36., 816., 120., text(ui, T::StateNeeded), 22.);
                            b.button(801, 372., 180., 816., text(ui, T::Diagnostics), false);
                        }
                    }
                    if let Some(s) = &self.snapshot {
                        match self.settings_section {
                            0 => {
                                b.label(
                                    372.,
                                    36.,
                                    816.,
                                    44.,
                                    s.engine.as_ref().map_or_else(
                                        || text(ui, T::NoEngine),
                                        |e| value(ui, T::EngineVersion, &e.version),
                                    ),
                                    30.,
                                );
                                b.label(372., 96., 816., 36., text(ui, T::Trust), 20.);
                                if let Some(a) = &s.attribution {
                                    b.label(
                                        372.,
                                        142.,
                                        816.,
                                        64.,
                                        format!(
                                            "{}\n{}",
                                            a.license,
                                            a.source_url.as_deref().unwrap_or_default()
                                        )
                                        .into(),
                                        20.,
                                    );
                                }
                                b.button(70, 372., 240., 380., text(ui, T::CheckUpdates), false);
                                b.row(
                                    71,
                                    372.,
                                    324.,
                                    816.,
                                    text(ui, T::Rollback),
                                    s.previous
                                        .clone()
                                        .map_or_else(|| text(ui, T::NoRollback), Into::into),
                                    s.previous.is_some(),
                                );
                                if self
                                    .updates
                                    .iter()
                                    .any(|v| s.engine.as_ref().is_none_or(|e| e.version != *v))
                                {
                                    b.button(
                                        72,
                                        372.,
                                        420.,
                                        816.,
                                        text(ui, T::ReviewedUpdate),
                                        false,
                                    );
                                } else {
                                    b.label(
                                        372.,
                                        420.,
                                        816.,
                                        48.,
                                        text(
                                            ui,
                                            if self.updates_checked {
                                                T::NoUpdates
                                            } else {
                                                T::PreCheck
                                            },
                                        ),
                                        20.,
                                    );
                                }
                                // Spec 019 R5: the catalog in use, the manual
                                // check and the automatic toggle.
                                b.label(
                                    372.,
                                    486.,
                                    816.,
                                    40.,
                                    catalog_summary(ui, &s.catalog),
                                    22.,
                                );
                                b.button(
                                    73,
                                    372.,
                                    534.,
                                    380.,
                                    text(ui, T::CheckCatalogUpdates),
                                    false,
                                );
                                b.row(
                                    74,
                                    372.,
                                    600.,
                                    816.,
                                    text(ui, T::CatalogAutoUpdates),
                                    text(ui, if s.catalog.auto_update { T::On } else { T::Off }),
                                    true,
                                );
                            }
                            1 => {
                                // Spec 067 R1/R2/R4/R6: the two supported
                                // choices, the optional reviewed install and the
                                // honest custom-engine state. Default is always
                                // available; Arachno only after a verified install.
                                let audio = &s.audio;
                                let selected_default = match audio.choice {
                                    Some(Choice::Default) => true,
                                    Some(Choice::Arachno) => false,
                                    None => audio.current == Current::Default,
                                };
                                let selected_arachno = audio.choice == Some(Choice::Arachno);
                                b.label(372., 36., 816., 44., text(ui, T::AudioTitle), 30.);
                                b.label(372., 84., 816., 28., text(ui, T::AudioHelp), 20.);
                                b.choice_enabled(
                                    70,
                                    372.,
                                    116.,
                                    816.,
                                    text(ui, T::AudioDefault),
                                    selected_default,
                                    true,
                                );
                                b.label(372., 172., 816., 28., text(ui, T::AudioDefaultHelp), 20.);
                                let state = if audio.installed {
                                    text(ui, T::Installed)
                                } else {
                                    text(ui, T::AudioNotInstalled)
                                };
                                b.choice_enabled(
                                    71,
                                    372.,
                                    206.,
                                    816.,
                                    text(ui, T::AudioArachno),
                                    selected_arachno,
                                    audio.installed,
                                );
                                b.label(
                                    372.,
                                    262.,
                                    816.,
                                    28.,
                                    format!(
                                        "{} · {} · {}",
                                        ddl_core::soundfonts::AUTHOR,
                                        human_bytes(ddl_core::soundfonts::DOWNLOAD_BYTES),
                                        state
                                    )
                                    .into(),
                                    20.,
                                );
                                if let Some(warning) = &audio.warning {
                                    b.label(372., 296., 816., 48., warning.as_str().into(), 20.);
                                } else if audio.choice.is_none() && audio.current == Current::Custom
                                {
                                    b.label(
                                        372.,
                                        296.,
                                        816.,
                                        48.,
                                        format!(
                                            "{} — {}",
                                            text(ui, T::AudioCustom),
                                            text(ui, T::AudioCustomHelp)
                                        )
                                        .into(),
                                        20.,
                                    );
                                }
                                if !audio.installed || audio.warning.is_some() {
                                    b.button(
                                        72,
                                        372.,
                                        352.,
                                        380.,
                                        text(
                                            ui,
                                            if audio.warning.is_some() {
                                                T::AudioReinstall
                                            } else {
                                                T::AudioDownload
                                            },
                                        ),
                                        false,
                                    );
                                }
                                b.row(
                                    73,
                                    372.,
                                    416.,
                                    816.,
                                    text(ui, T::AudioTerms),
                                    "arachnosoft.com".into(),
                                    true,
                                );
                            }
                            2 => {
                                // R4 opt-in, default off; the existing Ember
                                // row component shows the current state.
                                b.row(
                                    72,
                                    372.,
                                    36.,
                                    816.,
                                    text(ui, T::SearchStandardFolders),
                                    text(
                                        ui,
                                        if s.search_standard_folders {
                                            T::On
                                        } else {
                                            T::Off
                                        },
                                    ),
                                    true,
                                );
                                b.button(70, 372., 128., 380., text(ui, T::AddFolder), false);
                                b.button(71, 776., 128., 412., text(ui, T::Rescan), false);
                                for (i, folder) in s.automatic_folders.iter().enumerate() {
                                    b.row(
                                        1000 + i as i32,
                                        372.,
                                        220. + i as f32 * 112.,
                                        816.,
                                        format!(
                                            "{} · {}",
                                            folder.source.label(),
                                            if folder.games.is_empty() {
                                                "No games identified".into()
                                            } else {
                                                folder.games.join(", ")
                                            }
                                        )
                                        .into(),
                                        folder.path.display().to_string().into(),
                                        true,
                                    );
                                }
                                if s.folders.is_empty() {
                                    b.label(
                                        372.,
                                        220. + s.automatic_folders.len() as f32 * 112.,
                                        816.,
                                        84.,
                                        text(ui, T::NoFolders),
                                        22.,
                                    );
                                }
                                for (i, p) in s.folders.iter().enumerate() {
                                    let y = 220. + (i + s.automatic_folders.len()) as f32 * 112.;
                                    let games: Vec<_> = s
                                        .base_games
                                        .iter()
                                        .filter(|b| {
                                            s.iwads.iter().any(|e| {
                                                e.game == Some(b.base.game) && e.path.starts_with(p)
                                            })
                                        })
                                        .map(|b| b.base.title())
                                        .collect();
                                    b.row(
                                        0,
                                        372.,
                                        y,
                                        636.,
                                        format!(
                                            "User folder · {}",
                                            if games.is_empty() {
                                                "No games identified".into()
                                            } else {
                                                games.join(", ")
                                            }
                                        )
                                        .into(),
                                        p.display().to_string().into(),
                                        false,
                                    );
                                    b.button(
                                        100 + i as i32,
                                        968.,
                                        y,
                                        220.,
                                        text(ui, T::Remove),
                                        false,
                                    );
                                    b.nodes.last_mut().unwrap().h = 88.;
                                }
                                // Spec 059 R4: say why a folder the host has
                                // shows nothing, instead of an empty list.
                                let rows = (s.folders.len() + s.automatic_folders.len()) as f32;
                                let notes_y = 316. + rows * 112.
                                    - if s.folders.is_empty() { 0. } else { 96. };
                                for (i, path) in s.hidden_paths.iter().enumerate() {
                                    b.label(
                                        372.,
                                        notes_y + i as f32 * 64.,
                                        816.,
                                        56.,
                                        value(ui, T::NotVisible, path.display()),
                                        20.,
                                    );
                                }
                            }
                            3 => {
                                b.label(
                                    372.,
                                    36.,
                                    816.,
                                    100.,
                                    text(
                                        ui,
                                        if s.config_notice {
                                            T::ConfigNotice
                                        } else {
                                            T::ConfigReady
                                        },
                                    ),
                                    22.,
                                );
                                b.label(
                                    372.,
                                    136.,
                                    816.,
                                    50.,
                                    value(ui, T::ConfigSchema, &s.config_version),
                                    22.,
                                );
                                b.button(70, 372., 200., 816., text(ui, T::Reset), false);
                            }
                            _ => {
                                b.label(372., 36., 816., 44., text(ui, T::Logs), 30.);
                                b.label(
                                    372.,
                                    96.,
                                    816.,
                                    64.,
                                    s.logs.display().to_string().into(),
                                    22.,
                                );
                                b.logo(372., 170., 560., 128., false);
                                b.label(960., 202., 228., 44., text(ui, T::AppVersion), 26.);
                                b.label(960., 264., 228., 44., crate::cli::VERSION.into(), 22.);
                                // Spec 064 R5: attribution and repository link stay
                                // above the longer licence text, so they are visible
                                // without scrolling a tall About panel.
                                b.label(372., 306., 816., 32., text(ui, T::MadeBy), 20.);
                                b.label(372., 342., 816., 32., text(ui, T::Repository), 20.);
                                // Spec 059: the engine version is its own line,
                                // then the sandbox state and, under Flatpak,
                                // where DDL's updates come from (R1, R7).
                                b.label(
                                    372.,
                                    378.,
                                    816.,
                                    40.,
                                    s.engine.as_ref().map_or_else(
                                        || text(ui, T::NoEngine),
                                        |e| value(ui, T::EngineVersion, &e.version),
                                    ),
                                    22.,
                                );
                                b.label(
                                    372.,
                                    418.,
                                    816.,
                                    40.,
                                    sandbox_text(ui, &self.sandbox),
                                    22.,
                                );
                                let mut y = 462.;
                                if self.sandbox.is_flatpak() {
                                    b.label(372., y, 816., 56., text(ui, T::StoreUpdates), 20.);
                                    y += 64.;
                                }
                                // Spec 017, R6: credit the widescreen status bars.
                                b.label(372., y, 816., 56., text(ui, T::StatusBarCredit), 20.);
                            }
                        }
                    }
                }
            }
        }
        if let Some(problem) = &self.problem {
            b.panel(12., 510., 1200., 222.);
            b.label(36., 530., 1152., 88., problem_text(ui, problem), 22.);
            b.button(800, 36., 638., 552., text(ui, T::Retry), false);
            b.button(801, 612., 638., 576., text(ui, T::Diagnostics), false);
        }
        self.local_nodes(ui, &mut b);
        self.nodes = b.nodes;
        if !self.playing
            && self.focus >= 0
            && self.sheet.is_none()
            && self.local.dialog.is_none()
            && !self
                .nodes
                .iter()
                .any(|n| n.id == self.focus && n.actionable)
        {
            self.focus = self.nodes.iter().find(|n| n.actionable).map_or(0, |n| n.id);
        }
        let setup = self.screen == Screen::Setup && self.local.view.is_none();
        let discover = self.screen == Screen::Discover;
        let list = matches!(self.screen, Screen::Discover | Screen::Library)
            && self.local.view.is_none()
            && self.view_format == services::ViewFormat::List;
        let viewport = (ui.window().size().height as f32
            - if discover {
                298.
            } else if list {
                278.
            } else {
                294.
            })
        .max(64.);
        let focus_inset = if discover { 24. } else { 12. };
        let plane: Vec<&ViewNode> = self.nodes.iter().filter(|n| !setup || n.x < 700.).collect();
        let focused = plane.iter().find(|n| n.id == self.focus).copied();
        // Geometry is part of the anchor: a favorite toggle can move the same
        // selected item. A neutral refresh with unchanged geometry preserves
        // manual scrolling, even when the selected item is off screen.
        let (focus_y, focus_h) = focused.map_or((0., 0.), |n| (n.y, n.h));
        let anchor = (self.screen, self.local.view, self.focus, focus_y, focus_h);
        let focus_changed = self.scroll_anchor != Some(anchor);
        // Update bounds before changing offset, so Flickable never clamps an
        // entering list against the previous screen's extent.
        ui.set_content_height(plane.iter().map(|n| n.y + n.h + 20.).fold(0_f32, f32::max));
        ui.set_continuous_list(list);
        if focus_changed {
            let offset = if self.local.view == Some(local_ui::View::Picker) {
                0.
            } else if list {
                focused.map_or(0., |n| {
                    continuous_offset(&plane, n, viewport, ui.get_presented_scroll_offset())
                })
            } else {
                focused.map_or(0., |n| follow_offset(&plane, n, viewport, focus_inset))
            };
            // Animate only navigation inside an existing list. Entering a tab,
            // changing layout or returning from detail reveals focus immediately.
            let smooth = list
                && self
                    .scroll_anchor
                    .is_some_and(|previous| previous.0 == anchor.0 && previous.1 == anchor.1);
            ui.invoke_follow_scroll(offset, smooth);
            ui.invoke_reset_scroll_x();
            self.scroll_anchor = Some(anchor);
        }
        ui.set_nodes(ModelRc::new(VecModel::from(self.nodes.clone())));
        ui.set_focused(self.focus);
        ui.set_active_tab(self.tab_index() as i32);
        ui.set_root_screen(self.root() && !self.playing);
        // The footer says Quit exactly where B opens the Quit dialog: a root
        // with no level left, and the recovery screen (spec 055).
        ui.set_quit_hint(
            !self.playing
                && ((self.root()
                    && self.history.is_empty()
                    && !(self.screen == Screen::Settings && self.focus >= 70))
                    || (self.screen == Screen::Recovery && self.sheet.is_none())),
        );
        ui.set_input_locked(self.playing);
        ui.set_setup_screen(setup && !self.playing);
        ui.set_install_pulse(
            self.screen == Screen::FirstRun
                && !self.playing
                && !self.busy
                && self.snapshot.as_ref().is_some_and(|s| s.engine.is_none()),
        );
        ui.set_discover_screen(self.screen == Screen::Discover);
        ui.set_a_hint(
            self.nodes
                .iter()
                .find(|n| n.id == self.focus && !n.label.is_empty())
                .map_or_else(|| text(ui, T::Open), |n| n.label.clone()),
        );
        let card_focus = matches!(self.screen, Screen::Discover | Screen::Library)
            && card_index(self.focus).is_some();
        let marked = if card_focus {
            self.snapshot
                .as_ref()
                .and_then(|s| s.items.get(card_index(self.focus).unwrap()))
        } else {
            self.item()
        }
        .is_some_and(|i| i.favorite);
        ui.set_favorite_hint(
            !self.playing && (card_focus || self.screen == Screen::Detail) && self.sheet.is_none(),
        );
        ui.set_favorite_label(text(
            ui,
            if marked {
                T::FavoriteRemove
            } else {
                T::FavoriteAdd
            },
        ));
        ui.set_view_hint(self.can_toggle_view());
        ui.set_list_view(self.view_format == services::ViewFormat::List);
        ui.set_category_hint(self.screen == Screen::Discover && !self.playing);
        ui.set_detail_hint(false);
        ui.set_import_hint(false);
        ui.set_profile_hint(
            matches!(self.sheet, Some(Sheet::Profiles))
                || (self.screen == Screen::Setup && self.focus == 60),
        );
        ui.set_profile_name_open(matches!(self.sheet, Some(Sheet::ProfileNames)));
        ui.set_sheet_open(self.sheet.is_some());
        ui.set_sheet_body(match self.sheet {
            Some(Sheet::ImportOffer) => text(ui, T::ImportOfferBody),
            Some(Sheet::UserComponent(ref id)) => {
                let optional = self.item().is_some_and(|item| {
                    item.recipe
                        .components
                        .iter()
                        .any(|component| component.id.as_str() == id.as_str() && component.optional)
                });
                if optional {
                    text(ui, T::ImportedComponentHelp)
                } else {
                    "".into()
                }
            }
            _ => "".into(),
        });
        ui.set_sheet_back_later(matches!(self.sheet, Some(Sheet::ImportOffer)));
        ui.set_sheet_height(
            if matches!(self.sheet, Some(Sheet::Profiles | Sheet::ProfileNames)) {
                if matches!(self.sheet, Some(Sheet::Profiles)) {
                    340. + self.item().map_or(0, |i| i.profiles.len().min(3)) as f32 * 62.
                } else {
                    460.
                }
            } else if matches!(self.sheet, Some(Sheet::StartFresh | Sheet::ImportOffer)) {
                380.
            } else if matches!(self.sheet, Some(Sheet::DeleteProfile(..))) {
                340.
            } else if matches!(self.sheet, Some(Sheet::Uninstall(_))) {
                420.
            } else {
                300.
            },
        );
        if matches!(self.sheet, Some(Sheet::Profiles | Sheet::ProfileNames)) {
            self.profile_sheet(ui);
        } else if matches!(self.sheet, Some(Sheet::Uninstall(_))) {
            self.uninstall_sheet(ui);
        } else if let Some(sheet) = &self.sheet {
            let (title, safe, confirm) = match sheet {
                Sheet::Quit => (T::QuitConfirm, T::Quit, T::KeepPlaying),
                Sheet::Cancel => (T::CancelConfirm, T::KeepDownloading, T::CancelInstall),
                Sheet::Reset => (T::ResetConfirm, T::Cancel, T::Reset),
                Sheet::StartFresh => (T::StartFreshConfirm, T::Cancel, T::StartFresh),
                Sheet::Remove(_) => (T::RemoveConfirm, T::Cancel, T::Remove),
                Sheet::Component(_) => (T::CompatConfirm, T::Keep, T::UseVariant),
                Sheet::UserComponent(id) => (
                    T::ImportedComponent,
                    T::Import,
                    if self.item().is_some_and(|item| {
                        item.recipe.components.iter().any(|component| {
                            component.id.as_str() == id.as_str() && !component.optional
                        })
                    }) {
                        // Required player-supplied main: import/replace only.
                        T::Done
                    } else if self
                        .item()
                        .is_some_and(|item| item.profile.components.contains(id))
                    {
                        T::Off
                    } else {
                        T::On
                    },
                ),
                Sheet::DeleteProfile(..) => (T::DeleteProfileConfirm, T::Cancel, T::DeleteProfile),
                Sheet::ImportOffer => (T::ImportOffer, T::ImportYes, T::ImportNo),
                Sheet::Profiles | Sheet::ProfileNames | Sheet::Uninstall(_) => unreachable!(),
            };
            ui.set_sheet_title(if let Sheet::DeleteProfile(_, name) = sheet {
                value(ui, title, name)
            } else {
                text(ui, title)
            });
            let mut b = Nodes::new(ui);
            let button_y = if matches!(sheet, Sheet::StartFresh | Sheet::ImportOffer) {
                268.
            } else if matches!(sheet, Sheet::DeleteProfile(..)) {
                228.
            } else {
                180.
            };
            b.button(900, 32., button_y, 304., text(ui, safe), false);
            b.button(901, 368., button_y, 304., text(ui, confirm), false);
            ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
        }
        self.local_sheet(ui);
        let bounds = |right: bool| {
            self.nodes
                .iter()
                .filter(|n| !ui.get_setup_screen() || (n.x >= 700.) == right)
                .fold((0_f32, 0_f32), |(w, h), n| {
                    (
                        (n.x + n.w + 20. - if right { 700. } else { 0. }).max(w),
                        (n.y + n.h + 20.).max(h),
                    )
                })
        };
        let (width, height) = bounds(false);
        ui.set_content_width(width);
        ui.set_content_height(height);
        let (width, height) = bounds(true);
        ui.set_setup_content_width(width);
        ui.set_setup_content_height(height);
        ui.set_setup_right_focused(self.nodes.iter().any(|n| n.id == self.focus && n.x >= 700.));
        if focus_changed {
            let summary: Vec<&ViewNode> = self.nodes.iter().filter(|n| n.x >= 700.).collect();
            let offset = summary
                .iter()
                .find(|n| n.id == self.focus)
                .map_or(0., |n| follow_offset(&summary, n, viewport, 12.));
            ui.set_setup_scroll_offset(offset);
        }
        if self.sheet.is_some() {
            let nodes: Vec<_> = ui.get_sheet_nodes().iter().collect();
            if !nodes.iter().any(|n| n.id == self.focus && n.actionable) {
                self.focus = nodes.iter().find(|n| n.actionable).map_or(900, |n| n.id);
            }
            ui.set_focused(self.focus);
            if let Some(n) = nodes.iter().find(|n| n.id == self.focus) {
                ui.set_a_hint(n.label.clone());
                if focus_changed {
                    let plane: Vec<_> = nodes.iter().collect();
                    let viewport = ui
                        .get_sheet_height()
                        .min(ui.window().size().height as f32 - 80.);
                    ui.set_sheet_scroll_offset(follow_offset(&plane, n, viewport, 16.));
                }
            }
        }
    }
}
/// Reveal the focus ring with the least possible movement (spec 102).
/// Nodes render with an 8 px inset; compact rings extend 8 px beyond the row.
/// Unlike Cards, lists never reserve a trailing viewport just to top-align focus.
fn continuous_offset(plane: &[&ViewNode], focus: &ViewNode, viewport: f32, current: f32) -> f32 {
    let extent = plane.iter().map(|n| n.y + n.h + 20.).fold(0_f32, f32::max);
    let limit = (extent - viewport).max(0.);
    let current = current.clamp(0., limit);
    let top = focus.y.max(0.);
    let bottom = focus.y + focus.h + 16.;
    let destination = if top < current || bottom - top > viewport {
        top
    } else if bottom > current + viewport {
        bottom - viewport
    } else {
        current
    };
    destination.clamp(0., limit)
}

/// Keep whole items at the top after focus navigation. Nodes have an 8 px
/// inset in the main plane; card frames extend 8 px and focus rings 16 px.
/// Prefer the preceding section heading when it fits with the focused item.
/// Manual right-stick scrolling does not call this function.
fn follow_offset(plane: &[&ViewNode], focus: &ViewNode, viewport: f32, inset: f32) -> f32 {
    let bottom = focus.y + focus.h + inset;
    if bottom <= viewport {
        return 0.;
    }
    let top = (focus.y - if focus.kind == 1 { 8. } else { 12. }).max(0.);
    let inside_card = |n: &ViewNode| {
        plane.iter().any(|c| {
            c.kind == 1
                && c.actionable
                && n.x >= c.x
                && n.y >= c.y
                && n.x + n.w <= c.x + c.w
                && n.y + n.h <= c.y + c.h
        })
    };
    plane
        .iter()
        .filter(|n| {
            !n.actionable
                && matches!(n.kind, 0 | 20)
                && n.size >= 26.
                && n.y < focus.y
                && n.x < focus.x + focus.w
                && n.x + n.w > focus.x
                && !inside_card(n)
        })
        .map(|n| (n.y + 4.).max(0.))
        .filter(|&offset| bottom - offset <= viewport)
        // Do not expose a previous card's bottom rail above the heading.
        .filter(|&offset| {
            !plane.iter().any(|n| {
                n.kind == 1
                    && n.y < offset
                    && offset < n.y + n.h + 16.
                    && (n.actionable || n.y + n.h + 16. - offset < 64.)
            })
        })
        .max_by(f32::total_cmp)
        .unwrap_or(top)
}
#[cfg(test)]
mod scroll_tests {
    use super::*;

    fn card(y: f32, h: f32) -> ViewNode {
        ViewNode {
            x: 12.,
            y,
            w: 384.,
            h,
            kind: 1,
            actionable: true,
            ..Default::default()
        }
    }

    #[test]
    fn continuous_follow_moves_only_at_edges_and_clamps_both_bounds() {
        let rows: Vec<_> = (0..30).map(|i| card(12. + 82. * i as f32, 72.)).collect();
        let plane: Vec<_> = rows.iter().collect();
        let mut offset = 0.;
        for row in &rows {
            let next = continuous_offset(&plane, row, 502., offset);
            assert!((0. ..=82.).contains(&(next - offset)));
            assert!(row.y + 4. >= next);
            assert!(row.y + row.h + 12. <= next + 502.);
            offset = next;
        }
        assert!(offset > 3. * 502., "several viewports traversed");
        for row in rows.iter().rev() {
            let next = continuous_offset(&plane, row, 502., offset);
            assert!((0. ..=82.).contains(&(offset - next)));
            offset = next;
        }
        assert_eq!(continuous_offset(&plane, &rows[0], 502., -100.), 0.);
        let limit = rows.last().unwrap().y + 72. + 20. - 502.;
        assert_eq!(
            continuous_offset(&plane, rows.last().unwrap(), 502., 10000.),
            limit
        );
        assert_eq!(
            continuous_offset(&plane, &rows[2], 502., 37.),
            37.,
            "visible row retains manual offset"
        );
    }

    #[test]
    fn focus_follow_keeps_a_section_heading_and_whole_card_together() {
        let previous = card(152., 84.);
        let heading = ViewNode {
            x: 12.,
            y: 252.,
            w: 1200.,
            h: 42.,
            kind: 20,
            size: 28.,
            ..Default::default()
        };
        let focus = card(298., 410.);
        let plane = [&previous, &heading, &focus];
        assert_eq!(follow_offset(&plane, &focus, 506., 12.), 256.);
        assert_eq!(
            follow_offset(&plane, &focus, 450., 12.),
            290.,
            "heading cannot fit"
        );
        assert_eq!(
            follow_offset(&plane, &previous, 506., 12.),
            0.,
            "already visible"
        );
    }

    #[test]
    fn focus_follow_aligns_later_and_oversized_items_without_partial_previous_rows() {
        let previous = card(298., 410.);
        let focus = card(736., 410.);
        let plane = [&previous, &focus];
        assert_eq!(follow_offset(&plane, &focus, 506., 12.), 728.);
        assert_eq!(
            follow_offset(&plane, &focus, 300., 12.),
            728.,
            "oversized item starts at its top"
        );
        let button = ViewNode {
            x: 32.,
            y: 780.,
            w: 304.,
            h: 56.,
            kind: 2,
            actionable: true,
            ..Default::default()
        };
        assert_eq!(
            follow_offset(&[&button], &button, 720., 16.),
            768.,
            "sheets and summary use whole rows too"
        );
    }
}

fn profile_count(ui: &AppWindow, count: usize) -> SharedString {
    if count == 1 {
        text(ui, T::SingleProfile)
    } else {
        value(ui, T::ProfileCount, count)
    }
}
/// Vocabulary labels of a recipe's tags in `order`, at most `limit`; ids the
/// vocabulary does not know are left out. `first` (the open category's tag)
/// leads, so a Cacowards card keeps its Cacowards chip when only two fit.
fn tag_labels(
    s: &Snapshot,
    recipe: &ddl_core::catalog::Recipe,
    limit: usize,
    first: Option<&str>,
) -> Vec<String> {
    let mut tags: Vec<_> = s
        .tags
        .ordered()
        .into_iter()
        .filter(|tag| recipe.tags.contains(&tag.id))
        .collect();
    tags.sort_by_key(|tag| Some(tag.id.as_str()) != first);
    tags.into_iter()
        .take(limit)
        .map(|tag| tag.label.clone())
        .collect()
}
/// Rough text width of a Barlow label at `size`, for packing chips.
fn chip_width(label: &str, size: f32) -> f32 {
    (label.chars().count() as f32 * size * 0.53 + 26.).ceil()
}
fn art(id: &str) -> i32 {
    match id {
        "sigil" => 0,
        "auger-zenith" => 2,
        "sigil-ii" => 1,
        "eviternity" => 3,
        _ => 4,
    }
}
fn game(ui: &AppWindow, game: ddl_core::catalog::Game) -> SharedString {
    text(
        ui,
        match game {
            ddl_core::catalog::Game::Doom => T::Doom,
            ddl_core::catalog::Game::Doom2 => T::Doom2,
            ddl_core::catalog::Game::Tnt => T::Tnt,
            ddl_core::catalog::Game::Plutonia => T::Plutonia,
        },
    )
}
fn item_availability(ui: &AppWindow, snapshot: &Snapshot, item: &Item) -> SharedString {
    text(
        ui,
        match item.recipe_status {
            ddl_core::library::RecipeStatus::Removed => T::RecipeRemoved,
            ddl_core::library::RecipeStatus::Update(_) if !item.update_dismissed => T::RecipeUpdate,
            _ if base_ready(snapshot, &item.recipe) => T::BaseReady,
            _ => T::BaseMissingShort,
        },
    )
}
fn base_ready(s: &Snapshot, r: &ddl_core::catalog::Recipe) -> bool {
    s.iwads.iter().any(|e| {
        e.game
            .is_some_and(|g| g.as_str() == ddl_core::resolver::game_label(r.base_game.game))
            && e.variant.as_ref().is_some_and(|v| {
                r.base_game.variants.is_empty() || r.base_game.variants.contains(v)
            })
    })
}
fn blocker_copy(b: &Blocker) -> (T, T) {
    match b {
        Blocker::MissingIwad { .. } => (T::BaseMissing, T::AddFolder),
        Blocker::EngineMissing => (T::EngineMissing, T::InstallEngine),
        Blocker::NotInstalled => (T::MapsetMissing, T::Install),
        Blocker::ComponentNotProvided { .. } => (T::ComponentMissing, T::Import),
        Blocker::IncompatibleChoice { .. } => (T::ChoiceInvalid, T::Setup),
        Blocker::IntegrityFailed { .. } => (T::IntegrityFailed, T::Install),
        // Spec 026: a local user file is missing or changed; the UI's phase-B
        // file list is where the player fixes it.
        Blocker::UserFileMissing { .. } | Blocker::UserFileChanged { .. } => {
            (T::IntegrityFailed, T::Setup)
        }
    }
}
fn problem_copy(p: &Problem) -> T {
    match p {
        Problem::Local(_) | Problem::LocalFile(..) => T::ErrorLaunch,
        Problem::Storage => T::ErrorStorage,
        Problem::State | Problem::Unreadable(_) => T::ErrorState,
        Problem::Network => T::ErrorNetwork,
        Problem::Stalled => T::ErrorStalled,
        Problem::Update => T::UpdateError,
        Problem::Catalog => T::ErrorCatalog,
        Problem::CatalogNeedsNewer { .. } => T::ErrorCatalogNeedsNewerUnknown,
        Problem::Checksum => T::ErrorChecksum,
        Problem::UnsafeArchive => T::ErrorArchive,
        Problem::Cancelled => T::ErrorCancelled,
        Problem::Engine => T::ErrorEngine,
        Problem::Config => T::ErrorConfig,
        Problem::Audio => T::ErrorAudio,
        Problem::Launch => T::ErrorLaunch,
        Problem::Busy => T::ErrorBusy,
        Problem::EngineRuntime(_) => T::ErrorEngineRuntime,
        Problem::NotVisible(_) => T::NotVisible,
    }
}

/// The problem panel's label: a static key, or the engine runtime's own
/// actionable message (spec 014 follow-up).
fn problem_text(ui: &AppWindow, problem: &Problem) -> slint::SharedString {
    if let Problem::NotVisible(path) = problem {
        return value(ui, T::NotVisible, path.display());
    }
    // Spec 098: name the version the catalog needs, so a user on an older DDL
    // updates the app instead of retrying a check that can never succeed.
    if let Problem::CatalogNeedsNewer { min: Some(min) } = problem {
        return value(ui, T::ErrorCatalogNeedsNewer, min);
    }
    match problem_detail(problem) {
        Some(detail) => value(ui, T::ErrorEngineRuntime, detail),
        None => text(ui, problem_copy(problem)),
    }
}

/// The runtime detail a problem carries, when it has one.
fn problem_detail(problem: &Problem) -> Option<String> {
    match problem {
        Problem::Local(message) => Some(message.clone()),
        Problem::LocalFile(path, _) => Some(path.display().to_string()),
        Problem::EngineRuntime(runtime) => Some(runtime.to_string()),
        _ => None,
    }
}
struct Nodes {
    nodes: Vec<ViewNode>,
}
impl Nodes {
    fn new(_ui: &AppWindow) -> Self {
        Self { nodes: vec![] }
    }
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        id: i32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        kind: i32,
        label: SharedString,
        detail: SharedString,
        actionable: bool,
        primary: bool,
    ) {
        self.nodes.push(ViewNode {
            id,
            x,
            y,
            w,
            h,
            kind,
            label,
            detail,
            actionable,
            primary,
            size: 22.,
            tone: 0,
            art: 0,
            screenshot: Default::default(),
            fitted_label: false,
        });
    }
    fn panel(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.push(
            -999,
            x,
            y,
            w,
            h,
            1,
            Default::default(),
            Default::default(),
            false,
            false,
        );
    }
    fn logo(&mut self, x: f32, y: f32, w: f32, h: f32, stacked: bool) {
        self.art(x, y, w, h, 0);
        self.nodes.last_mut().unwrap().kind = if stacked { 11 } else { 10 };
    }
    fn art(&mut self, x: f32, y: f32, w: f32, h: f32, art: i32) {
        self.push(
            -999,
            x,
            y,
            w,
            h,
            4,
            Default::default(),
            Default::default(),
            false,
            false,
        );
        self.nodes.last_mut().unwrap().art = art;
    }
    fn recipe_art(&mut self, x: f32, y: f32, w: f32, h: f32, recipe: &ddl_core::catalog::Recipe) {
        self.art(x, y, w, h, art(&recipe.id));
        self.nodes.last_mut().unwrap().screenshot = crate::screenshots::presentation(recipe);
    }
    fn label(&mut self, x: f32, y: f32, w: f32, h: f32, label: SharedString, size: f32) {
        self.push(-999, x, y, w, h, 0, label, Default::default(), false, false);
        self.nodes.last_mut().unwrap().size = size;
    }
    fn title(&mut self, x: f32, y: f32, w: f32, h: f32, label: SharedString, size: f32) {
        self.label(x, y, w, h, label, size);
        self.nodes.last_mut().unwrap().kind = 20;
    }
    fn author(&mut self, x: f32, y: f32, w: f32, h: f32, label: SharedString, size: f32) {
        self.label(x, y, w, h, label, size);
        self.nodes.last_mut().unwrap().kind = 21;
    }
    fn button(&mut self, id: i32, x: f32, y: f32, w: f32, label: SharedString, primary: bool) {
        self.push(
            id,
            x,
            y,
            w,
            56.,
            2,
            label,
            Default::default(),
            true,
            primary,
        );
    }
    fn check(&mut self, x: f32, y: f32, label: SharedString, tone: i32) {
        self.push(
            -999,
            x,
            y,
            336.,
            64.,
            7,
            label,
            Default::default(),
            false,
            false,
        );
        self.nodes.last_mut().unwrap().tone = tone;
        self.nodes.last_mut().unwrap().size = 22.;
    }
    /// A controller-toggleable checkbox (spec 022, R2): the Ember row shape
    /// with an amber tick when `checked`.
    fn checkbox(&mut self, id: i32, x: f32, y: f32, w: f32, label: SharedString, checked: bool) {
        self.push(id, x, y, w, 56., 15, label, Default::default(), true, false);
        self.nodes.last_mut().unwrap().size = 22.;
        self.nodes.last_mut().unwrap().art = i32::from(checked);
    }
    fn choice(&mut self, id: i32, x: f32, y: f32, w: f32, label: SharedString, selected: bool) {
        self.choice_enabled(id, x, y, w, label, selected, true);
    }
    /// A choice row that can be present but not yet selectable (spec 067 R2:
    /// Arachno before its reviewed install exists).
    #[allow(clippy::too_many_arguments)]
    fn choice_enabled(
        &mut self,
        id: i32,
        x: f32,
        y: f32,
        w: f32,
        label: SharedString,
        selected: bool,
        actionable: bool,
    ) {
        self.push(
            id,
            x,
            y,
            w,
            52.,
            8,
            label,
            Default::default(),
            actionable,
            selected,
        );
        self.nodes.last_mut().unwrap().size = 20.;
    }
    #[allow(clippy::too_many_arguments)]
    fn row(
        &mut self,
        id: i32,
        x: f32,
        y: f32,
        w: f32,
        label: SharedString,
        detail: SharedString,
        actionable: bool,
    ) {
        self.push(
            id,
            x,
            y,
            w,
            if detail.len() > 50 { 96. } else { 80. },
            3,
            label,
            detail,
            actionable,
            false,
        );
    }
    /// A gold star on a dark disc marking a favorite.
    fn favorite(&mut self, x: f32, y: f32) {
        self.push(
            -999,
            x,
            y,
            36.,
            36.,
            12,
            Default::default(),
            Default::default(),
            false,
            false,
        );
    }
    /// Tag chips from `x`, wrapping inside `width`; returns the rows used.
    /// Chips that do not fit in the second row are dropped.
    fn chips(&mut self, labels: &[String], x: f32, y: f32, width: f32) -> usize {
        let (mut cx, mut row) = (0., 0);
        for label in labels {
            let w = chip_width(label, 18.);
            if cx > 0. && cx + w > width {
                (cx, row) = (0., row + 1);
            }
            if row > 1 {
                break;
            }
            self.push(
                -999,
                x + cx,
                y + row as f32 * 38.,
                w,
                30.,
                13,
                label.as_str().into(),
                Default::default(),
                false,
                false,
            );
            self.nodes.last_mut().unwrap().size = 18.;
            cx += w + 8.;
        }
        if labels.is_empty() { 0 } else { row.min(1) + 1 }
    }
    /// The Discover category strip at the top of the grid. Chips wider than the
    /// grid scroll: `start` is the first chip drawn, kept so `selected` and
    /// `focused` stay visible. Returns the new `start`.
    fn strip(
        &mut self,
        chips: &[(String, bool)],
        selected: usize,
        start: usize,
        focused: Option<usize>,
    ) -> usize {
        let widths: Vec<f32> = chips.iter().map(|(l, _)| chip_width(l, 20.)).collect();
        let span = |a: usize, b: usize| widths[a..=b].iter().sum::<f32>() + 12. * (b - a) as f32;
        let overflow = span(0, chips.len() - 1) > STRIP_WIDTH;
        let room = if overflow {
            STRIP_WIDTH - 72.
        } else {
            STRIP_WIDTH
        };
        let anchor = focused.unwrap_or(selected).min(chips.len() - 1);
        let mut start = if overflow { start.min(anchor) } else { 0 };
        while overflow && start < anchor && span(start, anchor) > room {
            start += 1;
        }
        let mut x = if overflow { 48. } else { 12. };
        let mut last = start;
        for (k, (label, on)) in chips.iter().enumerate().skip(start) {
            if x + widths[k] > 12. + STRIP_WIDTH - if overflow { 36. } else { 0. } && k > anchor {
                break;
            }
            self.choice(
                CHIP + k as i32,
                x,
                12.,
                widths[k],
                label.as_str().into(),
                *on,
            );
            self.nodes.last_mut().unwrap().kind = 14;
            x += widths[k] + 12.;
            last = k;
        }
        if overflow {
            for (hidden, gx) in [(start > 0, 12.), (last + 1 < chips.len(), 1176.)] {
                if hidden {
                    let glyph = if gx < 100. { "‹" } else { "›" };
                    self.label(gx, 12., 36., 52., glyph.into(), 40.);
                }
            }
        }
        start
    }
    #[allow(clippy::too_many_arguments)]
    fn compact(
        &mut self,
        id: i32,
        x: f32,
        y: f32,
        title: SharedString,
        secondary: SharedString,
        status: SharedString,
        favorite: bool,
    ) {
        self.push(id, x, y, 1200., 74., 24, title, secondary, true, false);
        self.author(x + 958., y + 25., 224., 28., status, 20.);
        if favorite {
            self.favorite(x + 908., y + 22.);
        }
    }
    fn card(&mut self, id: i32, x: f32, y: f32, w: f32, h: f32) {
        self.push(
            id,
            x,
            y,
            w,
            h,
            1,
            Default::default(),
            Default::default(),
            true,
            false,
        );
    }
}
/// The drawable size of DDL's fullscreen window (spec 064, R6).
///
/// Slint reports physical pixels, which is what DSDA-Doom needs; gamescope makes
/// that the output resolution of the Deck panel or an attached TV. A windowed UI
/// has no display-sized drawable, so the template's `screen_resolution` stays the
/// fallback. Sizes outside DSDA's accepted range are ignored rather than guessed.
fn drawable_screen(ui: &AppWindow) -> Option<ddl_core::engines::dsda::ScreenSize> {
    let window = ui.window();
    screen_for_drawable(
        window.is_fullscreen(),
        window.size().width,
        window.size().height,
    )
}

/// The fullscreen/drawable decision behind [`drawable_screen`], testable without
/// a backend (spec 064, R6). Windowed measurements keep the config fallback.
fn screen_for_drawable(
    fullscreen: bool,
    width: u32,
    height: u32,
) -> Option<ddl_core::engines::dsda::ScreenSize> {
    if !fullscreen {
        return None;
    }
    ddl_core::engines::dsda::ScreenSize::new(width, height).ok()
}

pub fn attach(ui: &AppWindow) -> Result<Rc<RefCell<Runtime>>, Box<dyn std::error::Error>> {
    let paths = ddl_core::platform::paths::resolve(&|key: &str| std::env::var(key).ok())?;
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is missing")?);
    let env = Environment {
        paths,
        home,
        screen: None,
        #[cfg(test)]
        test: None,
    };
    Ok(attach_environment(ui, env))
}
pub fn attach_environment(ui: &AppWindow, env: Environment) -> Rc<RefCell<Runtime>> {
    attach_with_sandbox(ui, env, services::default_sandbox())
}
pub fn attach_with_sandbox(
    ui: &AppWindow,
    env: Environment,
    sandbox: Sandbox,
) -> Rc<RefCell<Runtime>> {
    tracing::info!(
        flatpak = sandbox.is_flatpak(),
        app_id = sandbox.app_id().unwrap_or_default(),
        "sandbox detected"
    );
    let state = Rc::new(RefCell::new(
        Runtime::new(env).with_sandbox(sandbox.clone()),
    ));
    let weak = ui.as_weak();
    let s = state.clone();
    ui.on_action(move |code| {
        if let (Some(ui), Some(action)) = (weak.upgrade(), Action::from_code(code)) {
            s.borrow_mut().action(&ui, action);
        }
    });
    let weak = ui.as_weak();
    let s = state.clone();
    ui.on_clicked(move |id| {
        if let Some(ui) = weak.upgrade() {
            s.borrow_mut().click(&ui, id);
        }
    });
    let weak = ui.as_weak();
    let s = state.clone();
    ui.on_services_ready(move || {
        if let Some(ui) = weak.upgrade() {
            s.borrow_mut().drain(&ui);
        }
    });
    ui.on_profile_name_focused(move || {
        if let Err(error) = osk::focused(&osk::SystemLauncher, &sandbox) {
            tracing::debug!(%error, "Steam keyboard unavailable; use Steam + X");
        }
    });
    state.borrow_mut().start(ui, Command::Startup);
    state.borrow_mut().sync(ui);
    state
}

/// The Settings → About sandbox line (spec 059 R1).
fn sandbox_text(ui: &AppWindow, sandbox: &Sandbox) -> SharedString {
    match (sandbox.is_flatpak(), sandbox.app_id()) {
        (false, _) => text(ui, T::FlatpakSandboxNo),
        (true, Some(id)) => value(ui, T::FlatpakSandboxYes, id),
        (true, None) => text(ui, T::FlatpakSandboxYesPlain),
    }
}

/// "Flatpak sandbox: yes (<app id>)" / "no" as plain text (spec 059 R1).
fn sandbox_line(sandbox: &Sandbox) -> String {
    match (sandbox.is_flatpak(), sandbox.app_id()) {
        (false, _) => "Flatpak sandbox: no".to_owned(),
        (true, Some(id)) => format!("Flatpak sandbox: yes ({id})"),
        (true, None) => "Flatpak sandbox: yes".to_owned(),
    }
}

fn display_title(recipe: &ddl_core::catalog::Recipe) -> &str {
    recipe.title.split(" (").next().unwrap_or(&recipe.title)
}

/// Spec 077, R1: whether a startup should check the catalog. Pure, so the
/// once-per-start, production and opt-out rules are tested without a window.
fn auto_check_due(production: bool, already_checked: bool, auto_update: bool) -> bool {
    production && !already_checked && auto_update
}

/// "Catalog: <serial date> (built-in | updated)" for Settings (spec 019, R5).
/// A build that never learned its serial shows the source without a date.
fn catalog_summary(ui: &AppWindow, info: &services::CatalogInfo) -> SharedString {
    let state = text(
        ui,
        if info.updated {
            T::CatalogUpdatedLabel
        } else {
            T::CatalogBuiltIn
        },
    );
    match ddl_core::catalog::serial_date(info.serial) {
        Some(date) => {
            ui.global::<crate::WiringStrings>().set_other(state);
            value(ui, T::CatalogSummary, date)
        }
        None => value(ui, T::CatalogSummarySimple, state),
    }
}

/// Spec 077 R3: Discover's header subtitle is the effective catalog date/time.
/// An unknown serial must not borrow today's date.
fn catalog_subtitle(serial: u64) -> SharedString {
    ddl_core::catalog::serial_datetime(serial)
        .unwrap_or_else(|| "Catalog date unavailable".into())
        .into()
}

/// The download row's fill fraction and resume state (spec 040, R6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DownloadRow {
    /// 0..=999 of the bar, or `None` when no total is known.
    pub(crate) fraction: Option<i32>,
    /// Whether a range request is in flight ("Resuming…").
    pub(crate) resuming: bool,
}

/// Reads the download row from the latest install progress.
///
/// A resumed transfer reports the bytes already on disk, so the fraction
/// starts there instead of jumping back to zero (spec 040, R6).
pub(crate) fn download_row(progress: Option<InstallProgress>) -> DownloadRow {
    let transfer = progress
        .filter(|p| p.stage == InstallStage::Download)
        .and_then(|p| p.transfer);
    DownloadRow {
        fraction: transfer.and_then(|p| {
            p.total
                .filter(|total| *total > 0)
                .map(|total| (p.downloaded.saturating_mul(1000) / total).min(1000) as i32)
        }),
        resuming: transfer.is_some_and(|p| p.resuming),
    }
}

/// IEC byte units preserve useful transfer precision without invented speed estimates.
fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut amount = bytes as f64;
    let mut unit = 0;
    while amount >= 1024. && unit < UNITS.len() - 1 {
        amount /= 1024.;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{amount:.1} {}", UNITS[unit])
    }
}

/// Keep the destination end of a long breadcrumb. Character-based to preserve UTF-8.
fn elide_path_start(path: &str, limit: usize) -> String {
    let count = path.chars().count();
    if count <= limit {
        return path.into();
    }
    format!(
        "…{}",
        path.chars()
            .skip(count - limit.saturating_sub(1))
            .collect::<String>()
    )
}

#[cfg(test)]
mod visual_copy_tests {
    use super::*;
    use ddl_core::downloads::Progress;
    /// R6 (spec 064): only a fullscreen drawable size is used; windowed or
    /// invalid measurements keep the template fallback.
    #[test]
    fn the_drawable_screen_only_comes_from_a_fullscreen_window() {
        assert_eq!(screen_for_drawable(false, 1280, 800), None);
        assert_eq!(
            screen_for_drawable(true, 1920, 1080).map(|s| (s.width(), s.height())),
            Some((1920, 1080))
        );
        assert_eq!(screen_for_drawable(true, 240, 160), None);
        assert_eq!(screen_for_drawable(true, 0, 0), None);
    }
    #[test]
    fn recipe_art_selects_bundled_covers_and_generic_fallback() {
        assert_eq!(art("sigil"), 0);
        assert_eq!(art("sigil-ii"), 1);
        assert_eq!(art("auger-zenith"), 2);
        assert_eq!(art("eviternity"), 3);
        let catalog = ddl_core::catalog::Catalog::embedded();
        let recipe = catalog
            .recipe("eviternity")
            .expect("real bundled Eviternity recipe");
        assert_eq!(art(&recipe.id), 3);
        for id in ["community-without-art", "", "SIGIL"] {
            assert_eq!(art(id), 4);
        }
    }
    #[test]
    fn a_resumed_download_row_starts_at_the_bytes_on_disk() {
        let fresh = download_row(Some(InstallProgress::transfer(Progress::new(
            500,
            Some(1000),
        ))));
        assert_eq!(fresh.fraction, Some(500));
        assert!(!fresh.resuming);

        let resumed = download_row(Some(InstallProgress::transfer(Progress {
            downloaded: 620,
            total: Some(1000),
            resuming: true,
        })));
        assert_eq!(resumed.fraction, Some(620), "no jump back to 0%");
        assert!(resumed.resuming);

        // Only a download stage feeds the row; verify/commit carry no bytes.
        assert_eq!(
            download_row(Some(InstallProgress::stage(InstallStage::Verify))).fraction,
            None
        );
        assert!(!download_row(None).resuming);
    }

    #[test]
    fn byte_units_and_unicode_breadcrumbs_are_human_readable() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1023), "1023 B");
        assert_eq!(human_bytes(1536), "1.5 KiB");
        assert_eq!(human_bytes(2 * 1024 * 1024), "2.0 MiB");
        assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.0 GiB");
        assert_eq!(elide_path_start("/short", 10), "/short");
        assert_eq!(elide_path_start("/long/日本語/maps", 9), "…日本語/maps");
    }

    #[test]
    fn engine_runtime_problems_carry_their_actionable_detail() {
        let missing =
            Problem::EngineRuntime(ddl_core::engines::dsda::RuntimeError::MissingLibrary {
                library: "libOpenGL.so.0".to_owned(),
            });
        let detail = problem_detail(&missing).expect("a runtime problem has a detail");
        assert!(detail.contains("libOpenGL.so.0"), "{detail}");
        assert!(matches!(problem_copy(&missing), T::ErrorEngineRuntime));

        let glibc = Problem::EngineRuntime(ddl_core::engines::dsda::RuntimeError::Glibc {
            missing: vec!["GLIBC_2.34".to_owned()],
        });
        let detail = problem_detail(&glibc).expect("a runtime problem has a detail");
        assert!(detail.contains("GLIBC_2.34"), "{detail}");
        assert!(detail.contains("Compatibility tool"), "{detail}");

        // Every other problem stays a static key.
        assert!(problem_detail(&Problem::Engine).is_none());
    }

    /// Spec 098, R2: a catalog that needs a newer DDL names the version and
    /// asks for an app update instead of the "Connect and Retry" copy.
    #[test]
    fn a_catalog_that_needs_a_newer_ddl_asks_for_an_app_update() {
        if crate::headless::isolated(
            module_path!(),
            "a_catalog_that_needs_a_newer_ddl_asks_for_an_app_update",
        ) {
            return;
        }
        let (_window, ui) = crate::headless::setup();

        let named = problem_text(
            &ui,
            &Problem::CatalogNeedsNewer {
                min: Some("0.0.12".to_owned()),
            },
        );
        assert!(named.contains("0.0.12"), "{named}");
        assert!(named.to_lowercase().contains("update ddl"), "{named}");
        assert!(!named.to_lowercase().contains("retry"), "{named}");

        let unnamed = problem_text(&ui, &Problem::CatalogNeedsNewer { min: None });
        assert!(unnamed.to_lowercase().contains("update ddl"), "{unnamed}");
        assert!(!unnamed.to_lowercase().contains("retry"), "{unnamed}");

        // The retryable failure keeps its own copy.
        let retry = problem_text(&ui, &Problem::Catalog);
        assert!(retry.contains("Retry"), "{retry}");
        assert!(!retry.to_lowercase().contains("update ddl"), "{retry}");
    }
}

#[cfg(test)]
mod category_tests {
    use super::*;
    use ddl_core::catalog::Catalog;

    fn chips(labels: &[&str], selected: usize) -> Vec<(String, bool)> {
        labels
            .iter()
            .enumerate()
            .map(|(i, l)| ((*l).to_owned(), i == selected))
            .collect()
    }
    fn drawn(b: &Nodes) -> Vec<i32> {
        b.nodes
            .iter()
            .filter(|n| n.kind == 14)
            .map(|n| n.id - CHIP)
            .collect()
    }
    #[test]
    fn requested_categories_keep_the_owner_order_and_omit_other_navigation() {
        let catalog = Catalog::embedded();
        let used = requested_categories(|id| {
            catalog
                .recipes
                .iter()
                .any(|r| r.recipe.tags.iter().any(|tag| tag == id))
        });
        let ids: Vec<&str> = used
            .iter()
            .map(|c| match c {
                Category::Tag(id) => id.as_str(),
                Category::Favorites => "Favorites",
            })
            .collect();
        let expected: Vec<&str> = CATEGORY_ORDER
            .into_iter()
            .filter(|id| ids.contains(id))
            .collect();
        // `ids` follows the same constant, so also pin the owner's relative order.
        let position = |id: &str| ids.iter().position(|candidate| *candidate == id).unwrap();
        assert!(position("cacowards") < position("megawad"));
        assert!(position("megawad") < position("community-projects"));
        assert!(position("community-projects") < position("classics"));
        assert!(position("classics") < position("episode"));
        assert!(position("episode") < position("challenging"));
        assert!(position("challenging") < position("gimmick"));
        assert_eq!(ids, expected, "owner order, only used requested ids");
        assert!(!ids.contains(&"featured"), "Featured is hidden");
        assert!(!ids.contains(&"beginner-friendly"), "unrequested is hidden");
        assert!(!ids.contains(&"All"), "All no longer exists");
        assert_eq!(first_available(&used), Category::Tag("cacowards".into()));
        assert_eq!(
            first_available(&[Category::Favorites]),
            Category::Favorites,
            "an empty catalog still has a safe category"
        );
        assert_eq!(first_available(&[]), Category::Favorites);
    }

    #[test]
    fn every_embedded_mapset_is_reachable_from_a_requested_category() {
        for recipe in &Catalog::embedded().recipes {
            assert!(
                CATEGORY_ORDER
                    .iter()
                    .any(|id| recipe.recipe.tags.iter().any(|tag| tag == id)),
                "{} is only in hidden categories",
                recipe.recipe.id
            );
        }
    }

    #[test]
    fn a_strip_that_fits_is_drawn_whole_without_arrows() {
        let mut b = Nodes { nodes: vec![] };
        let start = b.strip(&chips(&["All", "Episodes", "Favorites"], 0), 0, 0, None);
        assert_eq!((start, drawn(&b)), (0, vec![0, 1, 2]));
        assert!(b.nodes.iter().all(|n| n.kind != 0));
        assert!(b.nodes.iter().all(|n| n.x + n.w <= 12. + STRIP_WIDTH));
    }
    #[test]
    fn a_wide_strip_scrolls_to_keep_the_selected_and_focused_chip_visible() {
        let labels = [
            "All",
            "Featured",
            "Beginner Friendly",
            "Classics",
            "Cacowards",
            "Community Projects",
            "Megawads",
            "Episodes",
            "Challenging",
            "Favorites",
        ];
        for selected in 0..labels.len() {
            let mut b = Nodes { nodes: vec![] };
            let start = b.strip(&chips(&labels, selected), selected, 0, Some(selected));
            let shown = drawn(&b);
            assert!(shown.contains(&(selected as i32)), "{selected}: {shown:?}");
            assert_eq!(shown[0] as usize, start);
            assert!(
                b.nodes
                    .iter()
                    .all(|n| n.x >= 0. && n.x + n.w <= 12. + STRIP_WIDTH),
                "{selected}: strip inside the grid width"
            );
            let arrows: Vec<_> = b.nodes.iter().filter(|n| n.kind == 0).collect();
            assert_eq!(arrows.iter().any(|n| n.label == "‹"), start > 0);
            assert_eq!(
                arrows.iter().any(|n| n.label == "›"),
                *shown.last().unwrap() as usize + 1 < labels.len()
            );
        }
        // The window stays put while the selection moves inside it.
        let mut b = Nodes { nodes: vec![] };
        let held = b.strip(&chips(&labels, 5), 5, 3, Some(5));
        assert_eq!(held, 3);
    }
    #[test]
    fn cards_show_at_most_two_tag_chips_led_by_the_open_category() {
        let catalog = Catalog::embedded();
        let recipe = catalog.recipe("eviternity").unwrap();
        let s = Snapshot {
            items: vec![],
            base_games: vec![],
            automatic_folders: vec![],
            custom: vec![],
            custom_profiles: vec![],
            local_issues: Default::default(),
            profiles_with_saves: Default::default(),
            tags: catalog.tags.clone(),
            engine: None,
            previous: None,
            attribution: None,
            iwads: vec![],
            folders: vec![],
            hidden_paths: vec![],
            config_notice: false,
            config_usable: true,
            config_version: String::new(),
            audio: crate::services::AudioState {
                choice: None,
                installed: false,
                current: ddl_core::soundfonts::engine::Current::Default,
                warning: None,
            },
            logs: PathBuf::new(),
            search_standard_folders: false,
            catalog: crate::services::CatalogInfo {
                serial: 0,
                updated: false,
                source_commit: String::new(),
                fetched_at: None,
                auto_update: true,
                last_check: None,
            },
        };
        assert_eq!(
            tag_labels(&s, recipe, 2, None),
            ["Cacowards", "Megawads"],
            "vocabulary order after the 073 catalog"
        );
        assert_eq!(
            tag_labels(&s, recipe, 2, Some("megawad")),
            ["Megawads", "Cacowards"],
            "the open category leads"
        );
        assert_eq!(
            tag_labels(&s, recipe, usize::MAX, None).len(),
            recipe.tags.len()
        );
        let mut b = Nodes { nodes: vec![] };
        let labels = tag_labels(&s, recipe, usize::MAX, None);
        assert_eq!(b.chips(&labels, 0., 0., 760.), 1);
        let many: Vec<String> = (0..12).map(|n| format!("Long tag name {n}")).collect();
        let mut b = Nodes { nodes: vec![] };
        assert_eq!(
            b.chips(&many, 0., 0., 760.),
            2,
            "wraps, then drops the rest"
        );
        assert!(b.nodes.iter().all(|n| n.x + n.w <= 760.));
    }
}

#[cfg(test)]
mod startup_and_pistol_tests {
    use super::*;
    use ddl_core::catalog::Catalog;
    use ddl_core::engines::dsda::{Complevel, ComplevelWord};
    use ddl_core::library::RecipeStatus;
    use ddl_core::profiles::Profile;
    use ddl_core::resolver::{Summary, SummaryContent};

    /// Spec 077, R1: production only, once per start, opt-out respected — and no
    /// 24 h or 10 min throttle anywhere in the rule.
    #[test]
    fn a_startup_checks_once_per_start_unless_opted_out_or_not_production() {
        assert!(auto_check_due(true, false, true));
        assert!(
            !auto_check_due(true, true, true),
            "already checked this start"
        );
        assert!(!auto_check_due(true, false, false), "automatic updates off");
        assert!(!auto_check_due(false, false, true), "tests and dev builds");
    }

    #[test]
    fn compact_toggle_contexts_empty_lists_and_save_failure_remain_usable() {
        with_ui(|ui| {
            let temp = tempfile::tempdir().unwrap();
            let mut env = test_environment();
            env.paths.config = temp.path().join("config");
            let mut r = Runtime::new(env.clone());
            r.catalog_checked = true;
            for screen in [
                Screen::Loading,
                Screen::FirstRun,
                Screen::Recovery,
                Screen::Home,
                Screen::Settings,
                Screen::Detail,
                Screen::Setup,
                Screen::Progress,
                Screen::Review,
                Screen::Browser,
            ] {
                r.screen = screen;
                r.action(ui, Action::ToggleView);
                assert_eq!(r.view_format, services::ViewFormat::Cards, "{screen:?}");
                assert!(r.view_writer.is_none());
            }
            r.screen = Screen::Discover;
            let mut empty = snapshot_with_sigil(false);
            empty.items.clear();
            r.snapshot = Some(empty);
            r.choose_category(Category::Favorites);
            r.focus = CHIP;
            r.action(ui, Action::ToggleView);
            assert_eq!(r.view_format, services::ViewFormat::List);
            assert_eq!(r.focus, CHIP, "empty category control survives");
            r.screen = Screen::Library;
            r.focus = -2;
            r.action(ui, Action::ToggleView);
            assert_eq!(r.view_format, services::ViewFormat::Cards);
            assert_eq!(r.focus, -2);
            r.playing = true;
            r.action(ui, Action::ToggleView);
            r.playing = false;
            r.neutral = true;
            r.action(ui, Action::ToggleView);
            r.neutral = false;
            r.sheet = Some(Sheet::Quit);
            r.action(ui, Action::ToggleView);
            r.sheet = None;
            r.local.dialog = Some(local_ui::Dialog::Saves);
            r.action(ui, Action::ToggleView);
            r.local.dialog = None;
            for view in [
                local_ui::View::New,
                local_ui::View::Base,
                local_ui::View::Detail,
                local_ui::View::BaseSetup,
                local_ui::View::Picker,
            ] {
                r.local.view = Some(view);
                r.action(ui, Action::ToggleView);
            }
            assert_eq!(r.view_format, services::ViewFormat::Cards);
            r.local.view = None;
            r.finish_preferences();
            assert_eq!(
                services::load_view_format(&env).unwrap(),
                services::ViewFormat::Cards
            );
            std::fs::remove_file(env.paths.config.join("view.json")).unwrap();
            std::fs::create_dir(env.paths.config.join("view.json")).unwrap();
            r.action(ui, Action::ToggleView);
            assert_eq!(r.view_format, services::ViewFormat::List);
            for _ in 0..200 {
                r.drain(ui);
                if !ui.get_toast().is_empty() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert_eq!(ui.get_toast(), text(ui, T::ErrorStorage));
            assert_eq!(
                r.view_format,
                services::ViewFormat::List,
                "failed save keeps the usable layout"
            );
            assert!(!r.busy && r.pending.is_none());
            r.finish_preferences();
        });
    }

    #[test]
    fn compact_layout_survives_stale_snapshot_and_preserves_separate_focus_identity() {
        let mut r = Runtime::new(test_environment());
        let mut first = snapshot_with_sigil(false);
        let mut other = first.items[0].clone();
        other.recipe.id = "another".into();
        first.items.push(other);
        r.snapshot = Some(first.clone());
        r.screen = Screen::Discover;
        r.selected = 0;
        r.focus = CARD + 1;
        r.view_loaded = true;
        r.view_format = services::ViewFormat::List;
        first.items.swap(0, 1);
        r.replace_snapshot(first);
        assert_eq!(r.focus, CARD);
        assert_eq!(r.selected, 1);
        assert_eq!(r.view_format, services::ViewFormat::List);
    }

    fn test_environment() -> Environment {
        // Unique per call: tests run in parallel and must not share state.
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let home = std::env::temp_dir().join(format!(
            "ddl-075-077-runtime-{}-{unique}",
            std::process::id()
        ));
        let paths = ddl_core::platform::paths::resolve(&|key: &str| match key {
            "HOME" => Some(home.display().to_string()),
            _ => None,
        })
        .expect("paths resolve from HOME");
        Environment {
            paths,
            home,
            screen: None,
            test: None,
        }
    }

    /// One Discover/Library row for `sigil` whose launch summary shows the
    /// profile's Pistol start value.
    fn snapshot_with_sigil(pistol_start: bool) -> Snapshot {
        let catalog = Catalog::embedded();
        let recipe = catalog.recipe("sigil").expect("sigil").clone();
        let profile = Profile::new("sigil-default", "Default", "sigil");
        let summary = Summary {
            base_game: recipe.base_game.game,
            iwad_present: true,
            iwad_variant: None,
            iwad_path: PathBuf::from("/tmp/doom2.wad"),
            content: vec![SummaryContent {
                position: 1,
                label: "Main".into(),
                path: PathBuf::from("/tmp/sigil.wad"),
            }],
            complevel: Complevel::from_word(ComplevelWord::Vanilla, 2).expect("vanilla"),
            skill: 3,
            pistol_start,
            engine_version: "0.30.1".into(),
        };
        Snapshot {
            items: vec![Item {
                recipe,
                manifest: None,
                profile,
                profiles: vec![],
                last_played: None,
                launch: Ok(summary),
                favorite: false,
                recipe_status: RecipeStatus::Current,
                change_summary: String::new(),
                update_dismissed: false,
                update_notice: vec![],
            }],
            base_games: vec![],
            automatic_folders: vec![],
            custom: vec![],
            custom_profiles: vec![],
            local_issues: Default::default(),
            profiles_with_saves: Default::default(),
            tags: catalog.tags.clone(),
            engine: None,
            previous: None,
            attribution: None,
            iwads: vec![],
            folders: vec![],
            hidden_paths: vec![],
            config_notice: false,
            config_usable: true,
            config_version: String::new(),
            audio: crate::services::AudioState {
                choice: None,
                installed: false,
                current: ddl_core::soundfonts::engine::Current::Default,
                warning: None,
            },
            logs: PathBuf::new(),
            search_standard_folders: false,
            catalog: crate::services::CatalogInfo {
                serial: 0,
                updated: false,
                source_commit: String::new(),
                fetched_at: None,
                auto_update: true,
                last_check: None,
            },
        }
    }

    // One headless UI per test thread: the Slint platform is installed once for
    // the thread, and every phase borrows the same component.
    thread_local! {
        static HEADLESS: std::cell::RefCell<
            Option<(
                std::rc::Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
                AppWindow,
            )>,
        > = const { std::cell::RefCell::new(None) };
    }

    fn with_ui<R>(f: impl FnOnce(&AppWindow) -> R) -> R {
        HEADLESS.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_none() {
                *slot = Some(crate::headless::setup());
            }
            let held = slot.as_ref().expect("headless setup");
            f(&held.1)
        })
    }

    /// Pumps worker events until every Pistol request settled.
    fn settle(runtime: &mut Runtime, ui: &AppWindow) {
        for _ in 0..600 {
            crate::headless::pump_events();
            runtime.drain(ui);
            if !runtime.busy
                && !runtime.pistol_in_flight
                && runtime.pistol_queue.is_none()
                && runtime.base_pistol_queue.is_none()
            {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the Pistol requests did not settle");
    }

    fn persist_catalog_profile(env: &Environment) {
        let mut services = services::Services::open(env.clone()).expect("services open");
        let profile = services
            .snapshot()
            .expect("snapshot")
            .items
            .into_iter()
            .find(|item| item.recipe.id == "sigil")
            .expect("sigil item")
            .profile;
        services
            .execute(
                services::Command::Save(profile),
                &CancellationToken::new(),
                &mut |_| {},
            )
            .expect("profile saved");
    }

    fn catalog_pistol_start(env: &Environment) -> bool {
        services::Services::open(env.clone())
            .expect("reopen")
            .snapshot()
            .expect("snapshot")
            .items
            .into_iter()
            .find(|item| item.recipe.id == "sigil")
            .expect("sigil item")
            .profile
            .pistol_start
    }

    /// Spec 075, R2 review: a save result that arrives **after** a newer press
    /// must not invert the visible row — the intent wins — and the next press
    /// still persists correctly (deterministic event injection, no timing).
    #[test]
    fn a_stale_saved_result_never_inverts_the_latest_intent() {
        if crate::headless::isolated(
            module_path!(),
            "a_stale_saved_result_never_inverts_the_latest_intent",
        ) {
            return;
        }
        with_ui(|ui| {
            let env = test_environment();
            persist_catalog_profile(&env);
            let mut runtime = Runtime::new(env.clone());
            let mut snapshot = snapshot_with_sigil(false);
            // A selected-profile mirror the row can drift from if it is not kept
            // in step with the visible value.
            snapshot.items[0].profiles = vec![snapshot.items[0].profile.clone()];
            runtime.snapshot = Some(snapshot);
            runtime.screen = Screen::Setup;
            runtime.focus = 41;

            // Two presses while the first write is in flight: the latest intent
            // is `false` (the row was flipped on, then off).
            runtime.busy = true;
            runtime.set_pistol_start(ui, "sigil".into(), "sigil-default".into(), true);
            runtime.set_pistol_start(ui, "sigil".into(), "sigil-default".into(), false);
            // The first write's result (true) lands late.
            let mut saved = runtime.snapshot.as_ref().unwrap().items[0].profile.clone();
            saved.pistol_start = true;
            runtime.apply_saved_profile(&saved);

            let item = &runtime.snapshot.as_ref().unwrap().items[0];
            assert!(
                !item.profile.pistol_start,
                "the latest intent stays visible"
            );
            assert!(
                !item.profiles[0].pistol_start,
                "the selected-profile mirror follows the visible value"
            );
            assert_eq!(
                runtime.pistol_baseline.get("sigil-default").copied(),
                Some(true),
                "the stale result is the persisted baseline"
            );

            // The next press requests on again and must end persisted.
            runtime.busy = false;
            runtime.action(ui, Action::Confirm);
            settle(&mut runtime, ui);
            assert!(
                catalog_pistol_start(&env),
                "the final requested value is durable"
            );
        });
    }

    /// Spec 075, R2 review: three quick Confirm presses on the real route keep
    /// the last intent (on), and a later failing press restores the persisted
    /// value instead of a stale one.
    #[test]
    fn rapid_confirm_presses_on_a_mapset_keep_the_last_intent() {
        if crate::headless::isolated(
            module_path!(),
            "rapid_confirm_presses_on_a_mapset_keep_the_last_intent",
        ) {
            return;
        }
        with_ui(|ui| {
            let env = test_environment();
            persist_catalog_profile(&env);
            let mut runtime = Runtime::new(env.clone());
            runtime.snapshot = Some(snapshot_with_sigil(false));
            runtime.screen = Screen::Setup;
            runtime.focus = 41;

            // false -> true (dispatch), -> false (queued), -> true (queued)
            runtime.action(ui, Action::Confirm);
            runtime.action(ui, Action::Confirm);
            runtime.action(ui, Action::Confirm);
            settle(&mut runtime, ui);
            assert_eq!(
                runtime.pistol_intent.get("sigil-default").copied(),
                Some(true),
                "the latest intent wins"
            );
            assert!(
                catalog_pistol_start(&env),
                "the last requested value is durable"
            );
            assert!(
                matches!(&runtime.snapshot.as_ref().unwrap().items[0].launch, Ok(summary) if summary.pistol_start),
                "the launch summary follows the row"
            );

            // Failure phase: two quick presses, both writes fail.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let profiles = env.paths.config.join("profiles");
                let mut permissions = std::fs::metadata(&profiles).unwrap().permissions();
                permissions.set_mode(0o500);
                std::fs::set_permissions(&profiles, permissions).unwrap();
                runtime.problem = None;
                runtime.action(ui, Action::Confirm);
                runtime.action(ui, Action::Confirm);
                settle(&mut runtime, ui);
                let mut restore = std::fs::metadata(&profiles).unwrap().permissions();
                restore.set_mode(0o700);
                std::fs::set_permissions(&profiles, restore).unwrap();

                assert!(runtime.problem.is_some(), "the failure is visible");
                assert!(
                    runtime.pistol_intent.is_empty(),
                    "the failed intent is dropped"
                );
                assert!(runtime.pistol_queue.is_none(), "no stale queue entry");
                assert!(catalog_pistol_start(&env), "disk keeps the persisted value");
                assert!(
                    runtime.snapshot.as_ref().unwrap().items[0]
                        .profile
                        .pistol_start,
                    "the row shows the persisted value, not the failed one"
                );
            }
        });
    }

    /// Spec 075, R2 review: the base-game route behaves the same way for three
    /// quick presses and for a failing write.
    #[cfg(unix)]
    #[test]
    fn rapid_confirm_presses_on_a_base_game_keep_the_last_intent() {
        use std::os::unix::fs::PermissionsExt;
        if crate::headless::isolated(
            module_path!(),
            "rapid_confirm_presses_on_a_base_game_keep_the_last_intent",
        ) {
            return;
        }
        with_ui(|ui| {
            let env = test_environment();
            std::fs::create_dir_all(env.paths.config.as_path()).unwrap();
            std::fs::create_dir_all(env.paths.data.as_path()).unwrap();
            let doom2 = env.paths.data.join("doom2.wad");
            std::fs::write(&doom2, b"IWAD").unwrap();
            let size = std::fs::metadata(&doom2).unwrap().len();
            // A minimal identified Doom II registry entry: the base-game row
            // needs one, and the file must exist for `base_games::list`.
            std::fs::write(
                env.paths.config.join("iwads.json"),
                format!(
                    r#"{{"schema_version":1,"entries":[{{"path":{},"variant":"doom2-1.9","game":"doom2","sha256":"{}","size":{size},"mtime":{{"secs":1700000000,"nanos":0}},"source":"userfolder"}}]}}"#,
                    serde_json::to_string(&doom2.display().to_string()).unwrap(),
                    "0".repeat(64),
                ),
            )
            .unwrap();
            let mut services = services::Services::open(env.clone()).expect("services open");
            let base = services.base_item("base:doom2", None).expect("base item");
            let profile_id = base.profile.id.clone();
            services
                .execute(
                    services::Command::Local(services::LocalCommand::BaseSave(
                        base.profile.clone(),
                    )),
                    &CancellationToken::new(),
                    &mut |_| {},
                )
                .expect("base profile saved");
            drop(services);

            let mut runtime = Runtime::new(env.clone());
            let mut snapshot = snapshot_with_sigil(false);
            snapshot.base_games = vec![base];
            runtime.snapshot = Some(snapshot);
            runtime.screen = Screen::Home;
            runtime.local.open_base_for_tests("base:doom2");
            runtime.focus = 5;

            runtime.action(ui, Action::Confirm);
            runtime.action(ui, Action::Confirm);
            runtime.action(ui, Action::Confirm);
            settle(&mut runtime, ui);
            assert_eq!(
                runtime.pistol_intent.get(&profile_id).copied(),
                Some(true),
                "the latest base intent wins"
            );
            assert!(
                runtime.snapshot.as_ref().unwrap().base_games[0]
                    .profile
                    .pistol_start,
                "the base row follows the intent"
            );
            assert!(
                services::Services::open(env.clone())
                    .expect("reopen")
                    .base_item("base:doom2", None)
                    .expect("base item")
                    .profile
                    .pistol_start,
                "the base value is durable"
            );

            // Failure phase.
            let profiles = env.paths.config.join("profiles");
            let mut permissions = std::fs::metadata(&profiles).unwrap().permissions();
            permissions.set_mode(0o500);
            std::fs::set_permissions(&profiles, permissions).unwrap();
            runtime.problem = None;
            runtime.action(ui, Action::Confirm);
            runtime.action(ui, Action::Confirm);
            settle(&mut runtime, ui);
            let mut restore = std::fs::metadata(&profiles).unwrap().permissions();
            restore.set_mode(0o700);
            std::fs::set_permissions(&profiles, restore).unwrap();

            assert!(runtime.problem.is_some(), "the base failure is visible");
            assert!(
                runtime.pistol_intent.is_empty(),
                "the base intent is dropped"
            );
            assert!(
                runtime.base_pistol_queue.is_none(),
                "no stale base queue entry"
            );
            assert!(
                runtime.snapshot.as_ref().unwrap().base_games[0]
                    .profile
                    .pistol_start,
                "the base row shows the persisted value"
            );
        });
    }

    /// Spec 075, R2: the row and the launch summary flip together, and a failed
    /// write puts the previous value back instead of showing it as saved.
    #[test]
    fn the_optimistic_pistol_value_is_reverted_when_the_write_fails() {
        let mut runtime = Runtime::new(test_environment());
        runtime.snapshot = Some(snapshot_with_sigil(false));

        let previous = runtime.patch_pistol_value("sigil-default", true);
        assert_eq!(previous, Some(false));
        let item = &runtime.snapshot.as_ref().unwrap().items[0];
        assert!(item.profile.pistol_start, "the row flips immediately");
        assert!(
            matches!(&item.launch, Ok(summary) if summary.pistol_start),
            "the launch summary follows the row"
        );

        runtime.pistol_in_flight = true;
        // No baseline was recorded, so the revert falls back to the value the
        // row had before the optimistic patch (false).
        runtime.revert_pistol_start("sigil-default");
        let item = &runtime.snapshot.as_ref().unwrap().items[0];
        assert!(!item.profile.pistol_start, "the failed value is undone");
        assert!(
            matches!(&item.launch, Ok(summary) if !summary.pistol_start),
            "the summary is restored too"
        );
        assert!(!runtime.pistol_in_flight, "nothing stays pending");

        // Another profile's row is never touched.
        assert_eq!(runtime.patch_pistol_value("other", true), None);
    }
    /// Spec 090 R5: a fresh snapshot keeps the selected mapset and the Discover
    /// card focus by recipe identity, even when the catalog inserted or
    /// reordered recipes (Event::Finished overwrites the previous snapshot).
    #[test]
    fn a_fresh_snapshot_remaps_selection_and_card_focus_by_recipe_id() {
        let mut runtime = Runtime::new(test_environment());
        let mut first = snapshot_with_sigil(false);
        let scythe = Catalog::embedded()
            .recipe("scythe")
            .expect("scythe")
            .clone();
        let mut other = first.items[0].clone();
        other.recipe = scythe;
        other.profile = Profile::new("scythe-default", "Default", "scythe");
        first.items.push(other);
        runtime.snapshot = Some(first);
        runtime.screen = Screen::Discover;
        runtime.selected = 1;
        runtime.focus = CARD + 1;

        // A new snapshot puts scythe first (the catalog gained a newer/older
        // entry or the order changed): the selection must follow the recipe.
        let mut second = runtime.snapshot.clone().unwrap();
        second.items.reverse();
        runtime.replace_snapshot(second);
        assert_eq!(
            runtime.snapshot.as_ref().unwrap().items[0].recipe.id,
            "scythe"
        );
        assert_eq!(runtime.selected, 0, "selected mapset follows its id");
        assert_eq!(runtime.focus, CARD, "card focus follows the same card");

        // If the selected recipe disappears, the index clamps to a valid one.
        let mut third = runtime.snapshot.clone().unwrap();
        third.items.remove(0);
        runtime.replace_snapshot(third);
        assert!(runtime.selected < runtime.snapshot.as_ref().unwrap().items.len());
        assert_eq!(runtime.focus, CARD + runtime.selected as i32);
    }
}
