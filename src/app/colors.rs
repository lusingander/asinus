use ratatui::widgets::ListState;

use crate::{
    app::{Action, App, Notice, list, theme_target::ThemeTarget},
    color_details::ColorRole,
    theme::Theme,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFocus {
    Current,
    Colors,
}

pub struct ColorDetails {
    pub focus: Option<ColorFocus>,
    pub list: ListState,
    pub target: ThemeTarget,
}

impl Default for ColorDetails {
    fn default() -> Self {
        Self {
            focus: None,
            list: ListState::default().with_selected(Some(0)),
            target: ThemeTarget::Current,
        }
    }
}

impl ColorDetails {
    pub fn is_open(&self) -> bool {
        self.focus.is_some()
    }

    pub fn is_focused(&self) -> bool {
        self.focus == Some(ColorFocus::Colors)
    }

    pub fn role(&self) -> ColorRole {
        ColorRole::ALL[self.list.selected().unwrap_or(0)]
    }

    pub fn switch_focus(&mut self) {
        self.focus = self.focus.map(|focus| match focus {
            ColorFocus::Current => ColorFocus::Colors,
            ColorFocus::Colors => ColorFocus::Current,
        });
    }

    pub fn navigate(&mut self, action: Action, rows: usize) -> bool {
        if !matches!(
            action,
            Action::Next
                | Action::Previous
                | Action::First
                | Action::Last
                | Action::PageNext
                | Action::PagePrevious
        ) {
            return false;
        }
        list::navigate(&mut self.list, ColorRole::ALL.len(), action, rows);
        true
    }
}

impl App {
    pub fn color_theme(&self) -> Option<&Theme> {
        self.theme_for_target(self.color_details.target)
    }

    pub fn can_switch_color_target(&self) -> bool {
        !self.mode.is_compare() && self.can_switch_theme_target()
    }

    pub fn sync_color_target(&mut self) {
        if self.editor.is_none()
            && self.color_details.is_open()
            && self.color_theme().is_none()
            && self
                .theme_for_target(self.color_details.target.other())
                .is_some()
        {
            self.color_details.target = self.color_details.target.other();
        }
    }

    pub fn open_colors(&mut self) -> bool {
        let Some(target) = self.default_theme_target() else {
            self.notice = Some(Notice::Warning("No theme to inspect".into()));
            return false;
        };
        self.color_details.target = target;
        self.color_details.focus = Some(ColorFocus::Colors);
        true
    }

    pub fn close_colors(&mut self) {
        self.color_details.focus = None;
        if let Some(info) = self.suspended_theme_info.take() {
            self.theme_info = Some(info);
        }
    }

    pub fn toggle_colors(&mut self) {
        if self.color_details.is_open() {
            self.close_colors();
        } else {
            self.open_colors();
        }
    }
}
