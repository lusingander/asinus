use crate::{
    app::{App, Notice},
    marks::Marks,
    preview::Sample,
    search::Filter,
    workspace::{Workspace, WorkspaceAction, WorkspaceChange},
};

impl App {
    pub fn restore(&mut self, marks: Marks, workspace: Workspace, read_error: Option<String>) {
        self.read_only = read_error.is_some();
        self.notice = read_error.map(|reason| Notice::Warning(format!("Read-only: {reason}")));
        self.marks = marks;
        self.session = workspace.session.clone();
        self.preview_sample = Sample::BuiltIn(self.session.preset);
        self.preview_content = self.session.preview_content;
        self.compare_direction = self.session.layout.into();
        let query = self.session.query.as_deref().unwrap_or("ignore:false");
        let filter = Filter::parse(query).expect("validated session query");
        self.workspace = workspace;
        self.theme_list = Default::default();
        self.apply_filter(filter);
    }

    pub fn record_session(&mut self, condition: bool) {
        if self.read_only {
            return;
        }
        if condition && (self.session.query.is_some() || !self.filter.is_default()) {
            self.session.query = Some(self.filter.query());
        }
        self.session.preview_content = self.preview_content;
        self.session.layout = self.compare_direction.into();
        if self.session != self.workspace.session {
            self.workspace_action = Some(WorkspaceAction::Session);
        }
    }

    pub fn workspace_snapshot(
        &self,
        change: Option<&WorkspaceChange>,
    ) -> Result<Workspace, String> {
        let mut next = match change {
            Some(change) => self.workspace.changed(change)?,
            None => self.workspace.clone(),
        };
        next.session = self.session.clone();
        Ok(next)
    }

    pub fn workspace_saved(&mut self, workspace: Workspace, change: Option<WorkspaceChange>) {
        self.workspace = workspace;
        if let Some(change) = change {
            self.finish_workspace_change(&change);
            self.notice = Some(Notice::Success(change.message()));
        }
    }
}
