mod input;
mod list;
mod marks;
mod persistence;
#[cfg(test)]
mod tests;

pub mod colors;
pub mod copy;
pub mod editor;
pub mod info;
pub mod preview;
pub mod saved;
pub mod theme_target;

use ratatui::{layout::Direction, widgets::ListState};

pub use crate::app::{
    editor::{Editor, FilterField},
    input::{Action, InputContext},
};
use crate::{
    app::{
        colors::ColorDetails,
        copy::{CopyRequest, CopySelection},
        info::ThemeInfo,
        saved::{FilterDialog, SavedScreen},
    },
    marks::{MarkChange, Marks},
    preview::{ContentSize, Preset},
    search::Filter,
    theme::{self, Theme},
    workspace::{Session, Workspace, WorkspaceAction},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Explore,
    Compare,
}

impl Mode {
    pub fn is_compare(self) -> bool {
        self == Self::Compare
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Explore => "Explore",
            Self::Compare => "Compare",
        }
    }
}

pub struct HelpScreen {
    pub context: InputContext,
    pub offset: usize,
}

pub enum Notice {
    Message(String),
    Success(String),
    PinRequired,
    Warning(String),
    Error(String),
}

pub struct App {
    pub color_details: ColorDetails,
    pub theme_info: Option<ThemeInfo>,
    suspended_theme_info: Option<ThemeInfo>,
    pub browser_action: Option<String>,
    pub copy_selection: Option<CopySelection>,
    pub copy_action: Option<CopyRequest>,
    pub preset_selection: Option<crate::app::preview::PreviewSelection>,
    pub help: Option<HelpScreen>,
    pub saved: Option<SavedScreen>,
    pub dialog: Option<FilterDialog>,
    pub workspace: Workspace,
    pub workspace_action: Option<WorkspaceAction>,
    session: Session,
    pub marks: Marks,
    pub storage_action: Option<MarkChange>,
    pub read_only: bool,
    pub notice: Option<Notice>,
    themes: &'static [Theme],
    results: Vec<usize>,
    pin: Option<String>,
    pub filter: Filter,
    pub editor: Option<Editor>,
    pub theme_list: ListState,
    pub preset: Preset,
    pub preview_content: ContentSize,
    pub mode: Mode,
    pub compare_direction: Direction,
    pub details_maximized: bool,
    explore_list_visible: bool,
    compare_list_visible: bool,
    pub quit: bool,
}

impl Default for App {
    fn default() -> Self {
        let themes = theme::THEMES;
        let results = (0..themes.len()).collect();
        Self {
            color_details: ColorDetails::default(),
            theme_info: None,
            suspended_theme_info: None,
            browser_action: None,
            copy_selection: None,
            copy_action: None,
            preset_selection: None,
            help: None,
            saved: None,
            dialog: None,
            workspace: Workspace::default(),
            workspace_action: None,
            session: Session::default(),
            marks: Marks::new(),
            storage_action: None,
            read_only: false,
            notice: None,
            themes,
            results,
            pin: None,
            filter: Filter::default(),
            editor: None,
            theme_list: ListState::default().with_selected(Some(0)),
            preset: Preset::Palette,
            preview_content: ContentSize::Auto,
            mode: Mode::Explore,
            compare_direction: Direction::Horizontal,
            details_maximized: false,
            explore_list_visible: true,
            compare_list_visible: false,
            quit: false,
        }
    }
}

impl App {
    pub fn panel_maximized(&self) -> Option<bool> {
        (self.theme_info.is_some() || self.color_details.is_open())
            .then_some(self.details_maximized)
    }

    pub fn theme_list_visible(&self) -> bool {
        match self.mode {
            Mode::Explore => self.explore_list_visible,
            Mode::Compare => self.compare_list_visible,
        }
    }

    pub fn themes(&self) -> impl ExactSizeIterator<Item = &Theme> {
        self.results.iter().map(|&index| &self.themes[index])
    }

    pub fn total(&self) -> usize {
        self.themes.len()
    }

    pub fn selected(&self) -> Option<usize> {
        self.theme_list.selected()
    }

    pub fn theme(&self) -> Option<&Theme> {
        self.selected()
            .map(|index| &self.themes[self.results[index]])
    }

    pub fn pinned_theme(&self) -> Option<&Theme> {
        let name = self.pin.as_deref()?;
        self.themes.iter().find(|theme| theme.name == name)
    }

    fn results_for(&self, filter: &Filter) -> Vec<usize> {
        self.themes
            .iter()
            .enumerate()
            .filter_map(|(index, theme)| {
                filter
                    .matches(theme, self.marks.get(theme.name).copied())
                    .then_some(index)
            })
            .collect()
    }

    fn apply_filter(&mut self, filter: Filter) {
        let selected_index = self.selected().map(|index| self.results[index]);
        let results = self.results_for(&filter);
        let selected = selected_index
            .and_then(|index| results.iter().position(|&result| result == index))
            .or_else(|| (!results.is_empty()).then_some(0));
        self.results = results;
        self.filter = filter;
        self.theme_list.select(selected);
    }

    pub fn prepare_draw(&mut self, rows: usize) {
        if self.saved.is_some() {
            self.prepare_saved(rows);
            return;
        }
        if let Some(Editor::Filters(edit)) = &mut self.editor {
            edit.offset = edit.offset.min(self.results.len().saturating_sub(rows));
        } else {
            self.ensure_selection_visible(rows);
        }
    }

    fn ensure_selection_visible(&mut self, rows: usize) {
        if rows == 0 {
            return;
        }
        let mut offset = self
            .theme_list
            .offset()
            .min(self.results.len().saturating_sub(rows));
        if let Some(selected) = self.selected() {
            if selected < offset {
                offset = selected;
            } else if selected >= offset + rows {
                offset = selected + 1 - rows;
            }
        }
        *self.theme_list.offset_mut() = offset;
    }

    pub fn update(&mut self, action: Action, rows: usize) {
        self.notice = None;
        if action == Action::Quit {
            self.quit = true;
            return;
        }
        self.sync_theme_info();
        self.sync_color_target();
        let context = self.context();
        if !context.allows(action) || !self.action_enabled(action) {
            return;
        }
        if action == Action::ToggleHelp {
            if self.help.is_some() {
                self.help = None;
            } else {
                self.help = Some(HelpScreen { context, offset: 0 });
            }
            return;
        }
        if let Some(help) = &mut self.help {
            match action {
                Action::Cancel => self.help = None,
                Action::Next => help.offset = help.offset.saturating_add(1),
                Action::Previous => help.offset = help.offset.saturating_sub(1),
                Action::PageNext => help.offset = help.offset.saturating_add(rows),
                Action::PagePrevious => help.offset = help.offset.saturating_sub(rows),
                Action::First => help.offset = 0,
                Action::Last => help.offset = usize::MAX,
                _ => {}
            }
            return;
        }
        if self.preset_selection.is_some() {
            self.update_preview(action);
            return;
        }
        if self.copy_selection.is_some() {
            self.update_copy(action);
            return;
        }
        if self.dialog.is_some() {
            self.update_filter_dialog(action);
            return;
        }
        if self.saved.is_some() {
            self.update_saved(action, rows);
            return;
        }
        if self.editor.is_some() {
            self.update_editor(action, rows);
            self.sync_color_target();
            return;
        }
        if action == Action::TogglePanelMaximize {
            self.details_maximized = !self.details_maximized;
            return;
        }
        if self.theme_info.is_some() && self.update_theme_info(action) {
            return;
        }
        if context == InputContext::Colors && self.color_details.navigate(action, rows) {
            return;
        }
        match action {
            Action::ToggleColors => self.toggle_colors(),
            Action::SwitchThemeTarget if self.can_switch_color_target() => {
                self.color_details.target = self.color_details.target.other();
            }
            Action::FieldNext | Action::FieldPrevious => self.color_details.switch_focus(),
            Action::OpenCopy => self.open_copy(),
            Action::OpenThemeInfo => self.open_theme_info(),
            Action::OpenPreset => self.open_preview(),
            Action::SaveFilter => self.open_filter_dialog(action),
            Action::OpenSaved => self.open_saved(),
            Action::EditQuery | Action::EditFilters => self.open_editor(action),
            Action::ClearFilters if !self.filter.is_default() => {
                self.apply_filter(Filter::default());
                self.record_session(true);
            }
            Action::Cancel if self.color_details.is_open() => self.close_colors(),
            Action::Cancel | Action::ToggleCompare if self.mode.is_compare() => {
                self.mode = Mode::Explore;
            }
            Action::ToggleThemeList => {
                let visible = match self.mode {
                    Mode::Explore => &mut self.explore_list_visible,
                    Mode::Compare => &mut self.compare_list_visible,
                };
                *visible = !*visible;
            }
            Action::PresetPrevious | Action::PresetNext => {
                self.preset = if action == Action::PresetNext {
                    self.preset.next()
                } else {
                    self.preset.previous()
                };
                self.record_session(false);
            }
            Action::ToggleFavorite | Action::ToggleIgnore => {
                self.update_marks(action);
            }
            Action::TogglePin => {
                let Some(theme) = self.theme() else {
                    return;
                };
                if self.pin.as_deref() == Some(theme.name) {
                    self.pin = None;
                    if self.mode.is_compare() {
                        self.mode = Mode::Explore;
                    }
                } else {
                    self.pin = Some(theme.name.to_owned());
                }
            }
            Action::ToggleCompare => {
                if self.pin.is_some() {
                    self.mode = Mode::Compare;
                } else {
                    self.notice = Some(Notice::PinRequired);
                }
            }
            Action::ToggleLayout if self.mode.is_compare() => {
                self.compare_direction = match self.compare_direction {
                    Direction::Horizontal => Direction::Vertical,
                    Direction::Vertical => Direction::Horizontal,
                };
                self.record_session(false);
            }
            _ => self.navigate(action, rows),
        }
        self.sync_theme_info();
        self.sync_color_target();
        if context != InputContext::Colors {
            self.ensure_selection_visible(rows);
        }
    }

    fn navigate(&mut self, action: Action, rows: usize) {
        list::navigate(&mut self.theme_list, self.results.len(), action, rows);
    }
}
