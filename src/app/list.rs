use ratatui::widgets::ListState;

use crate::app::Action;

pub fn navigate(list: &mut ListState, length: usize, action: Action, rows: usize) {
    let Some(last) = length.checked_sub(1) else {
        return;
    };
    let selected = list.selected().unwrap_or(0);
    let max_offset = length.saturating_sub(rows);
    match action {
        Action::Previous => list.select(Some(selected.saturating_sub(1))),
        Action::Next => list.select(Some((selected + 1).min(last))),
        Action::First => {
            list.select(Some(0));
            *list.offset_mut() = 0;
        }
        Action::Last => {
            list.select(Some(last));
            *list.offset_mut() = max_offset;
        }
        Action::PagePrevious => {
            list.select(Some(selected.saturating_sub(rows)));
            *list.offset_mut() = list.offset().saturating_sub(rows);
        }
        Action::PageNext => {
            list.select(Some(selected.saturating_add(rows).min(last)));
            *list.offset_mut() = list.offset().saturating_add(rows).min(max_offset);
        }
        _ => {}
    }
}
