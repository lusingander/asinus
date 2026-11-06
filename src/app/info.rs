use std::ops::Range;

use crate::{
    app::{Action, App, Notice, theme_target::ThemeTarget},
    theme::{Credit, Theme},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoSection {
    Overview,
    Credits,
}

pub struct ThemeInfo {
    pub section: InfoSection,
    pub focused: bool,
    pub overview_offset: usize,
    overview_length: usize,
    theme_name: Option<&'static str>,
    pub target: ThemeTarget,
    pub selected: Option<usize>,
    pub offset: usize,
    ranges: Vec<Range<usize>>,
    rows: usize,
}

impl ThemeInfo {
    fn new(target: ThemeTarget, count: usize) -> Self {
        Self {
            section: InfoSection::Overview,
            focused: true,
            overview_offset: 0,
            overview_length: 0,
            theme_name: None,
            target,
            selected: (count > 0).then_some(0),
            offset: 0,
            ranges: Vec::new(),
            rows: 0,
        }
    }

    pub fn prepare_overview(&mut self, length: usize, rows: usize) {
        self.overview_length = length;
        self.rows = rows;
        self.overview_offset = self.overview_offset.min(length.saturating_sub(rows));
    }

    fn scroll_overview(&mut self, action: Action) {
        let max = self.overview_length.saturating_sub(self.rows);
        self.overview_offset = match action {
            Action::Next => self.overview_offset.saturating_add(1).min(max),
            Action::Previous => self.overview_offset.saturating_sub(1),
            Action::PageNext => self.overview_offset.saturating_add(self.rows).min(max),
            Action::PagePrevious => self.overview_offset.saturating_sub(self.rows),
            Action::First => 0,
            Action::Last => max,
            _ => self.overview_offset,
        };
    }

    pub fn prepare(&mut self, ranges: Vec<Range<usize>>, rows: usize) {
        let changed = self.ranges != ranges || self.rows != rows;
        self.ranges = ranges;
        self.rows = rows;
        self.offset = self.offset.min(self.max_offset());
        if changed {
            self.reveal_selected(false);
        }
    }

    fn max_offset(&self) -> usize {
        self.ranges
            .last()
            .map_or(0, |range| range.end)
            .saturating_sub(self.rows)
    }

    fn is_visible(&self, range: &Range<usize>) -> bool {
        range.start < self.offset + self.rows && range.end > self.offset
    }

    fn reveal_selected(&mut self, align_start: bool) {
        let Some(range) = self.selected.and_then(|index| self.ranges.get(index)) else {
            return;
        };
        if range.len() > self.rows {
            if align_start || !self.is_visible(range) {
                self.offset = range.start;
            }
        } else if range.start < self.offset {
            self.offset = range.start;
        } else if range.end > self.offset + self.rows {
            self.offset = range.end - self.rows;
        }
        self.offset = self.offset.min(self.max_offset());
    }

    fn page(&mut self, forward: bool) {
        if self.rows == 0 {
            return;
        }
        let before = self.offset;
        self.offset = if forward {
            self.offset.saturating_add(self.rows).min(self.max_offset())
        } else {
            self.offset.saturating_sub(self.rows)
        };
        let selected_visible = self
            .selected
            .and_then(|index| self.ranges.get(index))
            .is_some_and(|range| self.is_visible(range));
        if self.offset == before || !selected_visible {
            // At a viewport edge, paging still reaches the first or last visible credit.
            let select_last = if self.offset == before {
                forward
            } else {
                !forward
            };
            self.selected = if select_last {
                self.ranges.iter().rposition(|range| self.is_visible(range))
            } else {
                self.ranges.iter().position(|range| self.is_visible(range))
            };
        }
    }

    fn navigate(&mut self, action: Action, count: usize) {
        let Some(last) = count.checked_sub(1) else {
            return;
        };
        let before = self.selected;
        let selected = self.selected.unwrap_or(0);
        match action {
            Action::Next => self.selected = Some(selected.saturating_add(1).min(last)),
            Action::Previous => self.selected = Some(selected.saturating_sub(1)),
            Action::First => {
                self.selected = Some(0);
                self.offset = 0;
            }
            Action::Last => {
                self.selected = Some(last);
                self.offset = self.max_offset();
            }
            Action::PageNext | Action::PagePrevious => self.page(action == Action::PageNext),
            _ => return,
        }
        if self.selected != before && matches!(action, Action::Next | Action::Previous) {
            self.reveal_selected(true);
        }
    }
}

impl App {
    pub fn info_theme(&self) -> Option<&Theme> {
        self.theme_for_target(self.theme_info.as_ref()?.target)
    }

    pub fn info_is_credits(&self) -> bool {
        self.theme_info
            .as_ref()
            .is_some_and(|info| info.section == InfoSection::Credits)
    }

    pub fn can_switch_info_target(&self) -> bool {
        self.can_switch_theme_target() && (!self.mode.is_compare() || self.info_is_credits())
    }

    pub fn selected_credit(&self) -> Option<&Credit> {
        if !self.info_is_credits() {
            return None;
        }
        let selected = self.theme_info.as_ref()?.selected?;
        self.info_theme()?.meta.credits.get(selected)
    }

    pub fn sync_theme_info(&mut self) {
        if self.editor.is_some() {
            return;
        }
        let Some(info) = &self.theme_info else {
            return;
        };
        let target = if self.theme_for_target(info.target).is_none()
            && self.theme_for_target(info.target.other()).is_some()
        {
            info.target.other()
        } else {
            info.target
        };
        let theme = self.theme_for_target(target);
        let name = theme.map(|theme| theme.name);
        let count = theme.map_or(0, |theme| theme.meta.credits.len());
        let info = self.theme_info.as_mut().unwrap();
        if info.target != target || info.theme_name != name {
            info.target = target;
            info.theme_name = name;
            info.selected = (count > 0).then_some(0);
            info.offset = 0;
            info.ranges.clear();
        }
    }

    pub fn open_theme_info(&mut self) {
        if self.suspended_theme_info.is_some() {
            self.close_colors();
            return;
        }
        let Some(target) = self.default_theme_target() else {
            self.notice = Some(Notice::Warning("No theme to inspect".into()));
            return;
        };
        let count = self.theme_for_target(target).unwrap().meta.credits.len();
        self.theme_info = Some(ThemeInfo::new(target, count));
        self.sync_theme_info();
    }

    pub fn update_theme_info(&mut self, action: Action) -> bool {
        self.sync_theme_info();
        if matches!(action, Action::Cancel | Action::OpenThemeInfo) {
            self.theme_info = None;
            return true;
        }
        if matches!(action, Action::FieldNext | Action::FieldPrevious) {
            let info = self.theme_info.as_mut().unwrap();
            info.focused = !info.focused;
            return true;
        }
        if action == Action::ToggleColors {
            if self.color_details.is_open() {
                self.theme_info = None;
            } else if self.open_colors() {
                self.suspended_theme_info = self.theme_info.take();
            }
            return true;
        }
        if !self.theme_info.as_ref().unwrap().focused {
            return false;
        }
        if matches!(action, Action::ValueNext | Action::ValuePrevious) {
            self.theme_info.as_mut().unwrap().section = match action {
                Action::ValueNext => InfoSection::Credits,
                _ => InfoSection::Overview,
            };
            return true;
        }
        if action == Action::SwitchThemeTarget {
            if self.can_switch_info_target() {
                let info = self.theme_info.as_mut().unwrap();
                info.target = info.target.other();
                info.theme_name = None;
                self.sync_theme_info();
            }
            return true;
        }
        if action == Action::Confirm {
            self.browser_action = self.selected_credit().map(|credit| credit.link.to_owned());
            return true;
        }
        if matches!(
            action,
            Action::Next
                | Action::Previous
                | Action::First
                | Action::Last
                | Action::PageNext
                | Action::PagePrevious
        ) {
            let count = self
                .info_theme()
                .map_or(0, |theme| theme.meta.credits.len());
            let info = self.theme_info.as_mut().unwrap();
            match info.section {
                InfoSection::Overview => info.scroll_overview(action),
                InfoSection::Credits => info.navigate(action, count),
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::app::Action::{First, Last, Next, PageNext, PagePrevious, Previous};
    use crate::app::colors::ColorFocus::{self, Colors, Current};

    #[rstest]
    #[case(None, Action::ToggleColors, Some(Colors))]
    #[case(Some(Current), Action::ToggleColors, Some(Current))]
    #[case(Some(Colors), Action::ToggleColors, Some(Colors))]
    #[case(Some(Current), Action::Cancel, Some(Current))]
    #[case(Some(Colors), Action::Cancel, Some(Colors))]
    #[case(Some(Current), Action::OpenThemeInfo, Some(Current))]
    fn info_closes_or_suspends_without_losing_color_selection_or_focus(
        #[case] previous: Option<ColorFocus>,
        #[case] action: Action,
        #[case] expected: Option<ColorFocus>,
    ) {
        let mut app = App::default();
        app.update(Action::TogglePin, 14);
        app.update(Action::Next, 14);
        app.update(Action::ToggleCompare, 14);
        if let Some(focus) = previous {
            app.update(Action::ToggleColors, 14);
            app.update(Action::Next, 14);
            if focus == Current {
                app.update(Action::FieldNext, 14);
            }
        }
        let role = app.color_details.role();
        let current_name = app.theme().unwrap().name;
        app.update(Action::OpenThemeInfo, 14);
        app.update(Action::FieldNext, 14);
        app.update(action, 14);
        assert!(app.theme_info.is_none());
        assert_eq!(app.color_details.focus, expected);
        assert_eq!(app.color_details.role(), role);
        assert_eq!(app.theme().unwrap().name, current_name);
        if action == Action::ToggleColors {
            if expected == Some(Colors) {
                app.update(Action::Next, 14);
                assert_ne!(app.color_details.role(), role);
                assert_eq!(app.theme().unwrap().name, current_name);
            }
            app.update(Action::Cancel, 14);
            if previous.is_none() {
                assert!(app.theme_info.is_some());
                assert_eq!(app.context(), crate::app::InputContext::Browse);
            } else {
                assert!(app.theme_info.is_none());
            }
        }
    }

    #[rstest]
    #[case(Action::ToggleCompare)]
    #[case(Action::TogglePin)]
    fn leaving_comparison_keeps_colors_and_restores_suspended_info_when_closed(
        #[case] action: Action,
    ) {
        let mut app = crate::test_support::comparison_in_context(crate::app::InputContext::Browse);
        app.update(Action::Previous, 14);
        app.update(Action::OpenThemeInfo, 14);
        app.update(Action::ValueNext, 14);
        app.update(Action::TogglePanelMaximize, 14);
        app.update(Action::ToggleColors, 14);
        if action == Action::TogglePin {
            app.update(Action::FieldNext, 14);
        }
        let context = app.context();
        app.update(action, 14);
        assert!(!app.mode.is_compare());
        assert!(app.color_details.is_open());
        assert!(app.suspended_theme_info.is_some());
        assert!(app.theme_info.is_none());
        assert_eq!(app.context(), context);
        assert_eq!(app.panel_maximized(), Some(true));
        app.update(Action::Cancel, 14);
        assert!(app.suspended_theme_info.is_none());
        assert_eq!(app.context(), crate::app::InputContext::ThemeInfo);
        assert!(app.info_is_credits());
        assert_eq!(app.panel_maximized(), Some(true));
        assert!(app.details_maximized);
        app.update(Action::Cancel, 14);
        assert!(app.theme_info.is_none());
        app.update(Action::ToggleCompare, 14);
        assert!(app.theme_info.is_none());
        assert!(!app.color_details.is_open());
    }

    #[test]
    fn cancelled_query_preserves_credit_selection_and_confirmed_change_resets_it() {
        let mut app = App::default();
        app.update(Action::EditQuery, 14);
        app.update(Action::ClearInput, 14);
        crate::test_support::insert_text(&mut app, "Andromeda");
        app.update(Action::Confirm, 14);
        app.update(Action::OpenThemeInfo, 14);
        app.update(Action::ValueNext, 14);
        app.update(Action::Next, 14);
        app.update(Action::FieldNext, 14);
        for action in [Action::Cancel, Action::Confirm] {
            app.update(Action::EditQuery, 14);
            app.update(Action::ClearInput, 14);
            crate::test_support::insert_text(&mut app, "nord");
            app.sync_theme_info();
            assert_eq!(app.theme_info.as_ref().unwrap().selected, Some(1));
            app.update(action, 14);
            app.sync_theme_info();
            if action == Action::Cancel {
                assert_eq!(app.selected_credit().unwrap().name, "EliverLara");
            } else {
                assert_eq!(app.selected_credit().unwrap().name, "Teraskull");
                assert_eq!(app.theme_info.as_ref().unwrap().selected, Some(0));
            }
        }
    }

    #[test]
    fn comparison_keeps_overview_position_and_resets_credits_when_the_current_theme_changes() {
        let mut app = App::default();
        app.update(Action::TogglePin, 14);
        app.update(Action::Next, 14);
        app.update(Action::ToggleCompare, 14);
        app.update(Action::OpenThemeInfo, 14);
        app.theme_info.as_mut().unwrap().prepare_overview(13, 6);
        app.update(Action::Last, 6);
        app.update(Action::SwitchThemeTarget, 6);
        assert_eq!(
            app.theme_info.as_ref().unwrap().target,
            ThemeTarget::Current
        );
        app.update(Action::ValueNext, 6);
        app.update(Action::FieldNext, 6);
        app.update(Action::Next, 6);
        let info = app.theme_info.as_ref().unwrap();
        assert_eq!(app.theme().unwrap().name, "Aardvark Blue");
        assert_eq!(app.pinned_theme().unwrap().name, "3024 Day");
        assert_eq!(info.overview_offset, 7);
        assert_eq!(info.selected, None);
        assert_eq!(info.offset, 0);
        assert!(!info.focused);
        app.update(Action::FieldPrevious, 6);
        app.update(Action::SwitchThemeTarget, 6);
        assert_eq!(app.info_theme().unwrap().name, "3024 Day");
        assert!(app.selected_credit().is_some());
        app.update(Action::ValuePrevious, 6);
        assert_eq!(app.theme_info.as_ref().unwrap().overview_offset, 7);
        app.update(Action::Cancel, 6);
        assert!(app.theme_info.is_none());
        assert!(app.mode.is_compare());
    }

    #[rstest]
    #[case(&[Next], 1, 2)]
    #[case(&[Next, PageNext], 1, 5)]
    #[case(&[Next, PageNext, PageNext], 1, 6)]
    #[case(&[Next, PageNext, PageNext, PageNext], 2, 6)]
    #[case(&[Last, PagePrevious], 1, 3)]
    #[case(&[Last, PagePrevious, PagePrevious], 1, 0)]
    #[case(&[Last, PagePrevious, PagePrevious, PagePrevious], 0, 0)]
    #[case(&[Last, Previous], 1, 2)]
    #[case(&[Next, PageNext, First], 0, 0)]
    #[case(&[Next, PageNext, Last], 2, 6)]
    fn variable_height_credits_keep_selection_visible_and_allow_reading_oversized_items(
        #[case] actions: &[Action],
        #[case] selected: usize,
        #[case] offset: usize,
    ) {
        let mut info = ThemeInfo::new(ThemeTarget::Current, 3);
        info.prepare(vec![0..2, 2..7, 7..9], 3);
        for &action in actions {
            info.navigate(action, 3);
            // Preparing an unchanged frame must preserve manual paging inside a long credit.
            info.prepare(vec![0..2, 2..7, 7..9], 3);
            assert!(info.is_visible(&info.ranges[info.selected.unwrap()]));
        }
        assert_eq!(info.selected, Some(selected));
        assert_eq!(info.offset, offset);
    }

    #[test]
    fn item_navigation_scrolls_only_when_needed_and_resize_reveals_the_selection() {
        let mut info = ThemeInfo::new(ThemeTarget::Current, 3);
        info.prepare(vec![0..2, 2..4, 4..6], 4);
        info.navigate(Next, 3);
        assert_eq!(info.offset, 0);
        info.navigate(Next, 3);
        assert_eq!(info.offset, 2);
        info.navigate(Previous, 3);
        assert_eq!(info.offset, 2);
        info.prepare(vec![0..3, 3..6, 6..9], 2);
        assert_eq!(info.selected, Some(1));
        assert!(info.is_visible(&info.ranges[1]));
        info.prepare(vec![0..2, 2..4, 4..6], 6);
        assert_eq!(info.selected, Some(1));
        assert_eq!(info.offset, 0);
    }
}
