use crate::{
    app::{Action, App, Notice},
    marks::{Mark, MarkChange},
};

impl App {
    pub fn update_marks(&mut self, action: Action) {
        if let Some(theme) = self.theme() {
            let mark = if action == Action::ToggleFavorite {
                Mark::Favorite
            } else {
                Mark::Ignored
            };
            self.storage_action = Some(MarkChange {
                name: theme.name.to_owned(),
                mark: (self.marks.get(theme.name) != Some(&mark)).then_some(mark),
            });
        }
    }

    pub fn mark_saved(&mut self, change: MarkChange) {
        let selected = self.selected().map(|i| self.results[i]);
        let index = self.selected().unwrap_or(0);
        change.apply(&mut self.marks);
        self.results = self.results_for(&self.filter);
        let next = selected
            .and_then(|theme| self.results.iter().position(|i| *i == theme))
            .or_else(|| (!self.results.is_empty()).then(|| index.min(self.results.len() - 1)));
        self.theme_list.select(next);
        self.notice = Some(Notice::Success(format!(
            "{}: {}",
            change.name,
            match change.mark {
                Some(Mark::Favorite) => "Favorite saved",
                Some(Mark::Ignored) => "Ignore saved",
                None => "Mark cleared",
            }
        )));
    }
}
