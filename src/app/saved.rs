use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    app::{Action, App, Notice, list},
    search::Filter,
    text_input,
    workspace::{SavedFilter, Workspace, WorkspaceAction, WorkspaceChange},
};

pub struct SavedScreen {
    pub list: ListState,
    pub source_query: String,
}

#[derive(Debug, Clone)]
pub enum DialogKind {
    New { query: String },
    Rename { name: String, query: String },
    Delete { name: String, query: String },
}

pub struct FilterDialog {
    pub kind: DialogKind,
    pub input: Input,
}

impl FilterDialog {
    pub fn has_name(&self) -> bool {
        matches!(
            self.kind,
            DialogKind::New { .. } | DialogKind::Rename { .. }
        )
    }

    pub fn change(&self, workspace: &Workspace) -> Result<WorkspaceChange, String> {
        Ok(match &self.kind {
            DialogKind::New { query } => WorkspaceChange::New {
                name: workspace.name(self.input.value(), None)?,
                query: query.clone(),
            },
            DialogKind::Rename { name, .. } => WorkspaceChange::Rename {
                from: name.clone(),
                to: workspace.name(self.input.value(), Some(name))?,
            },
            DialogKind::Delete { name, .. } => WorkspaceChange::Delete { name: name.clone() },
        })
    }
}

impl App {
    pub fn open_saved(&mut self) {
        self.saved = Some(SavedScreen {
            list: ListState::default().with_selected(Some(0)),
            source_query: self.filter.query(),
        });
    }

    pub fn selected_saved_filter(&self) -> Option<&SavedFilter> {
        let index = self.saved.as_ref()?.list.selected()?.checked_sub(1)?;
        self.workspace.saved_filters.get(index)
    }

    pub fn selected_filter(&self) -> Option<(&str, &str)> {
        match self.saved.as_ref()?.list.selected()? {
            0 => Some(("Initial", "ignore:false")),
            _ => self
                .selected_saved_filter()
                .map(|saved| (saved.name.as_str(), saved.query.as_str())),
        }
    }

    pub fn saved_matches(&self, query: &str) -> usize {
        self.results_for(&Filter::parse(query).expect("validated saved query"))
            .len()
    }

    pub fn open_filter_dialog(&mut self, action: Action) {
        let kind = match action {
            Action::SaveFilter => DialogKind::New {
                query: self
                    .saved
                    .as_ref()
                    .map_or_else(|| self.filter.query(), |screen| screen.source_query.clone()),
            },
            Action::RenameFilter | Action::DeleteFilter => {
                let Some(saved) = self.selected_saved_filter() else {
                    return;
                };
                if action == Action::RenameFilter {
                    DialogKind::Rename {
                        name: saved.name.clone(),
                        query: saved.query.clone(),
                    }
                } else {
                    DialogKind::Delete {
                        name: saved.name.clone(),
                        query: saved.query.clone(),
                    }
                }
            }
            _ => return,
        };
        let input = Input::new(match &kind {
            DialogKind::Rename { name, .. } => name.clone(),
            _ => String::new(),
        });
        self.dialog = Some(FilterDialog { kind, input });
    }

    pub fn update_filter_dialog(&mut self, action: Action) {
        let mut dialog = self.dialog.take().expect("dialog owns input");
        match action {
            Action::Cancel => return,
            Action::Confirm => match dialog.change(&self.workspace) {
                Ok(change) => {
                    let unchanged =
                        matches!(&change, WorkspaceChange::Rename { from, to } if from == to);
                    if unchanged {
                        self.notice = Some(Notice::Message("Saved filter unchanged".into()));
                        return;
                    }
                    self.workspace_action = Some(WorkspaceAction::Filter(change));
                }
                Err(error) => self.notice = Some(Notice::Error(error)),
            },
            _ if dialog.has_name() => {
                text_input::update(&mut dialog.input, action);
            }
            _ => {}
        }
        self.dialog = Some(dialog);
    }

    pub fn update_saved(&mut self, action: Action, rows: usize) {
        match action {
            Action::Cancel => self.saved = None,
            Action::Confirm => {
                if let Some((_, query)) = self.selected_filter() {
                    let filter = Filter::parse(query).expect("validated filter query");
                    self.saved = None;
                    self.apply_filter(filter);
                    self.record_session(true);
                }
            }
            Action::SaveFilter | Action::RenameFilter | Action::DeleteFilter => {
                self.open_filter_dialog(action)
            }
            _ => {
                let length = self.workspace.saved_filters.len() + 1;
                let screen = self.saved.as_mut().unwrap();
                list::navigate(&mut screen.list, length, action, rows);
            }
        }
    }

    pub fn prepare_saved(&mut self, rows: usize) {
        let screen = self.saved.as_mut().unwrap();
        let length = self.workspace.saved_filters.len() + 1;
        let list = &mut screen.list;
        list.select(Some(list.selected().unwrap_or(0).min(length - 1)));
        *list.offset_mut() = list.offset().min(length.saturating_sub(rows));
    }

    pub fn finish_workspace_change(&mut self, change: &WorkspaceChange) {
        self.dialog = None;
        if let Some(screen) = &mut self.saved {
            let index = match change {
                WorkspaceChange::New { name, .. } => self
                    .workspace
                    .saved_filters
                    .iter()
                    .position(|s| &s.name == name)
                    .map(|i| i + 1),
                WorkspaceChange::Rename { to, .. } => self
                    .workspace
                    .saved_filters
                    .iter()
                    .position(|s| &s.name == to)
                    .map(|i| i + 1),
                WorkspaceChange::Delete { .. } => screen
                    .list
                    .selected()
                    .map(|i| i.min(self.workspace.saved_filters.len())),
            };
            screen.list.select(index);
        }
    }
}
