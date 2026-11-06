use crate::{
    app::{Action, App, Notice, theme_target::ThemeTarget},
    copy::CopyFormat,
    theme::Theme,
};

pub struct CopySelection {
    pub target: ThemeTarget,
    pub format: CopyFormat,
}

pub struct CopyRequest {
    pub name: String,
    pub format: CopyFormat,
    pub text: String,
}

impl App {
    pub fn copy_theme(&self) -> Option<&Theme> {
        self.theme_for_target(self.copy_selection.as_ref()?.target)
    }

    pub fn open_copy(&mut self) {
        let Some(target) = self.default_theme_target() else {
            self.notice = Some(Notice::Warning("No theme to copy".into()));
            return;
        };
        self.copy_selection = Some(CopySelection {
            target,
            format: CopyFormat::Name,
        });
    }

    pub fn update_copy(&mut self, action: Action) {
        if action == Action::Cancel {
            self.copy_selection = None;
            return;
        }
        if action == Action::Confirm {
            let selection = self.copy_selection.as_ref().unwrap();
            let theme = self
                .copy_theme()
                .expect("copy target exists while selecting");
            self.copy_action = Some(CopyRequest {
                name: theme.name.to_owned(),
                format: selection.format,
                text: selection.format.text(theme),
            });
            return;
        }
        let can_switch = self.can_switch_theme_target();
        let selection = self.copy_selection.as_mut().unwrap();
        match action {
            Action::Next => selection.format = CopyFormat::Json,
            Action::Previous => selection.format = CopyFormat::Name,
            Action::SwitchThemeTarget if can_switch => {
                selection.target = selection.target.other();
            }
            _ => {}
        }
    }

    pub fn copy_finished(&mut self, request: CopyRequest, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.copy_selection = None;
                self.notice = Some(Notice::Success(format!(
                    "Copied {}: {}",
                    request.format.label(),
                    request.name
                )));
            }
            Err(error) => self.notice = Some(Notice::Error(format!("Copy failed: {error}"))),
        }
    }
}
