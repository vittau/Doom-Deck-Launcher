//! Spec 026 controller-only local-content screens. Reuses the Ember node renderer.
use super::*;
use crate::LocalStrings;
use crate::services::{LocalCommand as Op, NewIwad};
use ddl_core::local::{self, Compatibility, CustomIwad, CustomSetup, Group, UserFile};
use ddl_core::profiles::Profile;

const NEW: i32 = 6000;
const CUSTOM: i32 = 6100;
const EXTRAS: i32 = 6200;
const ENTRY: i32 = 7000;
const GROUP_OFFSET: i32 = 100_000;
const SHORTCUT: i32 = 200_000;
const PICK: i32 = 300_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum View {
    New,
    Detail,
    Order,
    Iwad,
    Compat,
    Picker,
}
#[derive(Clone)]
pub(super) enum Dialog {
    Delete,
    Issue(PathBuf, bool),
    Saves,
    Error(String),
}
#[derive(Default)]
pub(super) struct LocalUi {
    pub(super) view: Option<View>,
    target: Option<String>,
    creating: Option<String>,
    iwad: Option<NewIwad>,
    compatibility: Compatibility,
    picker_iwad: bool,
    picker_return: Option<View>,
    selected: Vec<PathBuf>,
    moving: Option<(Group, usize)>,
    pub(super) dialog: Option<Dialog>,
    dialog_focus: i32,
    delete_profiles: bool,
    return_focus: i32,
    picker_file_start: usize,
    picker_shortcut_start: usize,
}
impl Runtime {
    fn local_setup(&self) -> Option<&CustomSetup> {
        let id = self.local.target.as_ref()?;
        self.snapshot.as_ref()?.custom.iter().find(|s| &s.id == id)
    }
    fn local_profile(&self) -> Option<Profile> {
        if let Some(setup) = self.local_setup() {
            self.snapshot
                .as_ref()?
                .custom_profiles
                .iter()
                .find(|p| p.recipe_id == setup.id)
                .cloned()
        } else {
            self.item().map(|i| i.profile.clone())
        }
    }
    fn local_files(&self, group: Group) -> Vec<UserFile> {
        if let Some(setup) = self.local_setup() {
            match group {
                Group::Files => setup.files.clone(),
                Group::Patches => setup.patches.clone(),
            }
        } else {
            self.local_profile().map_or_else(Vec::new, |p| {
                p.extras
                    .into_iter()
                    .filter(|f| f.kind.is_patch() == (group == Group::Patches))
                    .collect()
            })
        }
    }
    fn local_op(&mut self, ui: &AppWindow, operation: Op) {
        self.start(ui, Command::Local(operation));
    }
    fn local_open(&mut self, view: View) {
        self.local.view = Some(view);
        self.local.moving = None;
        self.focus = 1;
    }
    fn local_picker(&mut self, ui: &AppWindow, iwad: bool) {
        self.local.picker_return = self.local.view;
        self.local.picker_iwad = iwad;
        self.local.selected.clear();
        self.local_open(View::Picker);
        self.local_op(ui, Op::Browse(self.env.home.clone()));
    }
    pub(super) fn local_finished(&mut self, command: &Option<Command>) {
        if let Some(Command::Local(operation)) = command {
            match operation {
                Op::Create(name, ..) => {
                    self.local.target = self
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.custom.iter().find(|c| &c.name == name))
                        .map(|s| s.id.clone());
                    self.local.creating = None;
                    self.local_open(View::Detail);
                }
                Op::Browse(_) => {
                    self.local.view = Some(View::Picker);
                    self.local.picker_file_start = 0;
                    self.local.picker_shortcut_start = 0;
                    self.focus = 1;
                }
                Op::Delete(..) => {
                    self.local = LocalUi::default();
                    self.screen = Screen::Library;
                    self.focus = NEW;
                }
                _ => {}
            }
        }
    }
    fn local_begin(&mut self, ui: &AppWindow) {
        self.local = LocalUi::default();
        let c = ui.global::<LocalStrings>();
        let mut index = 1;
        loop {
            c.set_value(index.to_string().into());
            let name = c.get_custom_name().to_string();
            if !self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.custom.iter().any(|setup| setup.name == name))
            {
                self.local.creating = Some(name);
                break;
            }
            index += 1;
        }
        self.local.return_focus = NEW;
        self.local_open(View::New);
    }
    fn local_issue(&self) -> Option<(PathBuf, bool)> {
        if let Some(id) = &self.local.target
            && self.local.view.is_some()
        {
            self.snapshot.as_ref()?.local_issues.get(id).cloned()
        } else {
            match &self.item()?.launch {
                Err(Blocker::UserFileMissing { path }) => Some((path.clone(), false)),
                Err(Blocker::UserFileChanged { path }) => Some((path.clone(), true)),
                _ => None,
            }
        }
    }
    pub(super) fn local_show_issue(&mut self, issue: (PathBuf, bool)) {
        self.local.dialog = Some(Dialog::Issue(issue.0, issue.1));
        self.local.dialog_focus = self.focus;
        self.focus = 900;
    }
    pub(super) fn local_error(&mut self, message: String) {
        self.local.dialog = Some(Dialog::Error(message));
        self.local.dialog_focus = self.focus;
        self.focus = 900;
    }
    fn local_target(&self) -> Option<(String, Option<String>)> {
        if let Some(id) = &self.local.target {
            Some((id.clone(), None))
        } else {
            self.item()
                .map(|i| (i.recipe.id.clone(), Some(i.profile.id.clone())))
        }
    }
    fn local_store_order(&mut self, ui: &AppWindow, group: Group, from: usize, to: Option<usize>) {
        if let Some(mut setup) = self.local_setup().cloned() {
            let result = if let Some(to) = to {
                setup.move_entry(group, from, to)
            } else {
                setup.remove(group, from).map(|_| ())
            };
            if result.is_ok() {
                self.local_op(ui, Op::Save(setup));
            }
        } else if let Some(mut profile) = self.local_profile() {
            let indices = local::group_indices(&profile.extras, group);
            if let Some(&index) = indices.get(from) {
                if let Some(to) = to {
                    if let Some(&next) = indices.get(to) {
                        profile.extras.swap(index, next);
                    }
                } else {
                    profile.extras.remove(index);
                }
                self.local_op(ui, Op::Extras(profile));
            }
        }
    }
    fn local_row_index(&self) -> Option<(Group, usize)> {
        if (ENTRY..ENTRY + GROUP_OFFSET).contains(&self.focus) {
            Some((Group::Files, (self.focus - ENTRY) as usize))
        } else if (ENTRY + GROUP_OFFSET..ENTRY + 2 * GROUP_OFFSET).contains(&self.focus) {
            Some((Group::Patches, (self.focus - ENTRY - GROUP_OFFSET) as usize))
        } else {
            None
        }
    }
    pub(super) fn local_action(&mut self, ui: &AppWindow, action: Action) -> bool {
        if self.sheet.is_some() {
            return false;
        }
        if self.local.dialog.is_some() {
            if self.busy {
                return true;
            }
            match action {
                Action::Back => {
                    self.local.dialog = None;
                    self.focus = self.local.dialog_focus;
                }
                Action::Up | Action::Down | Action::Left | Action::Right => {
                    let saved =
                        std::mem::replace(&mut self.nodes, ui.get_sheet_nodes().iter().collect());
                    self.move_focus(action);
                    self.nodes = saved;
                }
                Action::Confirm if self.focus == 902 => {
                    self.local.delete_profiles = !self.local.delete_profiles
                }
                Action::Confirm => {
                    let dialog = self.local.dialog.take().unwrap();
                    let confirm = self.focus == 901;
                    self.focus = self.local.dialog_focus;
                    if confirm {
                        match dialog {
                            Dialog::Delete => {
                                if let Some(id) = self.local.target.clone() {
                                    self.local_op(ui, Op::Delete(id, self.local.delete_profiles));
                                }
                            }
                            Dialog::Issue(_, true) => {
                                if let Some((id, profile)) = self.local_target() {
                                    self.local_op(ui, Op::Accept(id, profile));
                                }
                            }
                            Dialog::Saves | Dialog::Issue(_, false) | Dialog::Error(_) => {}
                        }
                    }
                }
                _ => {}
            }
            return true;
        }
        if self.local.view.is_none() {
            if self.busy {
                return false;
            }
            match (self.screen, action, self.focus) {
                (Screen::Library, Action::Y, _) | (Screen::Library, Action::Confirm, NEW) => {
                    self.local_begin(ui)
                }
                (Screen::Library | Screen::Home, Action::Confirm, id)
                    if id >= CUSTOM
                        && self
                            .snapshot
                            .as_ref()
                            .is_some_and(|s| s.custom.get((id - CUSTOM) as usize).is_some()) =>
                {
                    self.local.target = self
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.custom.get((id - CUSTOM) as usize))
                        .map(|s| s.id.clone());
                    self.local.return_focus = id;
                    self.local_open(View::Detail);
                }
                (Screen::Setup, Action::Confirm, EXTRAS) => {
                    self.local.target = None;
                    self.local.return_focus = EXTRAS;
                    self.local_open(View::Order);
                    if self.local_profile().is_some_and(|p| {
                        self.snapshot
                            .as_ref()
                            .is_some_and(|s| s.profiles_with_saves.contains(&p.id))
                    }) {
                        self.local.dialog = Some(Dialog::Saves);
                        self.local.dialog_focus = 1;
                        self.focus = 900;
                    }
                }
                (Screen::Setup, Action::Confirm, 1 | 50) => {
                    if let Some(issue) = self.local_issue() {
                        self.local_show_issue(issue);
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
            return true;
        }
        if self.busy {
            return true;
        }
        let view = self.local.view.unwrap();
        if action == Action::Back {
            self.local.moving = None;
            match view {
                View::Picker => {
                    self.local.view = self.local.picker_return.take();
                    self.local.selected.clear();
                    self.focus = 1;
                }
                View::Iwad | View::Compat => self.local_open(if self.local.creating.is_some() {
                    View::New
                } else {
                    View::Detail
                }),
                View::Order if self.local.target.is_some() => self.local_open(View::Detail),
                _ => {
                    self.local.view = None;
                    self.focus = self.local.return_focus;
                }
            }
            return true;
        }
        if view == View::Order {
            if let Some((group, index)) = self.local.moving {
                if matches!(action, Action::Up | Action::Down) {
                    let len = self.local_files(group).len();
                    let next = if action == Action::Up {
                        index.saturating_sub(1)
                    } else {
                        (index + 1).min(len.saturating_sub(1))
                    };
                    if next != index {
                        self.local_store_order(ui, group, index, Some(next));
                        self.local.moving = Some((group, next));
                        self.focus = ENTRY
                            + if group == Group::Patches {
                                GROUP_OFFSET
                            } else {
                                0
                            }
                            + next as i32;
                    }
                    return true;
                }
                if action == Action::Confirm {
                    self.local.moving = None;
                    return true;
                }
            }
            if action == Action::Y {
                self.local_picker(ui, false);
                return true;
            }
            if action == Action::X {
                if let Some((group, index)) = self.local_row_index() {
                    self.local_store_order(ui, group, index, None);
                }
                return true;
            }
        }
        if view == View::Picker
            && matches!(action, Action::Up | Action::Down)
            && self.focus >= SHORTCUT
        {
            let ids: Vec<i32> = if self.focus >= PICK {
                self.directory.as_ref().map_or_else(Vec::new, |d| {
                    d.entries
                        .iter()
                        .enumerate()
                        .filter(|(_, (path, folder))| {
                            !self.local.picker_iwad || *folder || d.iwad_files.contains(path)
                        })
                        .map(|(index, _)| PICK + index as i32)
                        .collect()
                })
            } else {
                self.directory.as_ref().map_or_else(Vec::new, |d| {
                    (0..d.shortcuts.len())
                        .map(|index| SHORTCUT + index as i32)
                        .collect()
                })
            };
            if let Some(index) = ids.iter().position(|id| *id == self.focus) {
                if action == Action::Up && index == 0 {
                    self.focus = if self.focus >= PICK || self.local.picker_iwad {
                        2
                    } else {
                        1
                    };
                } else {
                    self.focus = ids[if action == Action::Up {
                        index - 1
                    } else {
                        (index + 1).min(ids.len() - 1)
                    }];
                }
            }
            return true;
        }
        if matches!(
            action,
            Action::Up | Action::Down | Action::Left | Action::Right
        ) {
            self.move_focus(action);
            return true;
        }
        if view == View::Picker && action == Action::Menu {
            self.local_add_selected(ui);
            return true;
        }
        if action != Action::Confirm {
            return true;
        }
        match view {
            View::New | View::Detail => match self.focus {
                1 if view == View::Detail => {
                    if let Some(issue) = self.local_issue() {
                        self.local_show_issue(issue);
                    } else if let Some(id) = &self.local.target {
                        self.local_op(ui, Op::Play(id.clone()));
                    }
                }
                1 => self.local_open(View::Iwad),
                2 if view == View::Detail => self.local_open(View::Order),
                2 => self.local_open(View::Compat),
                3 if view == View::Detail => self.local_open(View::Iwad),
                3 => {
                    if let (Some(name), Some(iwad)) =
                        (self.local.creating.clone(), self.local.iwad.clone())
                    {
                        self.local_op(ui, Op::Create(name, iwad, self.local.compatibility));
                    }
                }
                4 => self.local_open(View::Compat),
                5 => {
                    self.local.delete_profiles = false;
                    self.local.dialog = Some(Dialog::Delete);
                    self.local.dialog_focus = 5;
                    self.focus = 900;
                }
                _ => {}
            },
            View::Iwad => {
                if self.focus == 1 {
                    self.local_picker(ui, true);
                } else if let Some(variant) = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| {
                        s.iwads
                            .iter()
                            .filter_map(|e| e.variant.as_ref())
                            .nth((self.focus - 10) as usize)
                    })
                    .cloned()
                {
                    self.local_choose_iwad(ui, NewIwad::Variant(variant));
                }
            }
            View::Compat => {
                let values = compatibility_values();
                if let Some(choice) = values.get((self.focus - 10) as usize) {
                    self.local.compatibility = *choice;
                    if let Some(mut setup) = self.local_setup().cloned() {
                        setup.compatibility = *choice;
                        self.local_op(ui, Op::Save(setup));
                    }
                    self.local_open(if self.local.creating.is_some() {
                        View::New
                    } else {
                        View::Detail
                    });
                }
            }
            View::Order => {
                if self.focus == 1 {
                    self.local_picker(ui, false);
                } else if let Some(entry) = self.local_row_index() {
                    self.local.moving = Some(entry);
                }
            }
            View::Picker => {
                if self.focus == 1 {
                    self.local_add_selected(ui);
                } else if self.focus == 2 {
                    if let Some(parent) = self
                        .directory
                        .as_ref()
                        .and_then(|d| d.path.parent())
                        .map(PathBuf::from)
                    {
                        self.local_op(ui, Op::Browse(parent));
                    }
                } else if (SHORTCUT..PICK).contains(&self.focus) {
                    if let Some(path) = self
                        .directory
                        .as_ref()
                        .and_then(|d| d.shortcuts.get((self.focus - SHORTCUT) as usize))
                        .cloned()
                    {
                        self.local_op(ui, Op::Browse(path));
                    }
                } else if self.focus >= PICK
                    && let Some((path, directory)) = self
                        .directory
                        .as_ref()
                        .and_then(|d| d.entries.get((self.focus - PICK) as usize))
                        .cloned()
                {
                    if directory {
                        self.local_op(ui, Op::Browse(path));
                    } else if self.local.picker_iwad {
                        self.local_choose_iwad(ui, NewIwad::File(path));
                    } else if let Some(index) = self.local.selected.iter().position(|p| p == &path)
                    {
                        self.local.selected.remove(index);
                    } else {
                        self.local.selected.push(path);
                    }
                }
            }
        }
        true
    }
    fn local_choose_iwad(&mut self, ui: &AppWindow, iwad: NewIwad) {
        if let Some(mut setup) = self.local_setup().cloned() {
            match &iwad {
                NewIwad::Variant(variant) => {
                    setup.iwad = CustomIwad::Identified {
                        variant: variant.clone(),
                    }
                }
                NewIwad::File(path) => {
                    // The picker already read this file on the services worker.
                    let Some(file) = self
                        .directory
                        .as_ref()
                        .and_then(|d| d.files.iter().find(|f| &f.path == path))
                        .cloned()
                    else {
                        return;
                    };
                    setup.iwad = CustomIwad::File { file };
                }
            }
            self.local_op(ui, Op::Save(setup));
        }
        self.local.iwad = Some(iwad);
        self.local_open(if self.local.creating.is_some() {
            View::New
        } else {
            View::Detail
        });
    }
    fn local_add_selected(&mut self, ui: &AppWindow) {
        if self.local.selected.is_empty() || self.local.picker_iwad {
            return;
        }
        if let Some((id, profile)) = self.local_target() {
            let paths = std::mem::take(&mut self.local.selected);
            self.local_open(View::Order);
            self.local_op(ui, Op::Add(id, profile, paths));
        }
    }
    pub(super) fn local_nodes(&mut self, ui: &AppWindow, b: &mut Nodes) {
        if self.playing {
            return;
        }
        let c = ui.global::<LocalStrings>();
        if let Some(view) = self.local.view {
            b.nodes.clear();
            if view != View::Picker {
                b.panel(12., 12., 1200., 478.);
            }
            match view {
                View::New | View::Detail => {
                    b.art(36., 36., 270., 330., 4);
                    let setup = self.local_setup();
                    let name = setup
                        .map(|s| s.name.clone())
                        .or_else(|| self.local.creating.clone())
                        .unwrap_or_default();
                    b.title(340., 30., 800., 50., name.into(), 34.);
                    b.label(340., 88., 800., 32., c.get_badge(), 22.);
                    b.label(36., 388., 270., 80., c.get_local_help(), 20.);
                    if view == View::New {
                        let iwad = self
                            .local
                            .iwad
                            .as_ref()
                            .map(iwad_name)
                            .unwrap_or_else(|| c.get_iwad().to_string());
                        b.row(1, 340., 138., 824., c.get_iwad(), iwad.into(), true);
                        b.row(
                            2,
                            340.,
                            234.,
                            824.,
                            c.get_compat(),
                            compatibility_name(ui, self.local.compatibility),
                            true,
                        );
                        if self.local.iwad.is_some() {
                            b.button(3, 340., 354., 824., c.get_create(), true);
                        }
                    } else if let Some(setup) = setup {
                        b.button(1, 340., 138., 394., c.get_play(), true);
                        b.button(2, 758., 138., 406., c.get_order(), false);
                        let base = match &setup.iwad {
                            CustomIwad::Identified { variant } => variant.clone(),
                            CustomIwad::File { file } => {
                                elide_path_start(&file.path.display().to_string(), 64)
                            }
                        };
                        b.row(3, 340., 214., 824., c.get_iwad(), base.into(), true);
                        b.row(
                            4,
                            340.,
                            310.,
                            824.,
                            c.get_compat(),
                            compatibility_name(ui, setup.compatibility),
                            true,
                        );
                        b.button(5, 340., 414., 824., c.get_delete(), false);
                    }
                }
                View::Iwad => {
                    b.button(1, 36., 36., 1152., c.get_added_iwad(), true);
                    if let Some(s) = &self.snapshot {
                        for (index, variant) in s
                            .iwads
                            .iter()
                            .filter_map(|e| e.variant.as_ref())
                            .enumerate()
                        {
                            b.row(
                                10 + index as i32,
                                36.,
                                128. + index as f32 * 96.,
                                1152.,
                                variant.clone().into(),
                                c.get_iwad(),
                                true,
                            );
                        }
                    }
                }
                View::Compat => {
                    for (index, choice) in compatibility_values().into_iter().enumerate() {
                        b.choice(
                            10 + index as i32,
                            36.,
                            36. + index as f32 * 68.,
                            1152.,
                            compatibility_name(ui, choice),
                            self.local_setup()
                                .map_or(self.local.compatibility, |s| s.compatibility)
                                == choice,
                        );
                    }
                }
                View::Order => {
                    b.button(1, 36., 30., 1152., c.get_add_hint(), true);
                    let mut y = 110.;
                    for group in [Group::Files, Group::Patches] {
                        b.label(
                            36.,
                            y,
                            1152.,
                            40.,
                            if group == Group::Files {
                                c.get_files()
                            } else {
                                c.get_patches()
                            },
                            28.,
                        );
                        y += 52.;
                        if self.local.target.is_none()
                            && let Some(item) = self.item()
                        {
                            let enabled: Vec<_> =
                                item.profile.components.iter().map(String::as_str).collect();
                            let locked: Vec<String> = item
                                .recipe
                                .active_components(&enabled)
                                .unwrap_or_default()
                                .into_iter()
                                .flat_map(|component| {
                                    component.members.iter().filter_map(move |member| {
                                        let patch =
                                            member.role == ddl_core::catalog::Role::Dehacked;
                                        if patch != (group == Group::Patches) {
                                            return None;
                                        }
                                        let manifest = item.manifest.as_ref()?;
                                        let installed = match (&member.package, &member.member) {
                                            (Some(package), Some(member)) => {
                                                manifest.find_downloaded(package, member)
                                            }
                                            _ => manifest.find_imported(&component.id),
                                        }?;
                                        Some(
                                            installed
                                                .path
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy()
                                                .to_string(),
                                        )
                                    })
                                })
                                .collect();
                            for name in locked {
                                b.row(0, 36., y, 1152., name.into(), c.get_locked(), false);
                                let row = b.nodes.last_mut().unwrap();
                                row.tone = 1;
                                row.kind = 16;
                                y += 100.;
                            }
                        }
                        let files = self.local_files(group);
                        if files.is_empty() {
                            b.label(36., y, 1152., 36., c.get_none(), 22.);
                            y += 52.;
                        }
                        for (index, file) in files.iter().enumerate() {
                            let id = ENTRY
                                + if group == Group::Patches {
                                    GROUP_OFFSET
                                } else {
                                    0
                                }
                                + index as i32;
                            let moving = self.local.moving == Some((group, index));
                            b.row(
                                id,
                                36.,
                                y,
                                1152.,
                                format!(
                                    "{}  {}",
                                    index + 1,
                                    file.path.file_name().unwrap_or_default().to_string_lossy()
                                )
                                .into(),
                                if moving {
                                    c.get_move()
                                } else {
                                    format!("{} · {}", file.kind, human_bytes(file.size)).into()
                                },
                                true,
                            );
                            b.nodes.last_mut().unwrap().tone = if moving { 3 } else { 0 };
                            y += 100.;
                        }
                    }
                }
                View::Picker => {
                    c.set_value(self.local.selected.len().to_string().into());
                    if !self.local.picker_iwad {
                        b.button(1, 36., 30., 780., c.get_add(), true);
                    }
                    b.button(2, 840., 30., 348., c.get_up(), false);
                    let height = (ui.window().size().height as f32 - 404.).max(152.);
                    let capacity = ((height - 32.) / 88.).floor().max(1.) as usize;
                    b.panel(12., 110., 352., height);
                    b.panel(380., 110., 832., height);
                    if let Some(directory) = &self.directory {
                        if (SHORTCUT..PICK).contains(&self.focus) {
                            self.local.picker_shortcut_start = page_start(
                                (self.focus - SHORTCUT) as usize,
                                self.local.picker_shortcut_start,
                                capacity,
                            );
                        }
                        for (index, path) in directory
                            .shortcuts
                            .iter()
                            .enumerate()
                            .skip(self.local.picker_shortcut_start)
                            .take(capacity)
                        {
                            b.row(
                                SHORTCUT + index as i32,
                                36.,
                                126. + (index - self.local.picker_shortcut_start) as f32 * 88.,
                                304.,
                                path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                                    .into(),
                                c.get_shortcut(),
                                true,
                            );
                            b.nodes.last_mut().unwrap().h = 72.;
                        }
                        let entries: Vec<_> = directory
                            .entries
                            .iter()
                            .enumerate()
                            .filter(|(_, (path, folder))| {
                                !self.local.picker_iwad
                                    || *folder
                                    || directory.iwad_files.contains(path)
                            })
                            .collect();
                        if self.focus >= PICK
                            && let Some(index) = entries
                                .iter()
                                .position(|(index, _)| PICK + *index as i32 == self.focus)
                        {
                            self.local.picker_file_start =
                                page_start(index, self.local.picker_file_start, capacity);
                        }
                        for (visible, (index, (path, _))) in entries
                            .iter()
                            .enumerate()
                            .skip(self.local.picker_file_start)
                            .take(capacity)
                        {
                            let file = directory.files.iter().find(|f| &f.path == path);
                            let label = format!(
                                "{}{}",
                                if self.local.selected.contains(path) {
                                    "✓  "
                                } else {
                                    ""
                                },
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                            let detail = file.map_or_else(
                                || elide_path_start(&path.display().to_string(), 48),
                                |f| format!("{} · {}", f.kind, human_bytes(f.size)),
                            );
                            b.row(
                                PICK + *index as i32,
                                404.,
                                126. + (visible - self.local.picker_file_start) as f32 * 88.,
                                784.,
                                label.into(),
                                detail.into(),
                                true,
                            );
                            b.nodes.last_mut().unwrap().h = 72.;
                        }
                    }
                }
            }
            for node in &mut b.nodes {
                if node.kind == 3 {
                    node.fitted_label = true;
                }
            }
            if view != View::Picker {
                let bottom = b.nodes.iter().map(|n| n.y + n.h).fold(478., f32::max);
                b.nodes[0].h = bottom + 24. - b.nodes[0].y;
            }
            return;
        }
        if matches!(self.screen, Screen::Library | Screen::Discover)
            && let Some(snapshot) = &self.snapshot
        {
            for item in &snapshot.items {
                if !item.profile.extras.is_empty()
                    && let Some(card) = b
                        .nodes
                        .iter()
                        .find(|n| {
                            n.id == 10
                                + snapshot
                                    .items
                                    .iter()
                                    .position(|i| i.recipe.id == item.recipe.id)
                                    .unwrap() as i32
                        })
                        .cloned()
                {
                    for node in &mut b.nodes {
                        if node.x == card.x + 20. && node.y == card.y + 300. {
                            node.label = c.get_badge();
                        }
                    }
                }
            }
        }
        if self.screen == Screen::Library {
            if self.library_items().is_empty()
                && self.snapshot.as_ref().is_some_and(|s| !s.custom.is_empty())
            {
                b.nodes.clear();
            }
            for node in &mut b.nodes {
                node.y += 84.;
            }
            b.button(NEW, 12., 12., 1200., c.get_new(), true);
            let shown = self.library_items().len();
            if let Some(snapshot) = &self.snapshot {
                for (index, setup) in snapshot.custom.iter().enumerate() {
                    let slot = shown + index;
                    let x = 12. + 408. * (slot % 3) as f32;
                    let y = 96. + 438. * (slot / 3) as f32;
                    b.card(CUSTOM + index as i32, x, y, 384., 410.);
                    b.art(x + 16., y + 16., 352., 142., 4);
                    b.title(x + 20., y + 174., 344., 80., setup.name.clone().into(), 26.);
                    b.label(x + 20., y + 270., 344., 56., c.get_badge(), 22.);
                    b.label(x + 20., y + 340., 344., 40., c.get_order(), 20.);
                }
            }
        }
        if self.screen == Screen::Home
            && let Some(s) = &self.snapshot
        {
            let catalog_time = s.items.iter().filter_map(|i| i.last_played).max();
            let latest = s
                .custom
                .iter()
                .enumerate()
                .filter_map(|(index, setup)| {
                    let time = s
                        .custom_profiles
                        .iter()
                        .filter(|p| p.recipe_id == setup.id)
                        .filter_map(|p| p.last_played)
                        .max()?;
                    Some((index, setup, time))
                })
                .max_by_key(|(_, _, time)| *time);
            if let Some((index, setup, time)) = latest
                && catalog_time.is_none_or(|old| time >= old)
            {
                if self.focus == 1 {
                    self.focus = CUSTOM + index as i32;
                }
                b.nodes.retain(|n| n.x >= 800. || n.y > 330. || n.kind == 1);
                b.art(36., 36., 240., 300., 4);
                b.nodes
                    .retain(|n| !(n.x == 852. && [172., 244., 316.].contains(&n.y)));
                b.label(852., 172., 336., 80., c.get_iwad_selected(), 22.);
                b.label(852., 280., 336., 90., c.get_badge(), 22.);
                b.title(308., 56., 448., 80., setup.name.clone().into(), 36.);
                b.label(308., 156., 448., 64., c.get_badge(), 22.);
                let profile_name = s
                    .custom_profiles
                    .iter()
                    .find(|p| p.recipe_id == setup.id)
                    .map_or_else(String::new, |p| p.name.clone());
                b.label(308., 212., 448., 28., profile_name.clone().into(), 22.);
                if catalog_time.is_none() {
                    b.nodes.retain(|n| n.label != text(ui, T::NoRecent));
                    b.art(36., 400., 56., 66., 4);
                    b.title(108., 400., 600., 34., setup.name.clone().into(), 26.);
                    b.label(108., 436., 600., 30., profile_name.into(), 20.);
                }

                b.button(
                    CUSTOM + index as i32,
                    308.,
                    252.,
                    448.,
                    text(ui, T::Continue),
                    true,
                );
            }
        }
        if self.screen == Screen::Setup && self.item().is_some_and(|i| !i.profile.extras.is_empty())
        {
            if let Some(node) = b.nodes.iter_mut().find(|n| n.x == 740. && n.y == 36.) {
                node.label = c.get_badge();
                node.size = 24.;
            }
            if self.local_issue().is_some()
                && let Some(node) = b.nodes.iter_mut().find(|n| n.id == 1 && n.actionable)
            {
                node.label = c.get_blocked();
            }
        }
        if self.screen == Screen::Setup && self.item().is_some_and(|i| i.manifest.is_some()) {
            let y = b
                .nodes
                .iter()
                .filter(|n| n.x < 700.)
                .map(|n| n.y + n.h)
                .fold(0., f32::max)
                + 20.;
            if let Some(panel) = b
                .nodes
                .iter_mut()
                .find(|n| n.kind == 1 && n.x == 12. && n.w == 680.)
            {
                panel.h = y + 104. - panel.y;
            }
            b.row(
                EXTRAS,
                36.,
                y,
                620.,
                c.get_extras(),
                if self.item().is_some_and(|i| !i.profile.extras.is_empty()) {
                    c.get_badge()
                } else {
                    c.get_order_help()
                },
                true,
            );
        }
    }
    pub(super) fn local_sheet(&mut self, ui: &AppWindow) {
        if self.playing {
            ui.set_local_add_hint(false);
            ui.set_local_remove_hint(false);
            ui.set_local_confirm_hint(false);
            return;
        }
        if self.screen == Screen::Home
            && self.local.view.is_none()
            && let Some(snapshot) = &self.snapshot
        {
            let catalog_time = snapshot.items.iter().filter_map(|i| i.last_played).max();
            let custom_time = snapshot
                .custom_profiles
                .iter()
                .filter(|p| snapshot.custom.iter().any(|s| s.id == p.recipe_id))
                .filter_map(|p| p.last_played)
                .max();
            if custom_time.is_some_and(|time| catalog_time.is_none_or(|old| time >= old)) {
                ui.set_band_art(4);
                ui.set_band_screenshot(Default::default());
                ui.set_subtitle(text(ui, T::HomeSub));
            }
        }
        let c = ui.global::<LocalStrings>();
        ui.set_local_add_hint(
            self.local.view == Some(View::Order)
                || self.screen == Screen::Library && self.local.view.is_none(),
        );
        ui.set_local_remove_hint(self.local.view == Some(View::Order));
        ui.set_local_confirm_hint(self.local.view == Some(View::Picker) && !self.local.picker_iwad);
        if let Some(view) = self.local.view {
            ui.set_root_screen(false);
            ui.set_setup_screen(false);
            ui.set_art_band(true);
            ui.set_band_art(4);
            ui.set_band_screenshot(Default::default());
            ui.set_favorite_hint(false);
            ui.set_category_hint(false);
            ui.set_import_hint(false);
            ui.set_heading(match view {
                View::New => c.get_new(),
                View::Detail => self
                    .local_setup()
                    .map_or(c.get_new(), |s| s.name.clone().into()),
                View::Order => {
                    if self.local.target.is_some() {
                        c.get_order()
                    } else {
                        c.get_extras()
                    }
                }
                View::Iwad => c.get_iwad(),
                View::Compat => c.get_compat(),
                View::Picker => c.get_picker(),
            });
            ui.set_subtitle(match view {
                View::New => c.get_new_help(),
                View::Order => c.get_order_help(),
                View::Iwad => c.get_iwad_help(),
                View::Compat => c.get_auto(),
                View::Picker => self
                    .directory
                    .as_ref()
                    .map_or(c.get_picker_help(), |d| d.path.display().to_string().into()),
                View::Detail => c.get_badge(),
            });
            if view == View::Order && self.local_row_index().is_some() {
                ui.set_a_hint(if self.local.moving.is_some() {
                    c.get_drop()
                } else {
                    c.get_pick_up()
                });
            }
            if view == View::Picker && self.focus >= PICK {
                ui.set_a_hint(c.get_select());
            }
            // The picker pages complete rows; other local screens share a scrollable plane.
            let viewport = (ui.window().size().height as f32 - 294.).max(64.);
            let offset = self
                .nodes
                .iter()
                .find(|n| n.id == self.focus)
                .map_or(0., |n| (n.y + n.h + 12. - viewport).max(0.));
            ui.set_scroll_offset(if view == View::Picker { 0. } else { offset });
        }
        if let Some(dialog) = &self.local.dialog {
            ui.set_sheet_open(true);
            ui.set_sheet_height(420.);
            let mut b = Nodes::new(ui);
            if matches!(dialog, Dialog::Delete) {
                c.set_value(
                    self.local_setup()
                        .map_or_else(SharedString::default, |s| s.name.clone().into()),
                );
            }
            ui.set_sheet_title(match dialog {
                Dialog::Delete => c.get_delete_title(),
                Dialog::Issue(..) => c.get_blocked(),
                Dialog::Saves => c.get_extras(),
                Dialog::Error(_) => c.get_error(),
            });
            match dialog {
                Dialog::Delete => {
                    b.label(32., 96., 656., 36., c.get_delete_help(), 22.);
                    b.checkbox(
                        902,
                        32.,
                        144.,
                        656.,
                        c.get_delete_check(),
                        self.local.delete_profiles,
                    );
                    b.label(
                        32.,
                        216.,
                        656.,
                        64.,
                        if self.local.delete_profiles {
                            c.get_deleted()
                        } else {
                            c.get_kept()
                        },
                        22.,
                    );
                    b.button(901, 368., 312., 304., c.get_delete(), false);
                }
                Dialog::Issue(path, changed) => {
                    b.title(
                        32.,
                        92.,
                        656.,
                        44.,
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                            .into(),
                        30.,
                    );
                    b.author(
                        32.,
                        148.,
                        656.,
                        32.,
                        elide_path_middle(&path.display().to_string(), 64).into(),
                        20.,
                    );
                    b.label(
                        32.,
                        204.,
                        656.,
                        48.,
                        if *changed {
                            c.get_file_changed()
                        } else {
                            c.get_file_missing()
                        },
                        22.,
                    );
                    if *changed {
                        b.button(901, 368., 312., 304., c.get_changed(), false);
                    } else {
                        b.label(32., 260., 656., 36., c.get_missing(), 20.);
                    }
                }
                Dialog::Error(message) => {
                    b.label(32., 96., 656., 180., message.clone().into(), 22.);
                }
                Dialog::Saves => {
                    b.label(32., 112., 656., 120., c.get_saves(), 24.);
                }
            }
            b.button(
                900,
                32.,
                312.,
                304.,
                if matches!(dialog, Dialog::Saves) {
                    c.get_continue()
                } else {
                    c.get_cancel()
                },
                false,
            );
            if let Some(node) = b.nodes.iter().find(|n| n.id == self.focus) {
                ui.set_a_hint(node.label.clone());
            }
            ui.set_sheet_nodes(ModelRc::new(VecModel::from(b.nodes)));
            ui.set_focused(self.focus);
        }
    }
}
fn page_start(index: usize, start: usize, capacity: usize) -> usize {
    if index < start {
        index
    } else if index >= start + capacity {
        index + 1 - capacity
    } else {
        start
    }
}
fn elide_path_middle(path: &str, limit: usize) -> String {
    let chars: Vec<_> = path.chars().collect();
    if chars.len() <= limit {
        return path.to_owned();
    }
    let left = (limit - 1) / 2;
    let right = limit - 1 - left;
    format!(
        "{}…{}",
        chars[..left].iter().collect::<String>(),
        chars[chars.len() - right..].iter().collect::<String>()
    )
}
fn iwad_name(iwad: &NewIwad) -> String {
    match iwad {
        NewIwad::Variant(v) => v.clone(),
        NewIwad::File(p) => p.display().to_string(),
    }
}
fn compatibility_values() -> Vec<Compatibility> {
    std::iter::once(Compatibility::Auto)
        .chain(
            (0..=17)
                .chain(std::iter::once(21))
                .filter_map(|n| Compatibility::parse(&n.to_string()).ok()),
        )
        .collect()
}
fn compatibility_name(ui: &AppWindow, value: Compatibility) -> SharedString {
    if value == Compatibility::Auto {
        ui.global::<LocalStrings>().get_auto()
    } else {
        value.to_string().into()
    }
}

#[cfg(test)]
#[path = "local_ui_tests.rs"]
mod tests;
