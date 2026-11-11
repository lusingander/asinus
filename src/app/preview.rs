use crate::{
    app::{Action, App},
    preview::{ContentSize, Preset},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewField {
    Sample,
    Content,
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewSelection {
    pub preset: Preset,
    pub content: ContentSize,
    pub field: PreviewField,
}

impl App {
    pub fn select_preset(&mut self, preset: Preset, content: ContentSize) {
        self.preset = preset;
        self.preview_content = content;
        if !self.read_only {
            self.session.preset = preset;
        }
        self.record_session(false);
    }

    pub fn open_preview(&mut self) {
        self.preset_selection = Some(PreviewSelection {
            preset: self.preset,
            content: self.preview_content,
            field: PreviewField::Sample,
        });
    }

    pub fn update_preview(&mut self, action: Action) {
        let selection = self
            .preset_selection
            .as_mut()
            .expect("preview selector is open");
        match action {
            Action::FieldNext | Action::FieldPrevious => {
                selection.field = match selection.field {
                    PreviewField::Sample => PreviewField::Content,
                    PreviewField::Content => PreviewField::Sample,
                };
            }
            Action::ValueNext | Action::ValuePrevious => match selection.field {
                PreviewField::Sample => {
                    selection.preset = if action == Action::ValueNext {
                        selection.preset.next()
                    } else {
                        selection.preset.previous()
                    };
                }
                PreviewField::Content => {
                    let index = ContentSize::ALL
                        .iter()
                        .position(|content| *content == selection.content)
                        .unwrap();
                    let length = ContentSize::ALL.len();
                    let offset = if action == Action::ValueNext {
                        1
                    } else {
                        length - 1
                    };
                    selection.content = ContentSize::ALL[(index + offset) % length];
                }
            },
            Action::Confirm => {
                let selection = self.preset_selection.take().unwrap();
                if self.preset != selection.preset || self.preview_content != selection.content {
                    self.select_preset(selection.preset, selection.content);
                }
            }
            Action::Cancel => self.preset_selection = None,
            _ => {}
        }
    }
}
