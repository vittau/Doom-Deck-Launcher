//! Fixture shell state and spatial navigation, independent of device input.
use crate::{
    AppWindow, ViewNode,
    copy::{Copy, text},
    fixtures::{self, MapsetId, ProfileView},
    input::Action,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Home,
    Discover,
    Library,
    Settings,
    Detail,
    Setup,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sheet {
    Quit,
    Compat,
}
#[derive(Clone, Copy)]
struct Place {
    screen: Screen,
    focus: i32,
}
pub struct Shell {
    pub screen: Screen,
    pub focus: i32,
    pub mapset: MapsetId,
    pub profiles: [ProfileView; 3],
    tab: usize,
    remembered: [i32; 4],
    stack: Vec<Place>,
    sheet: Option<Sheet>,
    invoking: i32,
    nodes: Vec<ViewNode>,
    favorites: [bool; 3],
    category: usize,
    filter: usize,
    section: usize,
}
impl Default for Shell {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            focus: 1,
            mapset: MapsetId::Sigil,
            profiles: Default::default(),
            tab: 0,
            remembered: [1, 10, 20, 60],
            stack: vec![],
            sheet: None,
            invoking: 1,
            nodes: vec![],
            favorites: [false; 3],
            category: 0,
            filter: 0,
            section: 0,
        }
    }
}
impl Shell {
    fn index(&self) -> usize {
        match self.mapset {
            MapsetId::Sigil => 0,
            MapsetId::Auger => 1,
            MapsetId::Sigil2 => 2,
        }
    }
    pub fn root(&self) -> bool {
        matches!(
            self.screen,
            Screen::Home | Screen::Discover | Screen::Library | Screen::Settings
        )
    }
    fn enter(&mut self, screen: Screen, focus: i32) {
        self.stack.push(Place {
            screen: self.screen,
            focus: self.focus,
        });
        self.screen = screen;
        self.focus = focus;
    }
    fn tab(&mut self, tab: usize) {
        if self.root() && self.sheet.is_none() {
            if self.focus > 0 {
                self.remembered[self.tab] = self.focus;
            }
            self.tab = tab;
            self.screen = [
                Screen::Home,
                Screen::Discover,
                Screen::Library,
                Screen::Settings,
            ][tab];
            self.focus = self.remembered[tab];
            self.stack.clear();
        }
    }
    fn toast(ui: &AppWindow) {
        ui.invoke_show_toast(text(ui, Copy::NotWired));
    }
    pub fn hotplug(&self, ui: &AppWindow, connected: bool) {
        ui.invoke_show_toast(text(
            ui,
            if connected {
                Copy::Connected
            } else {
                Copy::Disconnected
            },
        ));
    }
    pub fn action(&mut self, ui: &AppWindow, action: Action) {
        ui.invoke_show_toast(SharedString::default());
        match action {
            Action::Up | Action::Down | Action::Left | Action::Right => self.move_focus(action),
            Action::Confirm => self.activate(ui),
            Action::Back => {
                if self.sheet.take().is_some() {
                    self.focus = self.invoking;
                } else if self.screen == Screen::Settings && self.focus >= 70 {
                    self.focus = 60 + self.section as i32;
                } else if let Some(place) = self.stack.pop() {
                    self.screen = place.screen;
                    self.focus = place.focus;
                }
            }
            Action::PreviousTab => self.tab((self.tab + 3) % 4),
            Action::NextTab => self.tab((self.tab + 1) % 4),
            Action::Tab(tab) => self.tab(tab),
            Action::Menu => {
                if self.sheet.is_none() {
                    self.invoking = self.focus;
                    self.sheet = Some(Sheet::Quit);
                    self.focus = 100;
                }
            }
            Action::X if self.sheet.is_none() => {
                if self.screen == Screen::Home {
                    self.mapset = MapsetId::Sigil;
                    self.enter(Screen::Detail, 30);
                } else if matches!(
                    self.screen,
                    Screen::Discover | Screen::Library | Screen::Detail
                ) && (self.focus >= 20 || self.screen == Screen::Detail)
                {
                    let i = if self.screen == Screen::Detail {
                        self.index()
                    } else {
                        (self.focus - 20) as usize
                    };
                    if i < 3 {
                        self.favorites[i] = !self.favorites[i];
                    }
                }
            }
            Action::Y if self.screen == Screen::Library && self.sheet.is_none() => Self::toast(ui),
            _ => {}
        }
        self.sync(ui);
    }
    pub fn click(&mut self, ui: &AppWindow, id: i32) {
        let eligible = if self.sheet.is_some() {
            id == 100 || id == 101
        } else {
            self.nodes.iter().any(|n| n.id == id && n.actionable)
        };
        if eligible {
            self.focus = id;
            self.action(ui, Action::Confirm);
        }
    }
    fn activate(&mut self, ui: &AppWindow) {
        if let Some(sheet) = self.sheet.take() {
            if self.focus == 101 {
                match sheet {
                    Sheet::Quit => ui.invoke_quit_requested(),
                    Sheet::Compat => {
                        let i = self.index();
                        self.profiles[i].compat = !self.profiles[i].compat;
                    }
                }
            }
            self.focus = self.invoking;
            return;
        }
        if self.focus < 0 {
            self.tab((-self.focus - 1) as usize);
            return;
        }
        match (self.screen, self.focus) {
            (Screen::Home, 1) => {
                self.mapset = MapsetId::Sigil;
                self.enter(Screen::Setup, 50);
            }
            (Screen::Home, 2 | 4 | 5) => {
                self.mapset = if self.focus == 5 {
                    MapsetId::Auger
                } else {
                    MapsetId::Sigil
                };
                self.enter(Screen::Detail, 30);
            }
            (Screen::Home | Screen::Library, 3) => self.tab(1),
            (Screen::Discover, 10..=14) => self.category = (self.focus - 10) as usize,
            (Screen::Library, 10..=13) => self.filter = (self.focus - 10) as usize,
            (Screen::Discover | Screen::Library, 20..=22) => {
                self.mapset = fixtures::MAPSETS[(self.focus - 20) as usize];
                self.enter(Screen::Detail, 30);
            }
            (Screen::Detail, 30) => {
                if !fixtures::mapset(self.mapset).installed {
                    Self::toast(ui);
                } else {
                    self.enter(Screen::Setup, 50);
                }
            }
            // A separate setup entry keeps the missing-IWAD fixture reachable before install.
            (Screen::Detail, 37) => self.enter(Screen::Setup, 50),
            (Screen::Detail, 32) => Self::toast(ui),
            (Screen::Detail, 36) => {
                ui.invoke_show_toast(fixtures::mapset(self.mapset).source.into())
            }
            (Screen::Detail, 31) => {
                let i = self.index();
                self.favorites[i] = !self.favorites[i];
            }
            (Screen::Setup, 51) => ui.invoke_show_toast(text(ui, Copy::Missing)),
            (Screen::Setup, 41) => ui.invoke_show_toast(text(ui, Copy::Locked)),
            (Screen::Setup, 46) => ui.invoke_show_toast(text(ui, Copy::LockedOrder)),
            (Screen::Setup, 43) => {
                let i = self.index();
                if self.mapset == MapsetId::Auger {
                    self.profiles[i].addon = (self.profiles[i].addon + 1) % 3;
                } else if self.mapset == MapsetId::Sigil {
                    if !self.profiles[i].advanced {
                        self.profiles[i].advanced = true;
                    } else {
                        self.invoking = self.focus;
                        self.sheet = Some(Sheet::Compat);
                        self.focus = 100;
                    }
                } else {
                    Self::toast(ui);
                }
            }
            (Screen::Setup, 44) => {
                let i = self.index();
                self.profiles[i].skill = self.profiles[i].skill % 5 + 1;
            }
            (Screen::Setup, 45) => {
                let i = self.index();
                self.profiles[i].pistol = !self.profiles[i].pistol;
            }
            (Screen::Settings, 60..=65) => {
                self.section = (self.focus - 60) as usize;
                self.focus = 70;
            }
            _ => Self::toast(ui),
        }
    }
    fn move_focus(&mut self, direction: Action) {
        if self.sheet.is_some() {
            match direction {
                Action::Left => self.focus = 100,
                Action::Right => self.focus = 101,
                _ => {}
            }
            return;
        }
        if self.focus < 0 {
            match direction {
                Action::Down => {
                    let tab = (-self.focus - 1) as usize;
                    self.tab(tab);
                    self.focus = self.remembered[tab];
                }
                Action::Left if self.root() && self.focus < -1 => self.focus += 1,
                Action::Right if self.root() && self.focus > -4 => self.focus -= 1,
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
        let next = self
            .nodes
            .iter()
            .filter(|n| n.actionable && n.id != self.focus)
            .filter_map(|n| {
                let (dx, dy) = (n.x + n.w / 2. - cx, n.y + n.h / 2. - cy);
                let (along, across, overlap) = match direction {
                    Action::Up => (
                        -dy,
                        dx.abs(),
                        n.x < current.x + current.w && n.x + n.w > current.x,
                    ),
                    Action::Down => (
                        dy,
                        dx.abs(),
                        n.x < current.x + current.w && n.x + n.w > current.x,
                    ),
                    Action::Left => (
                        -dx,
                        dy.abs(),
                        n.y < current.y + current.h && n.y + n.h > current.y,
                    ),
                    Action::Right => (
                        dx,
                        dy.abs(),
                        n.y < current.y + current.h && n.y + n.h > current.y,
                    ),
                    _ => return None,
                };
                (along > 1.)
                    .then_some((n.id, along + across * 2. + if overlap { 0. } else { 1000. }))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|n| n.0);
        if let Some(id) = next {
            self.focus = id;
        } else if direction == Action::Up && self.root() {
            if self.focus > 0 {
                self.remembered[self.tab] = self.focus;
            }
            self.focus = -(self.tab as i32) - 1;
        }
    }
    pub fn sync(&mut self, ui: &AppWindow) {
        if self.screen == Screen::Settings && (60..=65).contains(&self.focus) {
            self.section = (self.focus - 60) as usize;
        }
        let mut b = Builder { ui, nodes: vec![] };
        let m = fixtures::mapset(self.mapset);
        let p = &self.profiles[self.index()];
        let heading = match self.screen {
            Screen::Home => Copy::HomeTitle,
            Screen::Discover => Copy::Discover,
            Screen::Library => Copy::LibraryTitle,
            Screen::Settings => Copy::Settings,
            Screen::Detail => Copy::Detail,
            Screen::Setup => Copy::Setup,
        };
        let sub = match self.screen {
            Screen::Home => Copy::HomeSub,
            Screen::Discover => Copy::DiscoverSub,
            Screen::Library => Copy::LibrarySub,
            Screen::Settings => Copy::SettingsSub,
            Screen::Detail => Copy::DetailSub,
            Screen::Setup => Copy::SetupSub,
        };
        ui.set_heading(if self.screen == Screen::Setup {
            format!("{} / {}", text(ui, heading), m.title).into()
        } else {
            text(ui, heading)
        });
        ui.set_subtitle(text(ui, sub));
        if self.screen == Screen::Setup && self.focus == 45 {
            ui.set_subtitle(text(ui, Copy::PistolHelp));
        }
        if self.screen == Screen::Setup && self.focus == 46 {
            ui.set_subtitle(text(ui, Copy::LockedOrder));
        }
        match self.screen {
            Screen::Home => {
                b.panel(40., 230., 784., 298.);
                b.art(64., 254., 224., 250., 0);
                b.data(320., 260., 460., 52., "SIGIL", 44., 0);
                b.data(320., 324., 460., 30., "John Romero", 22., 1);
                b.label(320., 380., 460., 40., Copy::DefaultGame, 22., 0);
                b.button(1, 320., 450., 220., Copy::Continue, true);
                b.button(2, 560., 450., 220., Copy::Details, false);
                b.panel(848., 230., 392., 298.);
                b.label(872., 254., 344., 40., Copy::Ready, 26., 0);
                b.label(872., 304., 344., 30., Copy::EngineReady, 22., 2);
                b.label(872., 346., 344., 30., Copy::DoomReady, 22., 2);
                b.label(872., 390., 344., 30., Copy::Pending, 20., 3);
                b.button(3, 872., 450., 344., Copy::DiscoverMapsets, false);
                b.label(40., 554., 800., 40., Copy::Recent, 26., 0);
                b.row(4, 40., 606., 584., Copy::Home, text(ui, Copy::RecentSigil));
                b.nodes.last_mut().unwrap().label = "SIGIL · John Romero".into();
                b.row(5, 656., 606., 584., Copy::Home, text(ui, Copy::RecentAuger));
                b.nodes.last_mut().unwrap().label = "AUGER;ZENITH · DBP 37".into();
            }
            Screen::Discover | Screen::Library => {
                let discover = self.screen == Screen::Discover;
                let labels = if discover {
                    vec![
                        Copy::Featured,
                        Copy::Beginner,
                        Copy::Classics,
                        Copy::Cacowards,
                        Copy::Doomer,
                    ]
                } else {
                    vec![
                        Copy::All,
                        Copy::FilterReady,
                        Copy::NeedsSetup,
                        Copy::Favorites,
                    ]
                };
                let widths = if discover {
                    vec![152., 224., 140., 164., 456.]
                } else {
                    vec![160., 160., 220., 200.]
                };
                let mut x = 40.;
                for (i, key) in labels.into_iter().enumerate() {
                    b.button(
                        10 + i as i32,
                        x,
                        230.,
                        widths[i],
                        key,
                        if discover {
                            i == self.category
                        } else {
                            i == self.filter
                        },
                    );
                    x += widths[i] + 16.;
                }
                for (i, id) in fixtures::MAPSETS.into_iter().enumerate() {
                    let v = fixtures::mapset(id);
                    if !discover {
                        if !v.installed
                            || (self.filter == 2 && v.iwad_ready)
                            || (self.filter == 1 && !v.iwad_ready)
                            || (self.filter == 3 && !self.favorites[i])
                        {
                            continue;
                        }
                        let x = 40. + 616. * i as f32;
                        b.card(20 + i as i32, x, 314., 584., 386.);
                        b.art(x + 24., 338., 156., 200., v.art);
                        b.data(x + 204., 346., 356., 40., v.title, 26., 0);
                        b.data(x + 204., 394., 356., 60., v.author, 20., 1);
                        b.label(x + 204., 464., 356., 40., v.game, 22., 0);
                        b.label(
                            x + 24.,
                            566.,
                            536.,
                            64.,
                            if i == 0 {
                                Copy::RecentSigil
                            } else {
                                Copy::RecentAuger
                            },
                            22.,
                            1,
                        );
                        b.label(x + 24., 646., 536., 40., Copy::Open, 22., 0);
                        continue;
                    }
                    let x = 40. + 408. * i as f32;
                    b.card(20 + i as i32, x, 314., 384., 386.);
                    b.art(x + 16., 330., 352., 154., v.art);
                    b.data(x + 20., 496., 344., 38., v.title, 26., 0);
                    b.data(x + 20., 538., 344., 32., v.author, 20., 1);
                    b.label(x + 20., 576., 344., 32., v.game, 22., 0);
                    b.label(
                        x + 20.,
                        616.,
                        344.,
                        30.,
                        if v.iwad_ready {
                            Copy::BaseReady
                        } else {
                            Copy::BaseMissing
                        },
                        20.,
                        if v.iwad_ready { 2 } else { 4 },
                    );
                    b.label(
                        x + 20.,
                        654.,
                        344.,
                        30.,
                        if v.installed {
                            Copy::Installed
                        } else {
                            Copy::NotInstalled
                        },
                        20.,
                        3,
                    );
                    if self.favorites[i] {
                        b.data(x + 310., 492., 40., 40., "✓", 26., 3);
                    }
                }
                if !discover
                    && !b
                        .nodes
                        .iter()
                        .any(|n| n.actionable && (20..=22).contains(&n.id))
                {
                    b.label(40., 354., 1000., 44., Copy::NoInstalled, 26., 0);
                    b.button(3, 40., 430., 360., Copy::DiscoverMapsets, true);
                }
            }
            Screen::Detail => {
                b.art(40., 230., 360., 330., m.art);
                b.nodes.last_mut().unwrap().full_credit = true;
                b.button(
                    30,
                    40.,
                    592.,
                    if m.installed { 360. } else { 176. },
                    if m.installed {
                        Copy::PlaySetup
                    } else {
                        Copy::Install
                    },
                    true,
                );
                if !m.installed {
                    b.button(37, 224., 592., 176., Copy::PlaySetup, false);
                }
                b.button(32, 40., 660., 360., Copy::Profiles, false);
                b.panel(432., 230., 808., 482.);
                b.data(456., 248., 550., 56., m.title, 44., 0);
                b.data(456., 310., 550., 32., m.author, 22., 1);
                b.data(1050., 310., 160., 32., m.release, 20., 1);
                b.button(
                    31,
                    1010.,
                    248.,
                    206.,
                    if self.favorites[self.index()] {
                        Copy::Favorited
                    } else {
                        Copy::Favorite
                    },
                    false,
                );
                b.label(456., 354., 760., 40., m.description, 26., 0);
                b.label(
                    456.,
                    398.,
                    760.,
                    32.,
                    if self.mapset == MapsetId::Auger {
                        Copy::AddonSummary
                    } else {
                        Copy::Midi
                    },
                    22.,
                    1,
                );
                b.row(
                    33,
                    456.,
                    446.,
                    760.,
                    m.game,
                    text(
                        ui,
                        if m.iwad_ready {
                            Copy::Match
                        } else {
                            Copy::BaseMissing
                        },
                    ),
                );
                b.row(34, 456., 534., 760., Copy::Pending, text(ui, m.complevel));
                b.row(
                    35,
                    456.,
                    622.,
                    360.,
                    Copy::Components,
                    text(
                        ui,
                        if self.mapset == MapsetId::Auger {
                            Copy::AddonSummary
                        } else {
                            Copy::SoundtrackSummary
                        },
                    ),
                );
                b.nodes.last_mut().unwrap().h = 88.;
                let source = if m.source.len() > 34 {
                    format!("{}…{}", &m.source[..16], &m.source[m.source.len() - 16..])
                } else {
                    m.source.into()
                };
                b.row(36, 840., 622., 376., Copy::Source, source.into());
                b.nodes.last_mut().unwrap().h = 88.;
            }
            Screen::Setup => {
                b.panel(40., 230., 680., 490.);
                b.panel(744., 230., 496., 490.);
                b.row(
                    40,
                    64.,
                    254.,
                    632.,
                    Copy::Profile,
                    text(ui, Copy::ProfileSub),
                );
                b.row(41, 64., 342., 632., Copy::Main, text(ui, Copy::Locked));
                // Required row remains inspectable, never toggles off.
                if self.mapset == MapsetId::Auger {
                    b.row(
                        43,
                        64.,
                        430.,
                        632.,
                        [Copy::None, Copy::Alternate, Copy::Deckard][p.addon],
                        text(ui, Copy::AddonHelp),
                    );
                } else {
                    b.row(
                        42,
                        64.,
                        430.,
                        632.,
                        Copy::Soundtrack,
                        text(ui, Copy::SoundtrackHelp),
                    );
                }
                if self.mapset != MapsetId::Auger {
                    b.row(
                        43,
                        64.,
                        518.,
                        632.,
                        if self.mapset == MapsetId::Sigil2 {
                            Copy::Evidence
                        } else if !p.advanced {
                            Copy::Advanced
                        } else if p.compat {
                            Copy::AdvancedOn
                        } else {
                            Copy::AdvancedOff
                        },
                        text(
                            ui,
                            if self.mapset == MapsetId::Sigil2 {
                                Copy::Complevel3
                            } else {
                                Copy::Exclusive
                            },
                        ),
                    );
                }
                let y = if self.mapset == MapsetId::Auger {
                    518.
                } else {
                    606.
                };
                b.button(
                    44,
                    64.,
                    y,
                    304.,
                    [
                        Copy::Skill1,
                        Copy::Skill2,
                        Copy::Skill3,
                        Copy::Skill4,
                        Copy::Skill5,
                    ][p.skill - 1],
                    false,
                );
                b.button(
                    45,
                    384.,
                    y,
                    312.,
                    if p.pistol {
                        Copy::PistolOn
                    } else {
                        Copy::PistolOff
                    },
                    false,
                );
                b.button(
                    46,
                    64.,
                    if self.mapset == MapsetId::Auger {
                        606.
                    } else {
                        676.
                    },
                    632.,
                    Copy::LoadOrder,
                    false,
                );
                b.label(768., 248., 448., 40., Copy::WillLaunch, 26., 0);
                b.row(
                    47,
                    768.,
                    296.,
                    448.,
                    m.game,
                    text(
                        ui,
                        if m.iwad_ready {
                            Copy::Match
                        } else {
                            Copy::BaseMissing
                        },
                    ),
                );
                let files = fixtures::files(self.mapset, p);
                b.label(
                    768.,
                    368.,
                    448.,
                    28.,
                    if files.len() == 1 {
                        Copy::OneFile
                    } else {
                        Copy::TwoFiles
                    },
                    22.,
                    0,
                );
                b.node(
                    49,
                    768.,
                    400.,
                    448.,
                    64.,
                    3,
                    format!("1  {}", files[0]).into(),
                    files
                        .get(1)
                        .map_or_else(SharedString::default, |f| format!("2  {f}").into()),
                    0,
                    20.,
                    false,
                    true,
                );
                b.label(768., 470., 448., 28., Copy::EngineRef, 20., 1);
                b.node(
                    48,
                    768.,
                    504.,
                    448.,
                    68.,
                    2,
                    text(ui, m.complevel),
                    Default::default(),
                    3,
                    20.,
                    false,
                    true,
                );
                ui.global::<crate::Strings>().set_skill(p.skill as i32);
                ui.global::<crate::Strings>().set_pistol(p.pistol);
                b.node(
                    -100,
                    768.,
                    578.,
                    448.,
                    30.,
                    0,
                    ui.global::<crate::Strings>().get_summary(),
                    Default::default(),
                    1,
                    22.,
                    false,
                    false,
                );
                b.button(
                    50,
                    768.,
                    620.,
                    if m.iwad_ready { 448. } else { 280. },
                    if m.iwad_ready {
                        Copy::Play
                    } else {
                        Copy::AddFolder
                    },
                    true,
                );
                if !m.iwad_ready {
                    b.button(51, 1064., 620., 152., Copy::Play, false);
                }
                b.label(
                    768.,
                    680.,
                    448.,
                    28.,
                    if m.iwad_ready {
                        if self.mapset == MapsetId::Auger {
                            Copy::AddonSummary
                        } else {
                            Copy::Optional
                        }
                    } else {
                        Copy::PlayUnavailable
                    },
                    20.,
                    if m.iwad_ready { 1 } else { 4 },
                );
                if !m.iwad_ready {
                    ui.set_subtitle(text(ui, Copy::Missing));
                }
            }
            Screen::Settings => {
                b.panel(40., 230., 312., 482.);
                b.panel(376., 230., 864., 482.);
                for (i, key) in [
                    Copy::Engine,
                    Copy::IwadFolders,
                    Copy::Storage,
                    Copy::Config,
                    Copy::Diagnostics,
                    Copy::Appearance,
                ]
                .into_iter()
                .enumerate()
                {
                    b.button(
                        60 + i as i32,
                        56.,
                        254. + 72. * i as f32,
                        280.,
                        key,
                        self.section == i,
                    );
                }
                let (title, info, first, second, third) = match self.section {
                    0 => (
                        Copy::Engine,
                        Copy::EngineInstalled,
                        Copy::CheckUpdates,
                        Copy::Rollback,
                        Copy::Unlisted,
                    ),
                    1 => (
                        Copy::IwadFolders,
                        Copy::LocalOnly,
                        Copy::AddFolder,
                        Copy::Rescan,
                        Copy::Requirements,
                    ),
                    2 => (
                        Copy::Storage,
                        Copy::LocalOnly,
                        Copy::ManageStorage,
                        Copy::ClearCache,
                        Copy::Details,
                    ),
                    3 => (
                        Copy::Config,
                        Copy::Backup,
                        Copy::ResetConfig,
                        Copy::Details,
                        Copy::Diagnostics,
                    ),
                    4 => (
                        Copy::Diagnostics,
                        Copy::LocalOnly,
                        Copy::ViewLogs,
                        Copy::Export,
                        Copy::LaunchDetails,
                    ),
                    _ => (
                        Copy::Appearance,
                        Copy::English,
                        Copy::Motion,
                        Copy::Details,
                        Copy::Diagnostics,
                    ),
                };
                b.label(400., 254., 816., 44., title, 30., 0);
                b.label(400., 310., 816., 40., info, 22., 0);
                b.label(400., 354., 816., 40., Copy::Trust, 22., 1);
                b.button(70, 400., 410., 380., first, true);
                b.row(
                    71,
                    400.,
                    492.,
                    816.,
                    second,
                    text(
                        ui,
                        if self.section == 0 {
                            Copy::RollbackSub
                        } else {
                            Copy::LocalOnly
                        },
                    ),
                );
                b.row(
                    72,
                    400.,
                    580.,
                    816.,
                    third,
                    text(
                        ui,
                        if self.section == 0 {
                            Copy::UnlistedSub
                        } else {
                            Copy::LocalOnly
                        },
                    ),
                );
                b.label(400., 674., 816., 28., Copy::NoInterrupt, 20., 1);
            }
        }
        self.nodes = b.nodes;
        if self.sheet.is_none()
            && self.focus >= 0
            && !self
                .nodes
                .iter()
                .any(|n| n.actionable && n.id == self.focus)
        {
            self.focus = self
                .nodes
                .iter()
                .find(|n| n.actionable)
                .map_or(-1, |n| n.id);
        }
        ui.set_nodes(ModelRc::new(VecModel::from(self.nodes.clone())));
        ui.set_active_tab(self.tab as i32);
        ui.set_root_screen(self.root());
        ui.set_setup_screen(self.screen == Screen::Setup);
        ui.set_focused(self.focus);
        ui.set_favorite_hint(
            self.sheet.is_none()
                && (self.screen == Screen::Detail
                    || matches!(self.screen, Screen::Discover | Screen::Library)
                        && self.focus >= 20),
        );
        ui.set_detail_hint(self.sheet.is_none() && self.screen == Screen::Home);
        ui.set_import_hint(self.sheet.is_none() && self.screen == Screen::Library);
        let hint = if self.focus < 0 {
            Copy::Select
        } else if self.sheet.is_some() {
            Copy::Confirm
        } else {
            match (self.screen, self.focus) {
                (Screen::Home, 1) => Copy::Continue,
                (Screen::Home, 2) => Copy::Details,
                (Screen::Home, 3) => Copy::DiscoverMapsets,
                (Screen::Setup, 42) => Copy::Import,
                (Screen::Setup, 43) if self.mapset == MapsetId::Auger => Copy::Choose,
                (Screen::Setup, 44) => Copy::Choose,
                (Screen::Setup, 45) => Copy::Toggle,
                (Screen::Setup, 50) => {
                    if m.iwad_ready {
                        Copy::Play
                    } else {
                        Copy::AddFolder
                    }
                }
                (Screen::Detail, 30) => {
                    if m.installed {
                        Copy::PlaySetup
                    } else {
                        Copy::Install
                    }
                }
                (Screen::Detail, 32) => Copy::Profiles,
                (Screen::Detail, 37) => Copy::PlaySetup,
                (_, 10..=14) => Copy::Select,
                (_, 20..=22) => Copy::Open,
                _ => Copy::Inspect,
            }
        };
        ui.set_a_hint(text(ui, hint));
        let viewport = (ui.window().size().height as f32 - 294.).max(64.);
        let offset = self
            .nodes
            .iter()
            .find(|n| n.id == self.focus)
            .map_or(0., |n| (n.y + n.h + 12. - viewport).max(0.));
        ui.set_scroll_offset(offset);
        ui.set_sheet_open(self.sheet.is_some());
        if let Some(sheet) = self.sheet {
            ui.set_sheet_title(text(
                ui,
                if sheet == Sheet::Quit {
                    Copy::QuitTitle
                } else {
                    Copy::CompatConfirm
                },
            ));
            let mut b = Builder { ui, nodes: vec![] };
            b.button(
                100,
                60.,
                398.,
                304.,
                if sheet == Sheet::Quit {
                    Copy::KeepPlaying
                } else {
                    Copy::KeepMain
                },
                false,
            );
            b.button(
                101,
                396.,
                398.,
                304.,
                if sheet == Sheet::Quit {
                    Copy::Quit
                } else {
                    Copy::UseCompat
                },
                false,
            );
            ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
        }
    }
}
struct Builder<'a> {
    ui: &'a AppWindow,
    nodes: Vec<ViewNode>,
}
impl Builder<'_> {
    #[allow(clippy::too_many_arguments)]
    fn node(
        &mut self,
        id: i32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        kind: i32,
        label: SharedString,
        detail: SharedString,
        tone: i32,
        size: f32,
        primary: bool,
        actionable: bool,
    ) {
        self.nodes.push(ViewNode {
            id,
            x: x - 28.,
            y: y - 218.,
            w,
            h,
            kind,
            label,
            detail,
            tone,
            size,
            art: 0,
            screenshot: Default::default(),
            credit: Default::default(),
            full_credit: false,
            fitted_label: false,
            primary,
            actionable,
        });
    }
    fn panel(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.node(
            -100,
            x,
            y,
            w,
            h,
            1,
            Default::default(),
            Default::default(),
            0,
            22.,
            false,
            false,
        );
    }
    fn art(&mut self, x: f32, y: f32, w: f32, h: f32, art: i32) {
        self.node(
            -100,
            x,
            y,
            w,
            h,
            4,
            Default::default(),
            Default::default(),
            0,
            22.,
            false,
            false,
        );
        self.nodes.last_mut().unwrap().art = art;
    }
    #[allow(clippy::too_many_arguments)]
    fn data(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str, size: f32, tone: i32) {
        self.node(
            -100,
            x,
            y,
            w,
            h,
            0,
            label.into(),
            Default::default(),
            tone,
            size,
            false,
            false,
        );
    }
    #[allow(clippy::too_many_arguments)]
    fn label(&mut self, x: f32, y: f32, w: f32, h: f32, key: Copy, size: f32, tone: i32) {
        self.node(
            -100,
            x,
            y,
            w,
            h,
            0,
            text(self.ui, key),
            Default::default(),
            tone,
            size,
            false,
            false,
        );
    }
    fn button(&mut self, id: i32, x: f32, y: f32, w: f32, key: Copy, primary: bool) {
        self.node(
            id,
            x,
            y,
            w,
            if matches!(key, Copy::Appearance) {
                64.
            } else {
                52.
            },
            2,
            text(self.ui, key),
            Default::default(),
            0,
            22.,
            primary,
            true,
        );
    }
    fn row(&mut self, id: i32, x: f32, y: f32, w: f32, key: Copy, detail: SharedString) {
        self.node(
            id,
            x,
            y,
            w,
            72.,
            3,
            text(self.ui, key),
            detail,
            0,
            22.,
            false,
            true,
        );
    }
    fn card(&mut self, id: i32, x: f32, y: f32, w: f32, h: f32) {
        self.node(
            id,
            x,
            y,
            w,
            h,
            1,
            Default::default(),
            Default::default(),
            0,
            22.,
            false,
            true,
        );
    }
}
pub fn attach(ui: &AppWindow) -> Rc<RefCell<Shell>> {
    let state = Rc::new(RefCell::new(Shell::default()));
    state.borrow_mut().sync(ui);
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
    state
}
