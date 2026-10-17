//! Production presentation state. Workers own filesystem/network/engine operations.
use crate::{
    AppWindow, ViewNode,
    input::Action,
    services::{self, Command, Directory, Environment, Event, Item, Problem, Services, Snapshot},
    wiring_copy::{Text as T, text, value},
};
use ddl_core::{
    downloads::{CancellationToken, InstallProgress, InstallStage},
    resolver::Blocker,
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
/// A Discover category (spec 015, R4). The strip lists `All`, one `Tag` per
/// vocabulary tag some recipe uses, then `Favorites`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Category {
    All,
    Tag(String),
    Favorites,
}
/// First node id of the category strip; chip `k` has id `CHIP + k`.
const CHIP: i32 = 200;
const CHIPS_MAX: usize = 30;
/// "Show all mapsets" button of the empty Favorites state.
const SHOW_ALL: i32 = 230;
/// Horizontal room for the strip, between the card grid's edges.
const STRIP_WIDTH: f32 = 1200.;
/// Vertical room the strip takes above the Discover grid.
const STRIP_HEIGHT: f32 = 72.;
#[derive(Clone)]
enum Sheet {
    Quit,
    Cancel,
    Reset,
    Remove(PathBuf),
    Component(String),
    Profiles,
    ProfileNames,
    DeleteProfile(String, String),
}
pub struct Runtime {
    env: Environment,
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
    settings_section: usize,
    advanced: bool,
    /// Discover category, remembered for the session (spec 015, R4).
    category: Category,
    /// First strip chip drawn when the strip is wider than the grid.
    strip_start: usize,
    updates: Vec<String>,
    updates_checked: bool,
    pub last_exit: Option<ddl_core::engines::dsda::LaunchOutcome>,
}
impl Runtime {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
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
            settings_section: 0,
            advanced: false,
            category: Category::All,
            strip_start: 0,
            updates: vec![],
            updates_checked: false,
            last_exit: None,
        }
    }
    fn item(&self) -> Option<&Item> {
        self.snapshot.as_ref()?.items.get(self.selected)
    }
    fn root(&self) -> bool {
        matches!(
            self.screen,
            Screen::Home | Screen::Discover | Screen::Library | Screen::Settings
        )
    }
    fn enter(&mut self, screen: Screen, focus: i32) {
        self.history.push((self.screen, self.focus));
        self.screen = screen;
        self.focus = focus;
    }
    fn tab(&mut self, tab: usize) {
        if self.root() {
            if self.focus >= 0 {
                self.remembered[self.tab_index()] = self.focus;
            }
            self.screen = [
                Screen::Home,
                Screen::Discover,
                Screen::Library,
                Screen::Settings,
            ][tab.min(3)];
            self.focus = self.remembered[tab.min(3)];
            self.history.clear();
        }
    }
    fn show_sheet(&mut self, sheet: Sheet) {
        self.invoking = self.focus;
        self.focus = if matches!(sheet, Sheet::Profiles) {
            if self.item().is_some_and(|i| {
                i.profiles.len() > 1 && i.profiles.last().is_some_and(|p| p.id != i.profile.id)
            }) {
                911
            } else {
                912
            }
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
        if matches!(command, Command::Play(_)) {
            self.playing = true;
            self.neutral = true;
        }
        self.problem = None;
        ui.set_toast(SharedString::default());
        self.progress = None;
        self.cancel = CancellationToken::new();
        if self.screen != Screen::Progress {
            self.return_to = (self.screen, self.focus);
        }
        self.pending = Some(command.clone());
        self.retry = Some(command.clone());
        if matches!(command, Command::InstallEngine(_) | Command::Install(_))
            && self.screen != Screen::Progress
        {
            self.enter(Screen::Progress, 1);
        }
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
            let result = Services::open(env).and_then(|mut services| {
                services.execute(command, &cancel, &mut emit)?;
                services.snapshot().map(Box::new)
            });
            emit(Event::Finished(result));
        });
    }
    /// Same completion path is drained by the Slint event-loop callback and headless tests.
    pub fn drain(&mut self, ui: &AppWindow) {
        let events: Vec<_> = self
            .rx
            .as_ref()
            .map(|r| r.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                Event::Working => {}
                Event::Progress(p) => self.progress = Some(p),
                Event::Playing => {
                    self.playing = true;
                    self.neutral = true;
                }
                Event::Directory(d) => {
                    self.directory = Some(d);
                    self.screen = Screen::Browser;
                    self.focus = 1;
                }
                Event::Updates(u) => {
                    self.updates = u;
                    self.updates_checked = true;
                }
                Event::EngineExit(outcome) => {
                    if !outcome.exit_status.is_clean() {
                        ui.set_toast(value(ui, T::EngineFailed, outcome.log_path.display()));
                    }
                    self.last_exit = Some(outcome);
                }
                Event::Finished(result) => {
                    self.busy = false;
                    self.playing = false;
                    let command = self.pending.take();
                    match result {
                        Ok(snapshot) => {
                            self.snapshot = Some(*snapshot);
                            self.problem = None;
                            match command {
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
                                }
                                Some(Command::InstallEngine(_)) => {
                                    self.screen = self.return_to.0;
                                    self.focus = 2;
                                    self.history.pop();
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
                                Some(Command::Install(_)) => {
                                    self.screen = Screen::Setup;
                                    self.focus = 50;
                                    if self.return_to.0 == Screen::Setup {
                                        self.history.pop();
                                    }
                                }
                                Some(Command::Play(_)) => {
                                    self.screen = Screen::Setup;
                                    self.focus = 50;
                                }
                                Some(Command::Import(..)) => {
                                    self.screen = Screen::Setup;
                                    self.focus = 50;
                                    self.history.pop();
                                }
                                _ => {}
                            }
                        }
                        Err(problem) => {
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
        self.sync(ui);
    }
    pub fn hotplug(&self, ui: &AppWindow, connected: bool) {
        if !self.playing {
            ui.set_toast(text(
                ui,
                if connected {
                    T::Connected
                } else {
                    T::Disconnected
                },
            ));
        }
    }
    pub fn action(&mut self, ui: &AppWindow, action: Action) {
        if self.playing {
            return;
        }
        if self.neutral {
            return;
        }
        if matches!(self.sheet, Some(Sheet::Profiles | Sheet::ProfileNames)) {
            match action {
                Action::Back => {
                    self.sheet = None;
                    self.focus = self.invoking;
                }
                Action::Up | Action::Down | Action::Left | Action::Right => {
                    let saved =
                        std::mem::replace(&mut self.nodes, ui.get_sheet_nodes().iter().collect());
                    self.move_focus(action);
                    self.nodes = saved;
                }
                Action::Confirm if !self.busy => self.profile_action(ui),
                _ => {}
            }
            self.sync(ui);
            return;
        }
        if self.sheet.is_some() {
            match action {
                Action::Left => self.focus = 900,
                Action::Right => self.focus = 901,
                Action::Back => {
                    self.sheet = None;
                    self.focus = self.invoking;
                }
                Action::Confirm => {
                    let sheet = self.sheet.take().unwrap();
                    let confirm = self.focus == 901;
                    self.focus = self.invoking;
                    if confirm {
                        match sheet {
                            Sheet::Quit => ui.invoke_quit_requested(),
                            Sheet::Cancel => self.cancel.cancel(),
                            Sheet::Reset => self.start(ui, Command::Reset),
                            Sheet::Remove(path) => self.start(ui, Command::RemoveFolder(path)),
                            Sheet::Component(id) => self.toggle(ui, &id),
                            Sheet::DeleteProfile(id, _) => {
                                self.start(ui, Command::DeleteProfile(id))
                            }
                            Sheet::Profiles | Sheet::ProfileNames => {}
                        }
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
                if self.screen == Screen::Progress {
                    (self.screen, self.focus) = self.return_to;
                } else if self.screen == Screen::Browser {
                    self.import_component = None;
                    (self.screen, self.focus) = self.history.pop().unwrap_or(self.return_to);
                } else if self.screen == Screen::Settings && self.focus >= 70 {
                    self.focus = 1 + self.settings_section as i32;
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
            Action::Up if self.screen == Screen::Discover && (10..=99).contains(&self.focus) => {
                self.move_focus(action);
                if (CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus) {
                    self.focus = CHIP + self.category_index() as i32;
                }
            }
            Action::Down
                if self.screen == Screen::Discover
                    && (CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus)
                    && !self.discover_items().is_empty() =>
            {
                self.focus = 10 + self.discover_items()[0] as i32;
            }
            Action::Up | Action::Down | Action::Left | Action::Right => self.move_focus(action),
            Action::PreviousCategory | Action::NextCategory if self.screen == Screen::Discover => {
                self.step_category(if action == Action::NextCategory {
                    1
                } else {
                    -1
                });
            }
            Action::X => self.toggle_favorite(ui),
            Action::Tab(tab) => self.tab(tab),
            Action::PreviousTab | Action::NextTab => {
                let tab = self.tab_index();
                self.tab((tab + if action == Action::NextTab { 1 } else { 3 }) % 4);
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
        if self.sheet.is_some() {
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
            self.tab((-self.focus - 1) as usize);
            return;
        }
        if self.focus == 800 {
            if let Some(command) = self.retry.clone() {
                self.start(ui, command);
            }
            return;
        }
        if self.focus == 801 {
            self.enter(Screen::Settings, 1);
            self.settings_section = 3;
            return;
        }
        if self.busy {
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
                if self.screen == Screen::FirstRun || self.settings_section == 1 =>
            {
                self.browse(ui, None)
            }
            (Screen::FirstRun, 3) => self.start(ui, Command::Scan(None)),
            (Screen::FirstRun, 4) => {
                self.screen = Screen::Discover;
                self.focus = 10;
            }
            (Screen::FirstRun, 5) => {
                self.screen = Screen::Home;
                self.focus = 1;
            }
            (Screen::Home, 1) => {
                if let Some(snapshot) = &self.snapshot {
                    if let Some((i, _)) = snapshot
                        .items
                        .iter()
                        .enumerate()
                        .filter(|(_, i)| i.last_played.is_some())
                        .max_by_key(|(_, i)| i.last_played)
                    {
                        self.selected = i;
                        self.enter(Screen::Setup, 50);
                    } else {
                        self.screen = Screen::Discover;
                        self.focus = 10;
                    }
                }
            }
            (Screen::Home, 2) => {
                self.screen = Screen::Discover;
                self.focus = 10;
            }
            (Screen::Discover, id) if (CHIP..CHIP + CHIPS_MAX as i32).contains(&id) => {
                if let Some(category) = self.categories().get((id - CHIP) as usize) {
                    self.category = category.clone();
                }
                if let Some(&first) = self.discover_items().first() {
                    self.focus = 10 + first as i32;
                }
            }
            (Screen::Discover, SHOW_ALL) => {
                self.category = Category::All;
                self.focus = self
                    .discover_items()
                    .first()
                    .map_or(CHIP, |&i| 10 + i as i32);
            }
            (Screen::Discover | Screen::Library, 10..=99) => {
                if self.screen == Screen::Library
                    && self
                        .snapshot
                        .as_ref()
                        .is_some_and(|s| s.items.iter().all(|i| i.manifest.is_none()))
                {
                    self.screen = Screen::Discover;
                    self.focus = 10;
                } else {
                    self.selected = (self.focus - 10) as usize;
                    self.enter(Screen::Detail, 1);
                }
            }
            (Screen::Detail, 1) => {
                if let Some(item) = self.item() {
                    if item.manifest.is_some() {
                        self.enter(Screen::Setup, 50);
                    } else {
                        self.start(ui, Command::Install(item.recipe.id.clone()));
                    }
                }
            }
            (Screen::Detail, 4) | (Screen::Setup, 60) => {
                if let Some(item) = self.item() {
                    // Materialise an implicit Default before any management action.
                    self.start(ui, Command::Save(item.profile.clone()));
                    self.show_sheet(Sheet::Profiles);
                }
            }
            (Screen::Detail, 2) => self.enter(Screen::Setup, 50),
            (Screen::Detail, 3) => self.toggle_favorite(ui),
            (Screen::Setup, 1) => {
                if self.snapshot.as_ref().is_some_and(|s| !s.config_usable) {
                    self.enter(Screen::Settings, 70);
                    self.settings_section = 2;
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
                    let mut profile = item.profile.clone();
                    profile.pistol_start = !profile.pistol_start;
                    self.start(ui, Command::Save(profile));
                }
            }
            (Screen::Setup, 42) => self.advanced = !self.advanced,
            (Screen::Setup, 43) => ui.set_toast(text(ui, T::OrderHelp)),
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
                        self.browse(ui, Some(c.id));
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
            (Screen::Settings, 1..=4) => {
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
                2 => self.show_sheet(Sheet::Reset),
                _ => {}
            },
            (Screen::Settings, 71) if self.settings_section == 0 => {
                self.start(ui, Command::Rollback)
            }
            (Screen::Settings, 71) if self.settings_section == 1 => {
                self.start(ui, Command::Scan(None))
            }
            (Screen::Settings, 72) if self.settings_section == 0 => {
                if let Some(version) = self.updates.first() {
                    self.start(ui, Command::InstallEngine(version.clone()));
                }
            }
            (Screen::Settings, 72) if self.settings_section == 1 => {
                if let Some(s) = &self.snapshot {
                    self.start(
                        ui,
                        Command::SetSearchStandardFolders(!s.search_standard_folders),
                    );
                }
            }
            (Screen::Settings, 100..) => {
                if self.settings_section == 1
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
    /// The strip: `All`, the vocabulary tags some recipe uses (in `order`), `Favorites`.
    fn categories(&self) -> Vec<Category> {
        let mut list = vec![Category::All];
        if let Some(s) = &self.snapshot {
            list.extend(
                s.tags
                    .ordered()
                    .into_iter()
                    .filter(|tag| s.items.iter().any(|i| i.recipe.tags.contains(&tag.id)))
                    .take(CHIPS_MAX - 2)
                    .map(|tag| Category::Tag(tag.id.clone())),
            );
        }
        list.push(Category::Favorites);
        list
    }
    fn category_label(&self, ui: &AppWindow, category: &Category) -> String {
        match category {
            Category::All => text(ui, T::AllCategory).into(),
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
    /// Snapshot indexes shown on Discover in the current category, favorites first.
    fn discover_items(&self) -> Vec<usize> {
        let Some(s) = &self.snapshot else {
            return vec![];
        };
        let mut shown: Vec<usize> = (0..s.items.len())
            .filter(|&i| match &self.category {
                Category::All => true,
                Category::Tag(id) => s.items[i].recipe.tags.contains(id),
                Category::Favorites => s.items[i].favorite,
            })
            .collect();
        shown.sort_by_key(|&i| !s.items[i].favorite);
        shown
    }
    /// Snapshot indexes shown on Library: installed mapsets, favorites first.
    fn library_items(&self) -> Vec<usize> {
        let Some(s) = &self.snapshot else {
            return vec![];
        };
        let mut shown: Vec<usize> = (0..s.items.len())
            .filter(|&i| s.items[i].manifest.is_some())
            .collect();
        shown.sort_by_key(|&i| !s.items[i].favorite);
        shown
    }
    /// Moves the category by `delta` without wrapping; the strip chip keeps focus
    /// when it had it, otherwise focus enters the new grid (or the chip when empty).
    fn step_category(&mut self, delta: isize) {
        let list = self.categories();
        let next = (self.category_index() as isize + delta).clamp(0, list.len() as isize - 1);
        self.category = list[next as usize].clone();
        if !(CHIP..CHIP + CHIPS_MAX as i32).contains(&self.focus) {
            self.focus = self
                .discover_items()
                .first()
                .map_or(CHIP + next as i32, |&i| 10 + i as i32);
        } else {
            self.focus = CHIP + next as i32;
        }
    }
    /// X: toggles the favorite mark of the focused card, or of the open detail.
    fn toggle_favorite(&mut self, ui: &AppWindow) {
        if self.busy {
            return;
        }
        let target = match self.screen {
            Screen::Discover | Screen::Library if (10..=99).contains(&self.focus) => {
                Some((self.focus - 10) as usize)
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
            Screen::Discover => 1,
            Screen::Library => 2,
            Screen::Settings => 3,
            _ => 0,
        }
    }
    fn profile_action(&mut self, ui: &AppWindow) {
        let Some(item) = self.item().cloned() else {
            return;
        };
        if matches!(self.sheet, Some(Sheet::ProfileNames)) {
            let names = services::profile_names();
            let index = self.focus - 910;
            let name = if (0..6).contains(&index) {
                names[index as usize].to_owned()
            } else if index == 6 {
                self.free_profile_name(&item)
            } else {
                return;
            };
            self.sheet = None;
            self.focus = self.invoking;
            self.start(ui, Command::RenameProfile(item.profile.id, name));
            return;
        }
        match self.focus {
            910 | 911 => {
                let index = item
                    .profiles
                    .iter()
                    .position(|p| p.id == item.profile.id)
                    .unwrap_or(0);
                let next = if self.focus == 910 {
                    index.checked_sub(1)
                } else {
                    (index + 1 < item.profiles.len()).then_some(index + 1)
                };
                if let Some(next) = next {
                    self.start(
                        ui,
                        Command::SelectProfile(item.recipe.id, item.profiles[next].id.clone()),
                    );
                }
            }
            912 | 913 => {
                let copy = self.focus == 912;
                self.sheet = None;
                self.focus = self.invoking;
                self.start(ui, Command::NewProfile(item.recipe.id, copy));
            }
            914 => {
                self.sheet = Some(Sheet::ProfileNames);
                self.focus = 910;
            }
            915 if item.profiles.len() > 1 => {
                self.sheet = Some(Sheet::DeleteProfile(item.profile.id, item.profile.name));
                self.focus = 900;
            }
            916 => {
                self.sheet = None;
                self.focus = self.invoking;
            }
            _ => {}
        }
    }
    fn free_profile_name(&self, item: &Item) -> String {
        let n = (2..)
            .find(|n| {
                !item
                    .profiles
                    .iter()
                    .any(|p| p.name == format!("Profile {n}"))
            })
            .unwrap();
        format!("Profile {n}")
    }
    fn profile_sheet(&self, ui: &AppWindow) {
        let Some(item) = self.item() else {
            return;
        };
        let mut b = Nodes::new(ui);
        if matches!(self.sheet, Some(Sheet::ProfileNames)) {
            ui.set_sheet_title(text(ui, T::RenameProfile));
            let keys = [
                T::PresetDefault,
                T::PresetCasual,
                T::PresetUv,
                T::PresetPistol,
                T::PresetCoop,
                T::PresetSpeedrun,
            ];
            for (i, key) in keys.into_iter().enumerate() {
                b.button(
                    910 + i as i32,
                    32. + (i % 2) as f32 * 336.,
                    136. + (i / 2) as f32 * 88.,
                    304.,
                    text(ui, key),
                    false,
                );
            }
            let name = self.free_profile_name(item);
            b.button(
                916,
                32.,
                400.,
                304.,
                value(ui, T::PresetNumber, name.trim_start_matches("Profile ")),
                false,
            );
        } else {
            ui.set_sheet_title(value(ui, T::ActiveProfile, &item.profile.name));
            b.label(
                32.,
                96.,
                656.,
                32.,
                profile_count(ui, item.profiles.len()),
                22.,
            );
            let index = item
                .profiles
                .iter()
                .position(|p| p.id == item.profile.id)
                .unwrap_or(0);
            b.button(910, 32., 152., 304., text(ui, T::PreviousProfile), false);
            b.nodes.last_mut().unwrap().actionable = index > 0;
            b.button(911, 368., 152., 304., text(ui, T::NextProfile), false);
            b.nodes.last_mut().unwrap().actionable = index + 1 < item.profiles.len();
            b.button(912, 32., 240., 304., text(ui, T::CopyProfile), false);
            b.button(913, 368., 240., 304., text(ui, T::DefaultProfile), false);
            b.button(914, 32., 328., 304., text(ui, T::RenameProfile), false);
            b.button(
                915,
                368.,
                328.,
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
            b.button(916, 32., 432., 656., text(ui, T::Cancel), false);
        }
        ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
    }
    fn move_focus(&mut self, direction: Action) {
        if self.focus < 0 && self.root() {
            match direction {
                Action::Left if self.focus < -1 => self.focus += 1,
                Action::Right if self.focus > -4 => self.focus -= 1,
                Action::Down => {
                    let tab = (-self.focus - 1) as usize;
                    self.tab(tab);
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
        let (cx, cy) = (current.x + current.w / 2., current.y + current.h / 2.);
        if let Some(next) = self
            .nodes
            .iter()
            .filter(|n| n.actionable && n.id != self.focus)
            .filter_map(|n| {
                let (dx, dy) = (n.x + n.w / 2. - cx, n.y + n.h / 2. - cy);
                let (along, across) = match direction {
                    Action::Up => (-dy, dx.abs()),
                    Action::Down => (dy, dx.abs()),
                    Action::Left => (-dx, dy.abs()),
                    Action::Right => (dx, dy.abs()),
                    _ => return None,
                };
                (along > 1.).then_some((n.id, along + across * 3.))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
        {
            self.focus = next.0;
        } else if direction == Action::Up && self.root() {
            self.remembered[self.tab_index()] = self.focus;
            self.focus = -(self.tab_index() as i32) - 1;
        }
    }
    pub fn sync(&mut self, ui: &AppWindow) {
        if !self.categories().contains(&self.category) {
            self.category = Category::All;
        }
        if self.screen == Screen::Settings && (1..=4).contains(&self.focus) {
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
        ui.set_subtitle(text(
            ui,
            if self.playing {
                T::PlayingSub
            } else if self.screen == Screen::FirstRun {
                T::FirstSub
            } else if self.busy {
                T::Working
            } else {
                match self.screen {
                    Screen::FirstRun => T::FirstHelp,
                    Screen::Home => {
                        if self
                            .snapshot
                            .as_ref()
                            .is_some_and(|s| s.items.iter().any(|i| i.last_played.is_some()))
                        {
                            T::HomeSub
                        } else {
                            T::HomeIntro
                        }
                    }
                    Screen::Review | Screen::Browser => T::ScanHelp,
                    Screen::Setup => T::SetupSub,
                    Screen::Detail => T::DetailSub,
                    Screen::Discover => T::DiscoverSub,
                    Screen::Library => T::LibrarySub,
                    Screen::Progress => T::InstallTitle,
                    Screen::Settings => T::SettingsSub,
                    Screen::Loading => T::Loading,
                }
            },
        ));
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
                        b.button(1, 36., 220., 536., text(ui, T::InstallEngine), true);
                    }
                    b.label(652., 36., 536., 44., text(ui, T::BaseGames), 30.);
                    b.label(652., 96., 536., 84., text(ui, T::ScanHelp), 22.);
                    b.button(2, 652., 208., 536., text(ui, T::AddFolder), ready);
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
                        b.art(36., 36., 240., 300., art(&item.recipe.id));
                        b.label(
                            308.,
                            56.,
                            448.,
                            60.,
                            display_title(&item.recipe).into(),
                            36.,
                        );
                        b.label(
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
                        b.label(36., 42., 720., 100., text(ui, T::HomeEmpty), 36.);
                        b.label(36., 160., 720., 80., text(ui, T::NoHistory), 22.);
                        b.button(1, 36., 252., 720., text(ui, T::DiscoverAction), true);
                    }
                    b.label(36., 356., 720., 32., text(ui, T::Recent), 26.);
                    if let Some(s) = &self.snapshot {
                        let mut recent: Vec<_> =
                            s.items.iter().filter(|i| i.last_played.is_some()).collect();
                        recent.sort_by_key(|i| std::cmp::Reverse(i.last_played));
                        if recent.is_empty() {
                            b.label(36., 402., 720., 54., text(ui, T::NoRecent), 22.);
                        }
                        for (n, i) in recent.into_iter().take(2).enumerate() {
                            let x = 36. + n as f32 * 360.;
                            b.art(x, 400., 56., 66., art(&i.recipe.id));
                            b.label(
                                x + 72.,
                                400.,
                                270.,
                                32.,
                                display_title(&i.recipe).into(),
                                26.,
                            );
                            b.label(x + 72., 436., 270., 30., i.profile.name.clone().into(), 20.);
                        }
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
                        b.check(852., 316., text(ui, T::Verification), 3);
                    }
                    b.button(2, 852., 402., 336., text(ui, T::DiscoverAction), false);
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
                            let item = &s.items[i];
                            let x = 12. + 408. * (visible % 3) as f32;
                            let y = 12. + top + 438. * (visible / 3) as f32;
                            b.card(10 + i as i32, x, y, 384., 410.);
                            b.art(x + 16., y + 16., 352., 142., art(&item.recipe.id));
                            if item.favorite {
                                b.favorite(x + 316., y + 26.);
                            }
                            b.label(
                                x + 20.,
                                y + 174.,
                                344.,
                                42.,
                                display_title(&item.recipe).into(),
                                26.,
                            );
                            b.label(
                                x + 20.,
                                y + 218.,
                                344.,
                                50.,
                                authors_line(&item.recipe, 60),
                                20.,
                            );
                            b.label(
                                x + 20.,
                                y + 270.,
                                344.,
                                28.,
                                game(ui, item.recipe.base_game.game),
                                22.,
                            );
                            b.label(
                                x + 20.,
                                y + 300.,
                                344.,
                                28.,
                                text(
                                    ui,
                                    if base_ready(s, &item.recipe) {
                                        T::BaseReady
                                    } else {
                                        T::BaseMissingShort
                                    },
                                ),
                                20.,
                            );
                            b.label(
                                x + 20.,
                                y + 330.,
                                344.,
                                28.,
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
                                x + 20.,
                                y + 366.,
                                344.,
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
                            b.button(SHOW_ALL, 36., 204. + top, 400., text(ui, T::ShowAll), true);
                        } else if shown.is_empty() {
                            b.panel(12., 12., 1200., 300.);
                            b.label(36., 60., 1100., 70., text(ui, T::NoInstalled), 30.);
                            b.button(10, 36., 180., 400., text(ui, T::DiscoverAction), true);
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
                        b.art(12., 12., 360., 262., art(&item.recipe.id));
                        if item.favorite {
                            b.favorite(316., 22.);
                        }
                        b.button(
                            1,
                            12.,
                            298.,
                            360.,
                            text(
                                ui,
                                if item.manifest.is_some() {
                                    T::Setup
                                } else {
                                    T::Install
                                },
                            ),
                            true,
                        );
                        b.button(2, 12., 366., 360., text(ui, T::Setup), false);
                        b.button(
                            3,
                            12.,
                            434.,
                            360.,
                            text(
                                ui,
                                if item.favorite {
                                    T::FavoriteRemoveButton
                                } else {
                                    T::FavoriteAddButton
                                },
                            ),
                            false,
                        );
                        b.button(
                            4,
                            12.,
                            502.,
                            360.,
                            value(ui, T::ActiveProfile, &item.profile.name),
                            false,
                        );
                        b.panel(404., 12., 808., 478.);
                        b.label(
                            428.,
                            36.,
                            760.,
                            60.,
                            display_title(&item.recipe).into(),
                            40.,
                        );
                        b.label(428., 100., 760., 32., authors_line(&item.recipe, 66), 22.);
                        b.label(
                            428.,
                            138.,
                            760.,
                            96.,
                            item.recipe.description.clone().into(),
                            22.,
                        );
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
                            760.,
                            36.,
                            game(ui, item.recipe.base_game.game),
                            26.,
                        );
                        b.label(
                            428.,
                            y + 44.,
                            760.,
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
                        b.label(428., y + 104., 760., 36., text(ui, T::Verification), 20.);
                        b.label(
                            428.,
                            y + 148.,
                            760.,
                            36.,
                            item.recipe.page.clone().into(),
                            20.,
                        );
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
                            let detail = if !c.optional {
                                text(ui, T::Locked)
                            } else if c.user_supplied {
                                text(ui, T::NotProvided)
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
                                c.optional,
                            );
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
                                value(ui, T::Complevel, summary.complevel),
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
                                b.button(1, 740., 402., 448., text(ui, T::Reset), true);
                                if self.focus == 50 {
                                    self.focus = 1;
                                }
                                ui.set_subtitle(text(ui, T::ErrorConfig));
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
                                b.button(2, 740., 282., 448., text(ui, T::InstallMore), true);
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
                                    true,
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
                        Some(Command::Install(id)) => self
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
                    b.label(36., 36., 1100., 60., subject, 36.);
                    let stage = self.progress.map_or(T::Working, |p| match p.stage {
                        InstallStage::Download => T::Download,
                        InstallStage::Verify => T::Verify,
                        InstallStage::Extract => T::Extract,
                        InstallStage::Commit => T::Commit,
                    });
                    let stage_help = match self.progress.map(|p| p.stage) {
                        Some(InstallStage::Download) => T::DownloadHelp,
                        Some(InstallStage::Verify) => T::VerifyHelp,
                        Some(InstallStage::Extract) => T::ExtractHelp,
                        Some(InstallStage::Commit) => T::CommitHelp,
                        None => T::Working,
                    };
                    ui.set_subtitle(text(
                        ui,
                        if self.problem.is_some() {
                            T::InstallErrorSub
                        } else if self.cancel.is_cancelled() {
                            T::CancelPending
                        } else {
                            stage_help
                        },
                    ));
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
                    b.nodes.last_mut().unwrap().art = self
                        .progress
                        .filter(|p| p.stage == InstallStage::Download)
                        .and_then(|p| p.transfer)
                        .and_then(|p| {
                            p.total
                                .filter(|t| *t > 0)
                                .map(|t| (p.downloaded.saturating_mul(1000) / t).min(1000) as i32)
                        })
                        .unwrap_or(-1);
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
                        b.button(1, 836., 390., 352., text(ui, T::UseGame), true);
                    }
                }
                Screen::Browser => {
                    if let Some(d) = &self.directory {
                        ui.set_subtitle(elide_path_start(&d.path.display().to_string(), 88).into());
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
                    b.panel(12., 12., 312., 360.);
                    let height = match self.settings_section {
                        1 => self
                            .snapshot
                            .as_ref()
                            .map_or(316., |s| (316. + s.folders.len() as f32 * 112.).max(316.)),
                        2 => 280.,
                        // 478 like the Engine section so the spec 017 R6 widescreen
                        // status-bar credit under the version block stays inside
                        // the panel instead of spilling past its lower border.
                        3 => 478.,
                        _ => 478.,
                    };
                    b.panel(348., 12., 864., height);
                    for (i, label) in [T::Engine, T::Folders, T::Config, T::Diagnostics]
                        .iter()
                        .enumerate()
                    {
                        b.button(
                            1 + i as i32,
                            28.,
                            36. + i as f32 * 80.,
                            280.,
                            text(ui, *label),
                            i == self.settings_section,
                        );
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
                                b.button(70, 372., 240., 380., text(ui, T::CheckUpdates), true);
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
                            }
                            1 => {
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
                                b.button(70, 372., 128., 380., text(ui, T::AddFolder), true);
                                b.button(71, 776., 128., 412., text(ui, T::Rescan), false);
                                if s.folders.is_empty() {
                                    b.label(372., 220., 816., 84., text(ui, T::NoFolders), 22.);
                                }
                                for (i, p) in s.folders.iter().enumerate() {
                                    b.row(
                                        100 + i as i32,
                                        372.,
                                        220. + i as f32 * 112.,
                                        816.,
                                        text(ui, T::Remove),
                                        p.display().to_string().into(),
                                        true,
                                    );
                                }
                            }
                            2 => {
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
                                b.button(70, 372., 200., 816., text(ui, T::Reset), true);
                            }
                            _ => {
                                b.label(372., 36., 816., 44., text(ui, T::Logs), 30.);
                                b.label(
                                    372.,
                                    96.,
                                    816.,
                                    100.,
                                    s.logs.display().to_string().into(),
                                    22.,
                                );
                                b.logo(372., 220., 560., 128., false);
                                b.label(960., 252., 228., 44., text(ui, T::AppVersion), 26.);
                                b.label(960., 314., 228., 44., crate::cli::VERSION.into(), 22.);
                                // Spec 017, R6: credit the widescreen status bars.
                                b.label(372., 372., 816., 60., text(ui, T::StatusBarCredit), 20.);
                            }
                        }
                    }
                }
            }
        }
        if let Some(problem) = &self.problem {
            b.panel(12., 510., 1200., 222.);
            b.label(36., 530., 1152., 88., problem_text(ui, problem), 22.);
            b.button(800, 36., 638., 552., text(ui, T::Retry), true);
            b.button(801, 612., 638., 576., text(ui, T::Diagnostics), false);
        }
        self.nodes = b.nodes;
        if !self.playing
            && self.focus >= 0
            && self.sheet.is_none()
            && !self
                .nodes
                .iter()
                .any(|n| n.id == self.focus && n.actionable)
        {
            self.focus = self.nodes.iter().find(|n| n.actionable).map_or(0, |n| n.id);
        }
        let setup = self.screen == Screen::Setup;
        let viewport = (ui.window().size().height as f32 - 294.).max(64.);
        let offset = self
            .nodes
            .iter()
            .find(|n| n.id == self.focus && (!setup || n.x < 700.))
            .map_or(0., |n| (n.y + n.h + 12. - viewport).max(0.));
        ui.set_scroll_offset(offset);
        ui.set_nodes(ModelRc::new(VecModel::from(self.nodes.clone())));
        ui.set_focused(self.focus);
        ui.set_active_tab(self.tab_index() as i32);
        ui.set_root_screen(self.root() && !self.playing);
        ui.set_input_locked(self.playing);
        ui.set_setup_screen(setup && !self.playing);
        ui.set_a_hint(
            self.nodes
                .iter()
                .find(|n| n.id == self.focus && !n.label.is_empty())
                .map_or_else(|| text(ui, T::Open), |n| n.label.clone()),
        );
        let card_focus = matches!(self.screen, Screen::Discover | Screen::Library)
            && (10..=99).contains(&self.focus);
        let marked = if card_focus {
            self.snapshot
                .as_ref()
                .and_then(|s| s.items.get((self.focus - 10) as usize))
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
        ui.set_category_hint(self.screen == Screen::Discover && !self.playing);
        ui.set_detail_hint(false);
        ui.set_import_hint(false);
        ui.set_sheet_open(self.sheet.is_some());
        ui.set_sheet_height(
            if matches!(self.sheet, Some(Sheet::Profiles | Sheet::ProfileNames)) {
                560.
            } else if matches!(self.sheet, Some(Sheet::DeleteProfile(..))) {
                340.
            } else {
                300.
            },
        );
        if matches!(self.sheet, Some(Sheet::Profiles | Sheet::ProfileNames)) {
            self.profile_sheet(ui);
        } else if let Some(sheet) = &self.sheet {
            let (title, safe, confirm) = match sheet {
                Sheet::Quit => (T::QuitConfirm, T::KeepPlaying, T::Quit),
                Sheet::Cancel => (T::CancelConfirm, T::KeepDownloading, T::CancelInstall),
                Sheet::Reset => (T::ResetConfirm, T::Cancel, T::Reset),
                Sheet::Remove(_) => (T::RemoveConfirm, T::Cancel, T::Remove),
                Sheet::Component(_) => (T::CompatConfirm, T::Keep, T::UseVariant),
                Sheet::DeleteProfile(..) => (T::DeleteProfileConfirm, T::Cancel, T::DeleteProfile),
                Sheet::Profiles | Sheet::ProfileNames => unreachable!(),
            };
            ui.set_sheet_title(if let Sheet::DeleteProfile(_, name) = sheet {
                value(ui, title, name)
            } else {
                text(ui, title)
            });
            let mut b = Nodes::new(ui);
            let button_y = if matches!(sheet, Sheet::DeleteProfile(..)) {
                228.
            } else {
                180.
            };
            b.button(900, 32., button_y, 304., text(ui, safe), false);
            b.button(901, 368., button_y, 304., text(ui, confirm), false);
            ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
        }
        if self.sheet.is_some() {
            let nodes: Vec<_> = ui.get_sheet_nodes().iter().collect();
            if !nodes.iter().any(|n| n.id == self.focus && n.actionable) {
                self.focus = nodes.iter().find(|n| n.actionable).map_or(900, |n| n.id);
            }
            ui.set_focused(self.focus);
            if let Some(n) = nodes.iter().find(|n| n.id == self.focus) {
                ui.set_a_hint(n.label.clone());
            }
        }
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
/// The credited authors on one line: whole names up to `max` characters, then `…`.
fn authors_line(recipe: &ddl_core::catalog::Recipe, max: usize) -> SharedString {
    let mut line = String::new();
    for (n, author) in recipe.authors.iter().enumerate() {
        let next = if n == 0 {
            author.clone()
        } else {
            format!("{line}, {author}")
        };
        if next.chars().count() > max && n > 0 {
            line.push('…');
            return line.into();
        }
        line = next;
    }
    line.into()
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
    }
}
fn problem_copy(p: &Problem) -> T {
    match p {
        Problem::Storage => T::ErrorStorage,
        Problem::State => T::ErrorState,
        Problem::Network => T::ErrorNetwork,
        Problem::Update => T::UpdateError,
        Problem::Checksum => T::ErrorChecksum,
        Problem::UnsafeArchive => T::ErrorArchive,
        Problem::Cancelled => T::ErrorCancelled,
        Problem::Engine => T::ErrorEngine,
        Problem::Config => T::ErrorConfig,
        Problem::UnknownFile => T::UnknownFile,
        Problem::Launch => T::ErrorLaunch,
        Problem::Busy => T::ErrorBusy,
        Problem::EngineRuntime(_) => T::ErrorEngineRuntime,
    }
}

/// The problem panel's label: a static key, or the engine runtime's own
/// actionable message (spec 014 follow-up).
fn problem_text(ui: &AppWindow, problem: &Problem) -> slint::SharedString {
    match problem_detail(problem) {
        Some(detail) => value(ui, T::ErrorEngineRuntime, detail),
        None => text(ui, problem_copy(problem)),
    }
}

/// The runtime detail a problem carries, when it has one.
fn problem_detail(problem: &Problem) -> Option<String> {
    match problem {
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
    fn label(&mut self, x: f32, y: f32, w: f32, h: f32, label: SharedString, size: f32) {
        self.push(-999, x, y, w, h, 0, label, Default::default(), false, false);
        self.nodes.last_mut().unwrap().size = size;
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
    fn choice(&mut self, id: i32, x: f32, y: f32, w: f32, label: SharedString, selected: bool) {
        self.push(
            id,
            x,
            y,
            w,
            52.,
            8,
            label,
            Default::default(),
            true,
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
pub fn attach(ui: &AppWindow) -> Result<Rc<RefCell<Runtime>>, Box<dyn std::error::Error>> {
    let paths = ddl_core::platform::paths::resolve(&|key: &str| std::env::var(key).ok())?;
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is missing")?);
    let env = Environment {
        paths,
        home,
        #[cfg(test)]
        test: None,
    };
    Ok(attach_environment(ui, env))
}
pub fn attach_environment(ui: &AppWindow, env: Environment) -> Rc<RefCell<Runtime>> {
    let state = Rc::new(RefCell::new(Runtime::new(env)));
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
    state.borrow_mut().start(ui, Command::Startup);
    state.borrow_mut().sync(ui);
    state
}

fn display_title(recipe: &ddl_core::catalog::Recipe) -> &str {
    recipe.title.split(" (").next().unwrap_or(&recipe.title)
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
            tags: catalog.tags.clone(),
            engine: None,
            previous: None,
            attribution: None,
            iwads: vec![],
            folders: vec![],
            config_notice: false,
            config_usable: true,
            config_version: String::new(),
            logs: PathBuf::new(),
            search_standard_folders: false,
        };
        assert_eq!(
            tag_labels(&s, recipe, 2, None),
            ["Classics", "Cacowards"],
            "vocabulary order"
        );
        assert_eq!(
            tag_labels(&s, recipe, 2, Some("megawad")),
            ["Megawads", "Classics"]
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
    #[test]
    fn author_lines_elide_whole_names() {
        let catalog = Catalog::embedded();
        let eviternity = catalog.recipe("eviternity").unwrap();
        let line = authors_line(eviternity, 66);
        assert!(line.ends_with('…') && line.chars().count() <= 67, "{line}");
        assert!(line.starts_with("Joshua"));
        let sigil = catalog.recipe("sigil").unwrap();
        assert_eq!(authors_line(sigil, 66), "John Romero");
    }
}
