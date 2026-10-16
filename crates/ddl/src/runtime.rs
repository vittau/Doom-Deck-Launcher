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
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
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
#[derive(Clone)]
enum Sheet {
    Quit,
    Cancel,
    Reset,
    Remove(PathBuf),
    Component(String),
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
        self.sheet = Some(sheet);
        self.focus = 900;
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
            Action::Up | Action::Down | Action::Left | Action::Right => self.move_focus(action),
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
            if id == 900 || id == 901 {
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
                        .filter(|(_, i)| i.profile.last_played.is_some())
                        .max_by_key(|(_, i)| i.profile.last_played)
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
            (Screen::Detail, 2) => self.enter(Screen::Setup, 50),
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
                    if c.user_supplied {
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
    fn tab_index(&self) -> usize {
        match self.screen {
            Screen::Discover => 1,
            Screen::Library => 2,
            Screen::Settings => 3,
            _ => 0,
        }
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
                .and_then(|s| s.items.iter().max_by_key(|i| i.profile.last_played))
        } else {
            self.item()
        };
        ui.set_band_art(band_item.map_or(0, |i| art(&i.recipe.id)));
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
                        if self.snapshot.as_ref().is_some_and(|s| {
                            s.items.iter().any(|i| i.profile.last_played.is_some())
                        }) {
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
                    b.label(40., 60., 1100., 100., text(ui, T::Loading), 30.);
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
                            .filter(|i| i.profile.last_played.is_some())
                            .max_by_key(|i| i.profile.last_played)
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
                                text(ui, T::Profile)
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
                        let mut recent: Vec<_> = s
                            .items
                            .iter()
                            .filter(|i| i.profile.last_played.is_some())
                            .collect();
                        recent.sort_by_key(|i| std::cmp::Reverse(i.profile.last_played));
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
                            b.label(x + 72., 436., 270., 30., text(ui, T::Profile), 20.);
                        }
                        b.label(852., 36., 336., 48., text(ui, T::ReadyChecks), 30.);
                        b.label(
                            852.,
                            104.,
                            336.,
                            54.,
                            s.engine.as_ref().map_or_else(
                                || text(ui, T::NoEngine),
                                |e| value(ui, T::EngineVersion, &e.version),
                            ),
                            22.,
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
                            b.label(
                                852.,
                                174. + n as f32 * 44.,
                                336.,
                                40.,
                                value(
                                    ui,
                                    if found { T::FoundGame } else { T::MissingGame },
                                    game(ui, g),
                                ),
                                22.,
                            );
                        }
                        b.label(852., 274., 336., 68., text(ui, T::Verification), 20.);
                    }
                    b.button(2, 852., 386., 336., text(ui, T::DiscoverAction), false);
                }
                Screen::Discover | Screen::Library => {
                    if let Some(s) = &self.snapshot {
                        let mut visible = 0;
                        for (i, item) in s.items.iter().enumerate() {
                            if self.screen == Screen::Library && item.manifest.is_none() {
                                continue;
                            }
                            let x = 12. + 408. * (visible % 3) as f32;
                            let y = 12. + 438. * (visible / 3) as f32;
                            b.card(10 + i as i32, x, y, 384., 410.);
                            b.art(x + 16., y + 16., 352., 142., art(&item.recipe.id));
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
                                y + 222.,
                                344.,
                                54.,
                                item.recipe.authors.join(", ").into(),
                                20.,
                            );
                            b.label(
                                x + 20.,
                                y + 284.,
                                344.,
                                30.,
                                game(ui, item.recipe.base_game.game),
                                22.,
                            );
                            b.label(
                                x + 20.,
                                y + 320.,
                                344.,
                                36.,
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
                                y + 366.,
                                344.,
                                30.,
                                text(
                                    ui,
                                    if item.manifest.is_some() {
                                        T::Installed
                                    } else {
                                        T::NotInstalled
                                    },
                                ),
                                20.,
                            );
                            visible += 1;
                        }
                        if visible == 0 {
                            b.panel(12., 12., 1200., 300.);
                            b.label(36., 60., 1100., 70., text(ui, T::NoInstalled), 30.);
                            b.button(10, 36., 180., 400., text(ui, T::DiscoverAction), true);
                        }
                    }
                }
                Screen::Detail => {
                    if let Some(item) = self.item() {
                        b.art(12., 12., 360., 318., art(&item.recipe.id));
                        b.button(
                            1,
                            12.,
                            362.,
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
                        b.button(2, 12., 438., 360., text(ui, T::Setup), false);
                        b.panel(404., 12., 808., 478.);
                        b.label(
                            428.,
                            36.,
                            760.,
                            60.,
                            display_title(&item.recipe).into(),
                            40.,
                        );
                        b.label(
                            428.,
                            110.,
                            760.,
                            36.,
                            item.recipe.authors.join(", ").into(),
                            22.,
                        );
                        b.label(
                            428.,
                            164.,
                            760.,
                            100.,
                            item.recipe.description.clone().into(),
                            22.,
                        );
                        b.label(
                            428.,
                            278.,
                            760.,
                            36.,
                            game(ui, item.recipe.base_game.game),
                            26.,
                        );
                        b.label(
                            428.,
                            326.,
                            760.,
                            54.,
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
                        b.label(428., 392., 760., 36., text(ui, T::Verification), 20.);
                        b.label(428., 438., 760., 36., item.recipe.page.clone().into(), 20.);
                    }
                }
                Screen::Setup => {
                    if let Some(item) = self.item() {
                        b.panel(12., 12., 680., 478.);
                        b.panel(716., 12., 496., 478.);
                        b.row(
                            -1,
                            36.,
                            36.,
                            632.,
                            text(ui, T::Profile),
                            text(ui, T::ProfileHelp),
                            false,
                        );
                        let mut y = 132.;
                        for (index, c) in item.recipe.components.iter().enumerate() {
                            if c.replaces.is_some() && !c.user_supplied && !self.advanced {
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
                        b.label(740., 28., 448., 40., text(ui, T::WillLaunch), 26.);
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
                    b.label(36., 40., 1100., 60., subject, 36.);
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
                            b.label(36., 24., 1152., 54., text(ui, T::Ambiguous), 30.);
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
                                24.,
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
                                b.row(
                                    -1,
                                    36.,
                                    100. + i as f32 * 112.,
                                    1152.,
                                    e.variant.clone().unwrap_or_default().into(),
                                    e.path.display().to_string().into(),
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
                        b.label(36., 28., 312., 38., text(ui, T::Shortcuts), 26.);
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
                        1 => self.snapshot.as_ref().map_or(240., |s| {
                            (144. + s.folders.len().max(1) as f32 * 112.).max(240.)
                        }),
                        2 => 280.,
                        3 => 382.,
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
                                b.button(70, 372., 36., 380., text(ui, T::AddFolder), true);
                                b.button(71, 776., 36., 412., text(ui, T::Rescan), false);
                                if s.folders.is_empty() {
                                    b.label(372., 128., 816., 84., text(ui, T::NoFolders), 22.);
                                }
                                for (i, p) in s.folders.iter().enumerate() {
                                    b.row(
                                        100 + i as i32,
                                        372.,
                                        128. + i as f32 * 112.,
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
                                b.label(372., 252., 816., 44., text(ui, T::AppVersion), 26.);
                                b.label(372., 314., 816., 44., crate::cli::VERSION.into(), 22.);
                            }
                        }
                    }
                }
            }
        }
        if let Some(problem) = self.problem {
            b.panel(12., 510., 1200., 222.);
            b.label(36., 530., 1152., 88., text(ui, problem_copy(problem)), 22.);
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
        ui.set_favorite_hint(false);
        ui.set_detail_hint(false);
        ui.set_import_hint(false);
        ui.set_sheet_open(self.sheet.is_some());
        if let Some(sheet) = &self.sheet {
            let (title, safe, confirm) = match sheet {
                Sheet::Quit => (T::QuitConfirm, T::KeepPlaying, T::Quit),
                Sheet::Cancel => (T::CancelConfirm, T::KeepDownloading, T::CancelInstall),
                Sheet::Reset => (T::ResetConfirm, T::Cancel, T::Reset),
                Sheet::Remove(_) => (T::RemoveConfirm, T::Cancel, T::Remove),
                Sheet::Component(_) => (T::CompatConfirm, T::Keep, T::UseVariant),
            };
            ui.set_sheet_title(text(ui, title));
            let mut b = Nodes::new(ui);
            b.button(900, 32., 180., 304., text(ui, safe), false);
            b.button(901, 368., 180., 304., text(ui, confirm), false);
            ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
        }
    }
}
fn art(id: &str) -> i32 {
    match id {
        "sigil" => 0,
        "auger-zenith" => 2,
        "sigil-ii" => 1,
        _ => 0,
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
fn problem_copy(p: Problem) -> T {
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
    fn byte_units_and_unicode_breadcrumbs_are_human_readable() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1023), "1023 B");
        assert_eq!(human_bytes(1536), "1.5 KiB");
        assert_eq!(human_bytes(2 * 1024 * 1024), "2.0 MiB");
        assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.0 GiB");
        assert_eq!(elide_path_start("/short", 10), "/short");
        assert_eq!(elide_path_start("/long/日本語/maps", 9), "…日本語/maps");
    }
}
